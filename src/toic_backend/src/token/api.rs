// https://github.com/dfinity/examples/blob/master/rust/tokenmania/backend/lib.rs

use std::{cell::RefCell, sync::Arc};

#[cfg(any(not(test), rust_analyzer))]
use ic_cdk::api::{caller, id, is_controller};
use ic_cdk::{query, update};
use ic_stable_structures::{BTreeMap, Cell};
pub use icrc_ledger_types::{
    icrc::generic_metadata_value::MetadataValue,
    icrc1::{
        account::Account,
        transfer::{BlockIndex, Memo, TransferArg, TransferError},
    },
    icrc2::{
        allowance::{Allowance, AllowanceArgs},
        approve::{ApproveArgs, ApproveError},
        transfer_from::{TransferFromArgs, TransferFromError},
    },
    icrc3::transactions::{Approve, Burn, Mint, Transaction, Transfer},
};
use lazy_static::lazy_static;

use crate::memory::{
    MEMORY_MANAGER, TOKEN_ACCOUNT_BALANCE_MEM_ID, TOKEN_ACCOUNT_STAKING_MEM_ID,
    TOKEN_CONFIG_MEM_ID, TOKEN_TX_LOG_MEM_ID,
};
#[cfg(any(not(test), rust_analyzer))]
use crate::utils::timestamp;

use super::{
    constant::TOKEN_DATA_IMAGE, to_approve_error, to_transfer_from_error, AccountBalanceRefCell,
    AccountOwnerBalanceRefCell, ConfigRefCell, Configuration, CreateTokenArgs, StakeTokenArgs,
    StorableToken, StorableTransaction, SupportedStandard, Tokens, TransactionLog,
    TransactionLogRefCell, TxInfo,
};

#[cfg(all(test, not(rust_analyzer)))]
use crate::utils::mocks::{caller, id, is_controller, timestamp};

const MAX_MEMO_SIZE: usize = 32;
const PERMITTED_DRIFT_NANOS: u64 = 60_000_000_000;
const TRANSACTION_WINDOW_NANOS: u64 = 24 * 60 * 60 * 1_000_000_000;

// Error codes
const MEMO_TOO_LONG_ERROR_CODE: usize = 0;
const SELF_TRANSFER_ERROR_CODE: usize = 1;
const INVALID_STAKE_ERROR_CODE: usize = 2;

thread_local! {
    static CONFIG: ConfigRefCell = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(TOKEN_CONFIG_MEM_ID)),
            Configuration::default()
        ).expect("failed to initialize the config cell")
    );

    static TRANSACTION_LOG: TransactionLogRefCell = RefCell::new(
        ic_stable_structures::Vec::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(TOKEN_TX_LOG_MEM_ID))
        ).expect("failed to initialize the transaction log")
    );

    static BALANCES: AccountBalanceRefCell = RefCell::new(
        BTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(TOKEN_ACCOUNT_BALANCE_MEM_ID))
        )
    );

    static STAKED: AccountOwnerBalanceRefCell = RefCell::new(
        BTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(TOKEN_ACCOUNT_STAKING_MEM_ID))
        )
    );
}

lazy_static! {
    pub static ref LEDGER_SERVICE: Arc<LedgerService> = Arc::new(LedgerService::default());
}

#[derive(Debug, Default)]
pub struct LedgerService;

impl LedgerService {
    pub fn transfer(&self, arg: TransferArg) -> Result<BlockIndex, TransferError> {
        icrc1_transfer(arg)
    }

    pub fn balance_of(&self, account: Account) -> Tokens {
        icrc1_balance_of(account)
    }

    pub fn stake(&self, arg: StakeTokenArgs) -> Result<BlockIndex, TransferError> {
        stake(arg)
    }

    pub fn locked_balance_of(&self, account: Account) -> Tokens {
        get_locked_balance(account)
    }
}

/// Calculates the total supply of tokens by traversing the transaction log.
fn total_supply() -> Tokens {
    TRANSACTION_LOG.with_borrow(|log| {
        let mut supply = Tokens::default();
        for tx_wrapper in log.iter() {
            let tx = tx_wrapper.0;
            if let Some(mint) = tx.mint {
                supply += mint.amount;
            }
            if let Some(burn) = tx.burn {
                supply -= burn.amount;
            }
            if let Some(transfer) = tx.transfer {
                if let Some(fee) = transfer.fee {
                    supply -= fee;
                }
            }
            if let Some(approve) = tx.approve {
                if let Some(fee) = approve.fee {
                    supply -= fee;
                }
            }
        }
        supply
    })
}

