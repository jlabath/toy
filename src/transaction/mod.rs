use crate::error::Error;
use rust_decimal::Decimal;
use serde::Deserialize;

/// Transaction represents the activity on the account.
#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub typ: TransactionType,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub transaction_id: u32,
    amount: Option<Decimal>,
    #[serde(skip)]
    state: TransactionState,
}

impl Transaction {
    #[cfg(test)]
    pub fn new(
        typ: TransactionType,
        client_id: u16,
        transaction_id: u32,
        amount: Option<Decimal>,
    ) -> Self {
        let state = TransactionState::OK;
        Transaction {
            typ,
            client_id,
            transaction_id,
            amount,
            state,
        }
    }

    /// Safe access to amount.
    /// None amounts will return 0.
    ///
    /// If the amount is negative it will error with TxNegativeAmount,
    /// as at present we do not support transactions with negative numbers.
    pub fn get_amount(&self) -> Result<Decimal, Error> {
        //safe way to obtain amount from a transaction
        let amount = self.amount.unwrap_or(Decimal::ZERO);
        if amount < Decimal::ZERO {
            return Err(Error::TxNegativeAmount);
        }
        Ok(amount)
    }

    /// Returns true if the transaction is one of the two types.
    pub fn is_deposit_or_withdrawal(&self) -> bool {
        match self.typ {
            TransactionType::Deposit => true,
            TransactionType::Withdrawal => true,
            _ => false,
        }
    }

    //state machine functions

    /// Moves the transaction to Disputed state or errors.
    pub fn dispute(&mut self) -> Result<(), Error> {
        match self.state {
            TransactionState::Disputed => Err(Error::TxAlreadyDisputed),
            TransactionState::Chargedback => Err(Error::TxAlreadyChargedBack),
            TransactionState::OK => {
                self.state = TransactionState::Disputed;
                Ok(())
            }
        }
    }

    /// Moves the transaction from Disputed back to OK state or errors.
    pub fn resolve(&mut self) -> Result<(), Error> {
        match self.state {
            TransactionState::Disputed => {
                self.state = TransactionState::OK;
                Ok(())
            }
            TransactionState::Chargedback => Err(Error::TxAlreadyChargedBack),
            TransactionState::OK => Err(Error::TxNotDisputed),
        }
    }

    /// Moves the transaction from Disputed to Chargedback state or errors.
    pub fn chargeback(&mut self) -> Result<(), Error> {
        match self.state {
            TransactionState::Disputed => {
                self.state = TransactionState::Chargedback;
                Ok(())
            }
            TransactionState::Chargedback => Err(Error::TxAlreadyChargedBack),
            TransactionState::OK => Err(Error::TxNotDisputed),
        }
    }
}

/// Supported transaction types.
#[derive(Debug, Deserialize)]
pub enum TransactionType {
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "withdrawal")]
    Withdrawal,
    #[serde(rename = "dispute")]
    Dispute,
    #[serde(rename = "resolve")]
    Resolve,
    #[serde(rename = "chargeback")]
    Chargeback,
}

/// TransactionState is used by Account to keep track of transaction disputes.
#[derive(Debug, Deserialize)]
pub enum TransactionState {
    ///Initial "normal" state it can move to Disputed.
    OK,
    ///Transaction is being disputed this can move the Chargedback or back to OK (upon resolve).
    Disputed,
    ///Final state when the transaction was charged back.
    Chargedback,
}

impl Default for TransactionState {
    fn default() -> Self {
        Self::OK
    }
}

#[cfg(test)]
pub mod generator;
#[cfg(test)]
mod tests;
