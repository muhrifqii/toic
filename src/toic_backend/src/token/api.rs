// https://github.com/dfinity/examples/blob/master/rust/tokenmania/backend/lib.rs

use std::cell::RefCell;

use ic_cdk::{query, update};
use ic_stable_structures::Cell;
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

use crate::{
    memory::{MEMORY_MANAGER, TOKEN_CONFIG_MEM_ID, TOKEN_TX_LOG_MEM_ID},
    utils::timestamp,
};

use super::types::{
    to_approve_error, to_transfer_from_error, ConfigRefCell, Configuration, CreateTokenArgs,
    SupportedStandard, Tokens, TransactionLog, TransactionLogRefCell, TransactionWrapper, TxInfo,
};

const MAX_MEMO_SIZE: usize = 32;
const PERMITTED_DRIFT_NANOS: u64 = 60_000_000_000;
const TRANSACTION_WINDOW_NANOS: u64 = 24 * 60 * 60 * 1_000_000_000;

// Error codes
const MEMO_TOO_LONG_ERROR_CODE: usize = 0;

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

/// Calculates the current balance of an account, by traversing the transaction log.
/// This is a naive implementation and may not be efficient for large transaction logs.
fn get_balance(account: Account) -> Tokens {
    TRANSACTION_LOG.with_borrow(|log| {
        let mut balance = Tokens::default();
        for tx_wrapper in log.iter() {
            let tx = tx_wrapper.0;
            if let Some(mint) = tx.mint {
                if mint.to == account {
                    balance += mint.amount;
                }
            }
            if let Some(burn) = tx.burn {
                if burn.from == account {
                    balance -= burn.amount;
                }
            }
            if let Some(transfer) = tx.transfer {
                if transfer.to == account {
                    balance += transfer.amount.clone();
                }
                if transfer.from == account {
                    balance -= transfer.amount;
                    if let Some(fee) = transfer.fee {
                        balance -= fee;
                    }
                }
            }
            if let Some(approve) = tx.approve {
                if let Some(fee) = approve.fee {
                    balance -= fee;
                }
            }
        }
        balance
    })
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

/// records the validated transaction into the transaction log
fn record_valid_transaction(tx: &TransactionWrapper) -> BlockIndex {
    TRANSACTION_LOG.with_borrow_mut(|log| {
        let idx = log.len();
        log.push(tx).expect("Failed to save transaction");
        idx.into()
    })
}

/// tries to find a transaction in the transaction log
fn find_tx(tx: &TxInfo) -> Option<BlockIndex> {
    TRANSACTION_LOG.with_borrow(|log| {
        for (i, stored_tx_wrapper) in log.iter().enumerate() {
            let stored_tx = stored_tx_wrapper.0;
            if tx.is_approval {
                if let Some(approve) = stored_tx.approve {
                    if tx.from == approve.from
                        && tx.spender == Some(approve.spender)
                        && tx.amount == approve.amount
                        && tx.expected_allowance == approve.expected_allowance
                        && tx.expires_at == approve.expires_at
                        && tx.memo == approve.memo
                        && tx.created_at_time == approve.created_at_time
                    {
                        return Some(i.into());
                    }
                }
            } else {
                let minting_account = CONFIG.with_borrow(|config| config.get().minting_account);
                if let Some(burn) = stored_tx.burn {
                    if tx.to == minting_account
                        && tx.from == burn.from
                        && tx.amount == burn.amount
                        && tx.spender == burn.spender
                        && tx.memo == burn.memo
                        && tx.created_at_time == burn.created_at_time
                    {
                        return Some(i.into());
                    }
                }
                if let Some(mint) = stored_tx.mint {
                    if Some(tx.from) == minting_account
                        && tx.to == Some(mint.to)
                        && tx.amount == mint.amount
                        && tx.memo == mint.memo
                        && tx.created_at_time == mint.created_at_time
                    {
                        return Some(i.into());
                    }
                }
                if let Some(transfer) = stored_tx.transfer {
                    if tx.from == transfer.from
                        && tx.to == Some(transfer.to)
                        && tx.amount == transfer.amount
                        && tx.spender == transfer.spender
                        && tx.memo == transfer.memo
                        && tx.created_at_time == transfer.created_at_time
                    {
                        return Some(i.into());
                    }
                }
            }
        }
        None
    })
}

fn map_tx_approval(tx: TxInfo, now: u64) -> Result<TransactionWrapper, TransferError> {
    let transfer_fee = CONFIG.with_borrow(|config| config.get().transfer_fee.clone());
    Ok(TransactionWrapper(Transaction::approve(
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

fn map_tx_mint(tx: TxInfo, now: u64) -> Result<TransactionWrapper, TransferError> {
    Ok(TransactionWrapper(Transaction::mint(
        Mint {
            amount: tx.amount,
            to: tx.to.expect("Bug: failed to forward mint receiver"),
            memo: tx.memo,
            created_at_time: tx.created_at_time,
        },
        now,
    )))
}

fn map_tx_burnt(tx: TxInfo, now: u64) -> Result<TransactionWrapper, TransferError> {
    let transfer_fee = CONFIG.with_borrow(|config| config.get().transfer_fee.clone());
    if tx.amount < transfer_fee {
        return Err(TransferError::BadBurn {
            min_burn_amount: transfer_fee.clone(),
        });
    }
    let balance = get_balance(tx.from);
    if balance < tx.amount.clone() + transfer_fee {
        return Err(TransferError::InsufficientFunds { balance });
    }
    Ok(TransactionWrapper(Transaction::burn(
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

fn map_tx_transfer(tx: TxInfo, now: u64) -> Result<TransactionWrapper, TransferError> {
    let transfer_fee = CONFIG.with_borrow(|config| config.get().transfer_fee.clone());
    let balance = get_balance(tx.from);
    if balance < tx.amount.clone() + transfer_fee.clone() {
        return Err(TransferError::InsufficientFunds { balance });
    }
    Ok(TransactionWrapper(Transaction::transfer(
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
fn map_tx(tx: TxInfo, now: u64) -> Result<TransactionWrapper, TransferError> {
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

/// Runs validity checks and records the transaction if it is valid
fn apply_tx(tx: TxInfo) -> Result<BlockIndex, TransferError> {
    validate_memo(tx.memo.as_ref())?;
    let now = timestamp();
    validate_created_at_time(tx.created_at_time, now)?;
    let transaction = map_tx(tx, now)?;
    Ok(record_valid_transaction(&transaction))
}

#[update]
fn create_token(args: CreateTokenArgs) -> Result<String, String> {
    if token_created() {
        return Err("Token already created".to_string());
    }

    let minting_account = Account {
        owner: ic_cdk::api::caller(),
        subaccount: None,
    };
    let init_tx = TransactionWrapper(Transaction::mint(
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
                transfer_fee: 10_000_usize.into(),
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
    if !token_created() {
        return Err("Token not created".to_string());
    };

    if ic_cdk::api::caller()
        != CONFIG.with_borrow(|config| config.get().minting_account.clone().unwrap().owner)
    {
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

#[update]
fn icrc1_transfer(arg: TransferArg) -> Result<BlockIndex, TransferError> {
    let from = Account {
        owner: ic_cdk::api::caller(),
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
    get_balance(account)
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
        owner: ic_cdk::api::caller(),
        subaccount: arg.from_subaccount,
    };
    let now = ic_cdk::api::time();
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
    if ic_cdk::api::caller() == arg.from.owner {
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
        owner: ic_cdk::api::caller(),
        subaccount: arg.spender_subaccount,
    };
    let now = ic_cdk::api::time();
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
    let now = ic_cdk::api::time();
    allowance(arg.account, arg.spender, now)
}