/// Retrieves the balance of an account from the cache.
fn get_cached_balance(account: Account) -> Tokens {
    BALANCES.with_borrow(|balances| {
        balances
            .get(&account)
            .map(|bal| bal.0.clone())
            .unwrap_or(Tokens::default())
    })
}

/// Retrieves the locked/staked balance of an account from the cache.
fn get_locked_balance(account: Account) -> Tokens {
    STAKED.with_borrow(|balances| {
        balances
            .get(&account.owner)
            .map(|bal| bal.0.clone())
            .unwrap_or(Tokens::default())
    })
}

/// Updates the balance of an account in the cache.
/// This function is called after a transaction is successfully applied.
fn update_balance(tx: &Transaction) -> Result<(), TransferError> {
    BALANCES.with_borrow_mut(|balances| {
        if let Some(mint) = &tx.mint {
            balances.insert(
                mint.to,
                StorableToken(
                    balances
                        .get(&mint.to)
                        .map(|bal| bal.0 + mint.amount.clone())
                        .unwrap_or_else(|| mint.amount.clone()),
                ),
            );
        }
        if let Some(burn) = &tx.burn {
            balances.insert(
                burn.from,
                StorableToken(
                    balances
                        .get(&burn.from)
                        .map(|bal| bal.0 - burn.amount.clone())
                        .ok_or(TransferError::InsufficientFunds {
                            balance: Tokens::default(),
                        })?,
                ),
            );
        }
        if let Some(transfer) = &tx.transfer {
            balances.insert(
                transfer.to,
                StorableToken(
                    balances
                        .get(&transfer.to)
                        .map(|bal| bal.0 + transfer.amount.clone())
                        .unwrap_or_else(|| transfer.amount.clone()),
                ),
            );
            balances.insert(
                transfer.from,
                StorableToken(
                    balances
                        .get(&transfer.from)
                        .map(|bal| {
                            bal.0
                                - transfer.amount.clone()
                                - transfer.fee.clone().unwrap_or_default()
                        })
                        .ok_or(TransferError::InsufficientFunds {
                            balance: Tokens::default(),
                        })?,
                ),
            );
        }
        if let Some(approve) = &tx.approve {
            if let Some(fee) = &approve.fee {
                balances.insert(
                    approve.from,
                    StorableToken(
                        balances
                            .get(&approve.from)
                            .map(|bal| bal.0 - fee.clone())
                            .ok_or(TransferError::InsufficientFunds {
                                balance: Tokens::default(),
                            })?,
                    ),
                );
            }
        }
        Ok(())
    })
}

/// Rebuilds the balances cache from the transaction log, if there's a discrepancy.
fn rebuild_balances_cache() {
    BALANCES.with_borrow_mut(|b| b.clear_new());
    TRANSACTION_LOG.with_borrow(|log| {
        for tx_wrapper in log.iter() {
            let tx = tx_wrapper.0;
            update_balance(&tx).expect("Failed to rebuild balance caches");
        }
    });
}

/// Calculates how much `spender` is allowed to spend from `account` at the moment
fn allowance(account: Account, spender: Account, now: u64) -> Allowance {
    TRANSACTION_LOG.with_borrow(|log| {
        let mut allowance = 0_usize.into();
        let mut last_approval_expiry = None;
        for tx in log.iter() {
            // Reset expired approval
            if let Some(expires_at) = last_approval_expiry {
                if expires_at < tx.0.timestamp {
                    allowance = 0_usize.into();
                    last_approval_expiry = None;
                }
            }
            // Add pending approval
            if let Some(approve) = tx.0.approve {
                if approve.from == account && approve.spender == spender {
                    allowance = approve.amount;
                    last_approval_expiry = approve.expires_at;
                }
            }
            if let Some(transfer) = tx.0.transfer {
                if transfer.from == account && transfer.spender == Some(spender) {
                    allowance -= transfer.amount;
                    if let Some(fee) = transfer.fee {
                        allowance -= fee;
                    }
                }
            }
        }
        if let Some(expires_at) = last_approval_expiry {
            if expires_at < now {
                allowance = 0_usize.into();
                last_approval_expiry = None;
            }
        }
        Allowance {
            allowance,
            expires_at: last_approval_expiry,
        }
    })
}

