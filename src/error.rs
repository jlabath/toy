use std::io;
use thiserror::Error;
use tokio::sync::mpsc;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid client id for a transaction")]
    InvalidClientId,
    #[error("account locked")]
    AccountLocked,
    #[error("insufficient funds in the account")]
    InsufficientFunds,
    #[error("no such transaction exists in the account")]
    TxNonExistant,
    #[error("at present we do not support transactions with negative amounts")]
    TxNegativeAmount,
    #[error("this transaction is already disputed")]
    TxAlreadyDisputed,
    #[error("this transaction is not disputed")]
    TxNotDisputed,
    #[error("this transaction is already charged back")]
    TxAlreadyChargedBack,
    #[error("duplicate transaction id")]
    TxDuplicate,
    #[error("other error")]
    Other,
    #[error("IO")]
    IO(#[from] io::Error),
    #[error("csv")]
    Csv(#[from] csv::Error),
    #[error("{0}")]
    Custom(&'static str),
    #[error("mpsc::sender")]
    Sender(#[from] mpsc::error::SendError<crate::transaction::Transaction>),
}
