// https://github.com/dfinity/ICRC-1/blob/main/standards/ICRC-1/README.md
// https://github.com/dfinity/examples/blob/master/rust/tokenmania/backend/types.rs

use std::cell::RefCell;

use candid::{CandidType, Nat};
use ic_stable_structures::{storable::Bound, BTreeMap, Cell, Storable};
use icrc_ledger_types::{
    icrc1::{
        account::Account,
        transfer::{Memo, TransferError},
    },
    icrc2::{approve::ApproveError, transfer_from::TransferFromError},
    icrc3::transactions::Transaction,
};
use serde::{Deserialize, Serialize};

use crate::memory::VMemory;

pub type ConfigRefCell = RefCell<Cell<Configuration, VMemory>>;
pub type TransactionLog = ic_stable_structures::Vec<StorableTransaction, VMemory>;
pub type TransactionLogRefCell = RefCell<TransactionLog>;
pub type AccountBalanceRefCell = RefCell<BTreeMap<Account, StorableToken, VMemory>>;
pub type Tokens = Nat;

#[derive(Debug)]
pub struct TxInfo {
    pub from: Account,
    pub to: Option<Account>,
    pub amount: Tokens,
    pub spender: Option<Account>,
    pub memo: Option<Memo>,
    pub fee: Option<Tokens>,
    pub created_at_time: Option<u64>,
    pub expected_allowance: Option<Tokens>,
    pub expires_at: Option<u64>,
    pub is_approval: bool,
}

#[derive(Debug, Default, CandidType, Deserialize, Serialize)]
pub struct Configuration {
    pub token_name: String,
    pub token_symbol: String,
    pub token_logo: String,
    pub transfer_fee: Tokens,
    pub decimals: u8,
    pub minting_account: Option<Account>,
    pub token_created: bool,
}

impl Storable for Configuration {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut encoded = Vec::new();
        ciborium::into_writer(self, &mut encoded).unwrap();
        std::borrow::Cow::Owned(encoded)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        ciborium::from_reader(bytes.as_ref()).unwrap()
    }
    const BOUND: Bound = Bound::Unbounded;
}

#[derive(Debug, CandidType, Deserialize, Serialize)]
pub struct StorableTransaction(pub Transaction);

impl Storable for StorableTransaction {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut encoded = Vec::new();
        ciborium::into_writer(&self.0, &mut encoded).unwrap();
        std::borrow::Cow::Owned(encoded)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self(ciborium::from_reader(bytes.as_ref()).unwrap())
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 1000,
        is_fixed_size: false,
    };
}

#[derive(Debug, CandidType, Deserialize, Serialize)]
pub struct StorableToken(pub Tokens);

impl Storable for StorableToken {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut encoded = Vec::new();
        ciborium::into_writer(&self.0, &mut encoded).unwrap();
        std::borrow::Cow::Owned(encoded)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self(ciborium::from_reader(bytes.as_ref()).unwrap())
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SupportedStandard {
    pub name: String,
    pub url: String,
}

#[derive(Debug, CandidType, Deserialize, Clone)]
pub struct CreateTokenArgs {
    pub token_name: String,
    pub token_symbol: String,
    pub initial_supply: Nat,
    pub token_logo: String,
}

/// ICRC-2 standard approval error
pub fn to_approve_error(err: TransferError) -> ApproveError {
    match err {
        TransferError::BadFee { expected_fee } => ApproveError::BadFee { expected_fee },
        TransferError::TooOld => ApproveError::TooOld,
        TransferError::CreatedInFuture { ledger_time } => {
            ApproveError::CreatedInFuture { ledger_time }
        }
        TransferError::TemporarilyUnavailable => ApproveError::TemporarilyUnavailable,
        TransferError::Duplicate { duplicate_of } => ApproveError::Duplicate { duplicate_of },
        TransferError::GenericError {
            error_code,
            message,
        } => ApproveError::GenericError {
            error_code,
            message,
        },
        TransferError::BadBurn { .. } | TransferError::InsufficientFunds { .. } => {
            ic_cdk::trap("Bug: cannot transform TransferError into ApproveError")
        }
    }
}

/// ICRC-2 standard transferFrom error
pub fn to_transfer_from_error(err: TransferError) -> TransferFromError {
    match err {
        TransferError::BadFee { expected_fee } => TransferFromError::BadFee { expected_fee },
        TransferError::TooOld => TransferFromError::TooOld,
        TransferError::CreatedInFuture { ledger_time } => {
            TransferFromError::CreatedInFuture { ledger_time }
        }
        TransferError::TemporarilyUnavailable => TransferFromError::TemporarilyUnavailable,
        TransferError::Duplicate { duplicate_of } => TransferFromError::Duplicate { duplicate_of },
        TransferError::GenericError {
            error_code,
            message,
        } => TransferFromError::GenericError {
            error_code,
            message,
        },
        TransferError::InsufficientFunds { balance } => {
            TransferFromError::InsufficientFunds { balance }
        }
        TransferError::BadBurn { min_burn_amount } => {
            TransferFromError::BadBurn { min_burn_amount }
        }
    }
}