/// validates `created_at_time` tobe inside the span of allowed timestamps window
fn validate_created_at_time(created_at_time: Option<u64>, now: u64) -> Result<(), TransferError> {
    if let Some(tx_time) = created_at_time {
        if tx_time > now && tx_time - now > TRANSACTION_WINDOW_NANOS + PERMITTED_DRIFT_NANOS {
            return Err(TransferError::CreatedInFuture { ledger_time: now });
        }
        if tx_time < now && now - tx_time > TRANSACTION_WINDOW_NANOS + PERMITTED_DRIFT_NANOS {
            return Err(TransferError::TooOld);
        }
    }
    Ok(())
}

/// validates Memo length of the transaction
fn validate_memo(memo: Option<&Memo>) -> Result<(), TransferError> {
    if let Some(memo) = memo {
        if memo.0.len() > MAX_MEMO_SIZE {
            return Err(TransferError::GenericError {
                error_code: MEMO_TOO_LONG_ERROR_CODE.into(),
                message: "Memo too long".into(),
            });
        }
    }
    Ok(())
}

fn validate_account(from: Account, to: Option<Account>) -> Result<(), TransferError> {
    if Some(from) == to {
        return Err(TransferError::GenericError {
            error_code: SELF_TRANSFER_ERROR_CODE.into(),
            message: "Cannot transfer to the same account".to_string(),
        });
    }
    Ok(())
}

/// records the validated transaction into the transaction log
fn record_valid_transaction(tx: &StorableTransaction) -> BlockIndex {
    let block_index = TRANSACTION_LOG.with_borrow_mut(|log| {
        let idx = log.len();
        log.push(tx).expect("Failed to save transaction");
        idx.into()
    });
    block_index
}

impl PartialEq<Transaction> for TxInfo {
    fn eq(&self, other: &Transaction) -> bool {
        if self.is_approval {
            if let Some(approve) = &other.approve {
                self.from == approve.from
                    && self.spender == Some(approve.spender)
                    && self.amount == approve.amount
                    && self.expected_allowance == approve.expected_allowance
                    && self.expires_at == approve.expires_at
                    && self.memo == approve.memo
                    && self.created_at_time == approve.created_at_time
            } else {
                false
            }
        } else {
            let minting_account = CONFIG.with_borrow(|config| config.get().minting_account);
            if let Some(burn) = &other.burn {
                self.to == minting_account
                    && self.from == burn.from
                    && self.amount == burn.amount
                    && self.spender == burn.spender
                    && self.memo == burn.memo
                    && self.created_at_time == burn.created_at_time
            } else if let Some(mint) = &other.mint {
                Some(self.from) == minting_account
                    && self.to == Some(mint.to)
                    && self.amount == mint.amount
                    && self.memo == mint.memo
                    && self.created_at_time == mint.created_at_time
            } else if let Some(transfer) = &other.transfer {
                self.from == transfer.from
                    && self.to == Some(transfer.to)
                    && self.amount == transfer.amount
                    && self.spender == transfer.spender
                    && self.memo == transfer.memo
                    && self.created_at_time == transfer.created_at_time
            } else {
                false
            }
        }
    }
}

impl PartialEq for StorableTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/// tries to find a transaction in the transaction log
fn find_tx(tx: &TxInfo) -> Option<BlockIndex> {
    TRANSACTION_LOG.with_borrow(|log| {
        for (i, stored_tx_wrapper) in log.iter().enumerate() {
            if tx == &stored_tx_wrapper.0 {
                return Some(i.into());
            }
        }
        None
    })
}

fn map_tx_approval(tx: TxInfo, now: u64) -> Result<StorableTransaction, TransferError> {
    let transfer_fee = CONFIG.with_borrow(|config| config.get().transfer_fee.clone());
    Ok(StorableTransaction(Transaction::approve(
        Approve {
            from: tx.from,
            spender: tx.spender.expect("Bug: failed to forward spender"),
            amount: tx.amount,
            expected_allowance: tx.expected_allowance,
            expires_at: tx.expires_at,
            memo: tx.memo,
            fee: Some(transfer_fee),
            created_at_time: tx.created_at_time,
        },
        now,
    )))
}

fn map_tx_mint(tx: TxInfo, now: u64) -> Result<StorableTransaction, TransferError> {
    Ok(StorableTransaction(Transaction::mint(
        Mint {
            amount: tx.amount,
            to: tx.to.expect("Bug: failed to forward mint receiver"),
            memo: tx.memo,
            created_at_time: tx.created_at_time,
        },
        now,
    )))
}

fn map_tx_burnt(tx: TxInfo, now: u64) -> Result<StorableTransaction, TransferError> {
    let transfer_fee = CONFIG.with_borrow(|config| config.get().transfer_fee.clone());
    if tx.amount < transfer_fee {
        return Err(TransferError::BadBurn {
            min_burn_amount: transfer_fee.clone(),
        });
    }
    let balance = get_cached_balance(tx.from);
    if balance < tx.amount.clone() + transfer_fee {
        return Err(TransferError::InsufficientFunds { balance });
    }
    Ok(StorableTransaction(Transaction::burn(
        Burn {
            amount: tx.amount,
            from: tx.from,
            spender: tx.spender,
            memo: tx.memo,
            created_at_time: tx.created_at_time,
        },
        now,
    )))
}

fn map_tx_transfer(tx: TxInfo, now: u64) -> Result<StorableTransaction, TransferError> {
    let transfer_fee = CONFIG.with_borrow(|config| config.get().transfer_fee.clone());
    let balance = get_cached_balance(tx.from);
    if balance < tx.amount.clone() + transfer_fee.clone() {
        return Err(TransferError::InsufficientFunds { balance });
    }
    Ok(StorableTransaction(Transaction::transfer(
        Transfer {
            amount: tx.amount,
            from: tx.from,
            to: tx.to.expect("Bug: failed to forward transfer receiver"),
            spender: tx.spender,
            memo: tx.memo,
            fee: Some(transfer_fee),
            created_at_time: tx.created_at_time,
        },
        now,
    )))
}

/// Turns TxInfo into a validated transaction
fn map_tx(tx: TxInfo, now: u64) -> Result<StorableTransaction, TransferError> {
    // Deduplication only happens if `created_at_time` is set
    if tx.created_at_time.is_some() {
        if let Some(duplicate_of) = find_tx(&tx) {
            return Err(TransferError::Duplicate { duplicate_of });
        }
    }
    if let Some(specified_fee) = tx.fee.clone() {
        let expected_fee = CONFIG.with_borrow(|config| config.get().transfer_fee.clone());
        if specified_fee != expected_fee {
            return Err(TransferError::BadFee { expected_fee });
        }
    }

    if tx.is_approval {
        return map_tx_approval(tx, now);
    } else if let Some(minter) = CONFIG.with_borrow(|config| config.get().minting_account) {
        if Some(tx.from) == Some(minter) {
            return map_tx_mint(tx, now);
        } else if tx.to == Some(minter) {
            return map_tx_burnt(tx, now);
        }
    }
    map_tx_transfer(tx, now)
}

/// Runs validity checks and records the transaction followed by updating balance cahches if it is valid
fn apply_tx(tx: TxInfo) -> Result<BlockIndex, TransferError> {
    validate_account(tx.from, tx.to)?;
    validate_memo(tx.memo.as_ref())?;
    let now = timestamp();
    validate_created_at_time(tx.created_at_time, now)?;
    let transaction = map_tx(tx, now)?;
    let block = record_valid_transaction(&transaction);
    update_balance(&transaction.0)?;
    Ok(block)
}

fn stake_token(from: Account, amount: Tokens) -> Result<BlockIndex, TransferError> {
    let tx = TxInfo {
        from,
        to: Some(stake_account_address()),
        amount: amount.clone(),
        spender: None,
        memo: None,
        fee: None,
        created_at_time: None,
        expected_allowance: None,
        expires_at: None,
        is_approval: false,
    };
    let block = apply_tx(tx)?;
    STAKED.with_borrow_mut(|m| {
        let prev = m.get(&from.owner).map(|s| s.0).unwrap_or_default();
        m.insert(from.owner, StorableToken(prev + amount));
    });
    Ok(block)
}

#[update]
fn create_token(args: Option<CreateTokenArgs>) -> Result<String, String> {
    let caller = caller();
    if !is_controller(&caller) {
        return Err("Unauthorized operation".to_string());
    }

    if token_created() {
        return Err("Token already created".to_string());
    }

    let args = if args.is_none() {
        CreateTokenArgs {
            token_name: "TOIC".to_string(),
            token_symbol: "TOIC".to_string(),
            token_logo: TOKEN_DATA_IMAGE.to_string(),
            initial_supply: 5_000_000_000_000_usize.into(),
            transfer_fee: 100_usize.into(),
        }
    } else {
        args.unwrap()
    };

    let minting_account = Account {
        owner: caller,
        subaccount: None,
    };
    let init_tx = StorableTransaction(Transaction::mint(
        Mint {
            amount: args.initial_supply,
            to: minting_account,
            memo: None,
            created_at_time: None,
        },
        timestamp(),
    ));
    record_valid_transaction(&init_tx);
    CONFIG.with_borrow_mut(|config| {
        config
            .set(Configuration {
                token_name: args.token_name,
                token_symbol: args.token_symbol,
                token_logo: args.token_logo,
                transfer_fee: args.transfer_fee,
                decimals: 8,
                minting_account: Some(minting_account),
                token_created: true,
            })
            .map_err(|_| "Failed to set initial config".to_string())?;
        Ok("Token created".to_string())
    })
}

#[query]
fn token_created() -> bool {
    CONFIG.with_borrow(|config| config.get().token_created)
}

#[update]
fn delete_token() -> Result<String, String> {
    let caller = caller();
    if !is_controller(&caller) {
        return Err("Unauthorized operation".to_string());
    }

    if !token_created() {
        return Err("Token not created".to_string());
    };

    if caller != CONFIG.with_borrow(|config| config.get().minting_account.clone().unwrap().owner) {
        return Err("Caller is not the token creator".to_string());
    };

    // Reset stable memory
    CONFIG.with_borrow_mut(|config| {
        config.set(Configuration::default()).unwrap();
    });
    TRANSACTION_LOG.with_borrow_mut(|cell| {
        let memory = MEMORY_MANAGER.with_borrow_mut(|mm| mm.get(TOKEN_TX_LOG_MEM_ID));
        *cell = TransactionLog::new(memory).unwrap();
    });
    Ok("Token deleted".to_string())
}

const STAKE_SUBACCOUNT: [u8; 32] = [137; 32];

fn stake_account_address() -> Account {
    Account {
        owner: id(),
        subaccount: Some(STAKE_SUBACCOUNT.into()),
    }
}

#[query]
fn stake(arg: StakeTokenArgs) -> Result<BlockIndex, TransferError> {
    let from = Account {
        owner: caller(),
        subaccount: arg.from_subaccount,
    };
    stake_token(from, arg.amount)
}

#[update]
fn icrc1_transfer(arg: TransferArg) -> Result<BlockIndex, TransferError> {
    let from = Account {
        owner: caller(),
        subaccount: arg.from_subaccount,
    };
    let tx = TxInfo {
        from,
        to: Some(arg.to),
        amount: arg.amount,
        spender: None,
        memo: arg.memo,
        fee: arg.fee,
        created_at_time: arg.created_at_time,
        expected_allowance: None,
        expires_at: None,
        is_approval: false,
    };
    apply_tx(tx)
}

#[query]
fn icrc1_balance_of(account: Account) -> Tokens {
    get_cached_balance(account)
}

#[query]
fn icrc1_total_supply() -> Tokens {
    total_supply()
}

#[query]
fn icrc1_minting_account() -> Option<Account> {
    CONFIG.with_borrow(|config| config.get().minting_account.clone())
}

#[query]
fn icrc1_name() -> String {
    CONFIG.with_borrow(|config| config.get().token_name.clone())
}

#[query]
fn icrc1_token_symbol() -> String {
    CONFIG.with_borrow(|config| config.get().token_symbol.clone())
}

#[query]
fn icrc1_decimals() -> u8 {
    CONFIG.with_borrow(|config| config.get().decimals)
}

#[query]
fn icrc1_fee() -> Tokens {
    CONFIG.with_borrow(|config| config.get().transfer_fee.clone())
}

#[query]
fn icrc1_metadata() -> Vec<(String, MetadataValue)> {
    vec![
        ("icrc1:name".to_string(), MetadataValue::Text(icrc1_name())),
        (
            "icrc1:symbol".to_string(),
            MetadataValue::Text(icrc1_token_symbol()),
        ),
        (
            "icrc1:decimals".to_string(),
            MetadataValue::Nat(icrc1_decimals().into()),
        ),
        ("icrc1:fee".to_string(), MetadataValue::Nat(icrc1_fee())),
        (
            "icrc1:logo".to_string(),
            MetadataValue::Text(CONFIG.with_borrow(|config| config.get().token_logo.clone())),
        ),
    ]
}

#[query]
fn icrc1_supported_standards() -> Vec<SupportedStandard> {
    vec![
        SupportedStandard {
            name: "ICRC-1".to_string(),
            url: "https://github.com/dfinity/ICRC-1/tree/main/standards/ICRC-1".to_string(),
        },
        SupportedStandard {
            name: "ICRC-2".to_string(),
            url: "https://github.com/dfinity/ICRC-1/tree/main/standards/ICRC-2".to_string(),
        },
    ]
}

#[update]
fn icrc2_approve(arg: ApproveArgs) -> Result<BlockIndex, ApproveError> {
    validate_memo(arg.memo.as_ref()).map_err(to_approve_error)?;
    let approver_account = Account {
        owner: caller(),
        subaccount: arg.from_subaccount,
    };
    let now = timestamp();
    if let Some(expected_allowance) = arg.expected_allowance.as_ref() {
        let current_allowance = allowance(approver_account, arg.spender, now).allowance;
        if current_allowance != *expected_allowance {
            return Err(ApproveError::AllowanceChanged { current_allowance });
        }
    }
    let tx = TxInfo {
        from: approver_account,
        to: None,
        amount: arg.amount,
        spender: Some(arg.spender),
        memo: arg.memo,
        fee: arg.fee,
        created_at_time: arg.created_at_time,
        expected_allowance: arg.expected_allowance,
        expires_at: arg.expires_at,
        is_approval: true,
    };
    apply_tx(tx).map_err(to_approve_error)
}

#[update]
fn icrc2_transfer_from(arg: TransferFromArgs) -> Result<BlockIndex, TransferFromError> {
    if caller() == arg.from.owner {
        return icrc1_transfer(TransferArg {
            to: arg.to,
            from_subaccount: arg.from.subaccount,
            amount: arg.amount,
            fee: arg.fee,
            memo: arg.memo,
            created_at_time: arg.created_at_time,
        })
        .map_err(to_transfer_from_error);
    }
    validate_memo(arg.memo.as_ref()).map_err(to_transfer_from_error)?;
    let spender = Account {
        owner: caller(),
        subaccount: arg.spender_subaccount,
    };
    let now = timestamp();
    let allowance = allowance(arg.from, spender, now);
    let transfer_fee = CONFIG.with_borrow(|config| config.get().transfer_fee.clone());
    if allowance.allowance < arg.amount.clone() + transfer_fee {
        return Err(TransferFromError::InsufficientAllowance {
            allowance: allowance.allowance,
        });
    }
    let tx = TxInfo {
        from: arg.from,
        to: Some(arg.to),
        amount: arg.amount,
        spender: Some(spender),
        memo: arg.memo,
        fee: arg.fee,
        created_at_time: arg.created_at_time,
        expected_allowance: None,
        expires_at: None,
        is_approval: false,
    };
    apply_tx(tx).map_err(to_transfer_from_error)
}

#[query]
fn icrc2_allowance(arg: AllowanceArgs) -> Allowance {
    let now = timestamp();
    allowance(arg.account, arg.spender, now)
}

#[cfg(test)]
mod tests {
    use candid::{Nat, Principal};
    use icrc_ledger_types::{
        icrc1::{
            account::Account,
            transfer::{TransferArg, TransferError},
        },
        icrc2::{allowance::AllowanceArgs, approve::ApproveArgs, transfer_from::TransferFromArgs},
    };

    use crate::{
        token::{
            api::{
                create_token, delete_token, icrc1_balance_of, icrc1_decimals, icrc1_fee,
                icrc1_metadata, icrc1_minting_account, icrc1_name, icrc1_supported_standards,
                icrc1_token_symbol, icrc1_total_supply, icrc1_transfer, icrc2_allowance,
                icrc2_approve, icrc2_transfer_from, token_created, validate_created_at_time,
                BALANCES, TRANSACTION_LOG, TRANSACTION_WINDOW_NANOS,
            },
            CreateTokenArgs,
        },
        utils::mocks::{caller, reset_timestamp, set_caller, timestamp},
        Tokens,
    };

    fn mock_principal() -> Principal {
        Principal::from_text("aaaaa-aa").unwrap()
    }

    fn mock_principal_2() -> Principal {
        Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap()
    }

    fn create_token_with_default_args() -> Result<String, String> {
        let args = CreateTokenArgs {
            token_name: "TestToken".to_string(),
            token_symbol: "TT".to_string(),
            token_logo: "logo".to_string(),
            initial_supply: 1_000_000_000_usize.into(),
            transfer_fee: 1_000_usize.into(),
        };
        create_token(Some(args))
    }

    #[test]
    fn test_create_token() {
        let result = create_token_with_default_args();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Token created".to_string());
        assert!(token_created());
    }

    #[test]
    fn test_create_token_already_created() {
        create_token_with_default_args().unwrap();
        let result = create_token_with_default_args();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Token already created".to_string());
    }

    #[test]
    fn test_delete_token() {
        create_token_with_default_args().unwrap();
        let result = delete_token();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Token deleted".to_string());
        assert!(!token_created());
    }

    #[test]
    fn test_delete_token_not_created() {
        let result = delete_token();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Token not created".to_string());
    }

    #[test]
    fn test_icrc1_metadata() {
        create_token_with_default_args().unwrap();
        let minting_account = icrc1_minting_account().unwrap();
        assert_eq!(minting_account.owner, caller());
        assert_eq!(minting_account.subaccount, None);

        let expected_fee: Tokens = 1_000_usize.into();
        assert_eq!(icrc1_fee(), expected_fee);
        assert_eq!(icrc1_decimals(), 8);
        assert_eq!(icrc1_name(), "TestToken".to_string());
        assert_eq!(icrc1_token_symbol(), "TT".to_string());
        assert_eq!(icrc1_metadata().len(), 5);
        assert_eq!(icrc1_metadata()[0].0, "icrc1:name");
        assert_eq!(icrc1_metadata()[1].0, "icrc1:symbol");
        assert_eq!(icrc1_metadata()[2].0, "icrc1:decimals");
        assert_eq!(icrc1_metadata()[3].0, "icrc1:fee");
        assert_eq!(icrc1_metadata()[4].0, "icrc1:logo");
    }

    #[test]
    fn test_icrc1_standards() {
        let standards = icrc1_supported_standards();
        assert_eq!(standards.len(), 2);
        assert_eq!(standards[0].name, "ICRC-1");
        assert_eq!(standards[1].name, "ICRC-2");
    }

    #[test]
    fn test_icrc1_transfer_mint() {
        create_token_with_default_args().unwrap();

        reset_timestamp(TRANSACTION_WINDOW_NANOS * 3);
        let valid_time = Some(timestamp() - TRANSACTION_WINDOW_NANOS / 2);

        let to = Account {
            owner: mock_principal(),
            subaccount: None,
        };

        let transfer_arg = TransferArg {
            from_subaccount: None,
            to,
            amount: 10_000_usize.into(),
            fee: None,
            memo: None,
            created_at_time: valid_time,
        };

        let result = icrc1_transfer(transfer_arg.clone());
        assert!(result.is_ok());

        // Prevent duplicate transaction
        let result = icrc1_transfer(transfer_arg);
        assert!(matches!(result, Err(TransferError::Duplicate { .. })));
    }

    #[test]
    fn test_icrc1_transfer_burn() {
        create_token_with_default_args().unwrap();

        reset_timestamp(TRANSACTION_WINDOW_NANOS * 3);
        set_caller(Some(&mock_principal().to_string()));

        let to = icrc1_minting_account().unwrap();
        let valid_time = Some(timestamp() - TRANSACTION_WINDOW_NANOS / 2);

        // Burn 100 tokens failed, because the fee is 1_000
        let transfer_arg = TransferArg {
            from_subaccount: None,
            to,
            amount: 100_usize.into(),
            fee: None,
            memo: None,
            created_at_time: valid_time,
        };
        let result = icrc1_transfer(transfer_arg);
        assert!(result.is_err());
        assert!(matches!(result, Err(TransferError::BadBurn { .. })));

        // Burn 10_000 tokens failed, because account has not enough balance
        let transfer_arg = TransferArg {
            from_subaccount: None,
            to,
            amount: 10_000_usize.into(),
            fee: None,
            memo: None,
            created_at_time: valid_time,
        };
        let result = icrc1_transfer(transfer_arg);
        assert!(matches!(
            result,
            Err(TransferError::InsufficientFunds { .. })
        ));

        // Burn 10_000 tokens success
        set_caller(None);
        let transfer_arg = TransferArg {
            from_subaccount: None,
            to: Account {
                owner: mock_principal(),
                subaccount: None,
            },
            amount: 100_000_usize.into(),
            fee: None,
            memo: None,
            created_at_time: None,
        };
        icrc1_transfer(transfer_arg).unwrap();
        set_caller(Some(&mock_principal().to_string()));
        let transfer_arg = TransferArg {
            from_subaccount: None,
            to,
            amount: 10_000_usize.into(),
            fee: None,
            memo: None,
            created_at_time: valid_time,
        };
        let result = icrc1_transfer(transfer_arg.clone());
        assert!(result.is_ok());

        // Prevent duplicate transaction
        let result = icrc1_transfer(transfer_arg);
        assert!(matches!(result, Err(TransferError::Duplicate { .. })));
    }

    #[test]
    fn test_icrc1_balance_of() {
        create_token_with_default_args().unwrap();

        let account = Account {
            owner: mock_principal(),
            subaccount: None,
        };
        icrc1_transfer(TransferArg {
            from_subaccount: None,
            to: account,
            fee: None,
            created_at_time: None,
            memo: None,
            amount: 1000_usize.into(),
        })
        .unwrap();

        assert_eq!(BALANCES.with_borrow(|b| b.len()), 1);
        assert_eq!(TRANSACTION_LOG.with_borrow(|l| l.len()), 2);

        let balance = icrc1_balance_of(account);
        let expected: Tokens = 1_000_usize.into();
        assert_eq!(balance, expected);
    }

    #[test]
    fn test_icrc1_total_supply() {
        create_token_with_default_args().unwrap();

        let total_supply = icrc1_total_supply();
        let expected: Tokens = 1_000_000_000_usize.into();
        assert_eq!(total_supply, expected);
    }

    #[test]
    fn test_validate_created_at_time() {
        reset_timestamp(TRANSACTION_WINDOW_NANOS * 3);
        let now = timestamp();
        let valid_time = Some(now - TRANSACTION_WINDOW_NANOS / 2);
        let result = validate_created_at_time(valid_time, now);
        assert!(result.is_ok());

        let invalid_time = Some(now - TRANSACTION_WINDOW_NANOS * 2);
        let result = validate_created_at_time(invalid_time, now);
        assert!(result.is_err());
        reset_timestamp(0);
    }

    #[test]
    fn test_icrc2_allowance_initial() {
        create_token_with_default_args().unwrap();

        let spender = Account {
            owner: mock_principal(),
            subaccount: None,
        };
        let creditor = Account {
            owner: mock_principal_2(),
            subaccount: None,
        };

        // fill the balance of the creditor
        let fill_creditor_arg = TransferArg {
            from_subaccount: None,
            to: creditor,
            amount: 100_000_usize.into(),
            fee: None,
            memo: None,
            created_at_time: None,
        };
        icrc1_transfer(fill_creditor_arg).unwrap();
        set_caller(Some(&mock_principal().to_string()));
        assert!(BALANCES.with_borrow(|b| b.contains_key(&creditor)));

        let allowance_result = icrc2_allowance(AllowanceArgs {
            account: creditor,
            spender,
        });

        assert_eq!(allowance_result.allowance, Nat::from(0_usize));
        assert!(allowance_result.expires_at.is_none());
    }
}
