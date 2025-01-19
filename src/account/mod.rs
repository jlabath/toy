use crate::error::Error;
use crate::transaction::{Transaction, TransactionType};
use rust_decimal::Decimal;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::collections::HashMap;

/// Represents the client account.
///
/// Uses HashMap to store associated transactions.
pub struct Account {
    pub client_id: u16,
    available: Decimal,
    held: Decimal,
    locked: bool,
    tx_storage: HashMap<u32, Transaction>,
}

impl Account {
    /// Constructs new account with the given client_id.
    pub fn new(client_id: u16) -> Self {
        Account {
            client_id,
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            locked: false,
            tx_storage: HashMap::new(),
        }
    }

    /// We do not store total as a separate field, but compute it on demand.
    pub fn total(&self) -> Decimal {
        self.available + self.held
    }

    /// This is the heart of the Account logic. Each outside action to Account is done via this call.
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<(), Error> {
        //sanity checks
        //client id check
        if self.client_id != tx.client_id {
            return Err(Error::InvalidClientId);
        }
        //nothing to be done with locked accounts
        if self.locked {
            return Err(Error::AccountLocked);
        }
        match tx.typ {
            TransactionType::Deposit => {
                self.check_for_duplicate(&tx)?;
                self.available += tx.get_amount()?;
            }
            TransactionType::Withdrawal => {
                self.check_for_duplicate(&tx)?;
                self.withdraw_from_available(tx.get_amount()?)?;
            }
            TransactionType::Dispute => self.dispute(&tx)?,
            TransactionType::Resolve => self.resolve(&tx)?,
            TransactionType::Chargeback => self.chargeback(&tx)?,
        };
        if tx.is_deposit_or_withdrawal() {
            self.tx_storage.insert(tx.transaction_id, tx);
        }
        Ok(())
    }

    /// Checks if such Tx already exists, in transaction storage.
    fn check_for_duplicate(&self, tx: &Transaction) -> Result<(), Error> {
        if self.tx_storage.contains_key(&tx.transaction_id) {
            Err(Error::TxDuplicate)
        } else {
            Ok(())
        }
    }

    /// Safe errorable withdrawal.
    fn withdraw_from_available(&mut self, amount: Decimal) -> Result<(), Error> {
        if self.available < amount {
            return Err(Error::InsufficientFunds);
        }
        self.available -= amount;
        Ok(())
    }

    fn dispute(&mut self, tx: &Transaction) -> Result<(), Error> {
        let old_tx = self
            .tx_storage
            .get_mut(&tx.transaction_id)
            .ok_or(Error::TxNonExistant)?;
        //attempt to set state on tx
        old_tx.dispute()?;
        let amount = old_tx.get_amount()?;
        match old_tx.typ {
            TransactionType::Deposit => {
                //decrease available
                self.withdraw_from_available(amount)?;
                //increase hold
                self.held += amount;
            }
            TransactionType::Withdrawal => {
                //only increase hold
                self.held += amount;
            }
            _ => return Err(Error::Other), //should never happen but just in case
        }
        Ok(())
    }

    fn resolve(&mut self, tx: &Transaction) -> Result<(), Error> {
        let old_tx = self
            .tx_storage
            .get_mut(&tx.transaction_id)
            .ok_or(Error::TxNonExistant)?;
        //attempt to set state on tx
        old_tx.resolve()?;
        let amount = old_tx.get_amount()?;
        match old_tx.typ {
            TransactionType::Deposit => {
                //encrease available
                self.available += amount;
                //decrease hold
                self.held -= amount;
            }
            TransactionType::Withdrawal => {
                //only decrease hold
                self.held -= amount;
            }
            _ => return Err(Error::Other), //should never happen but just in case
        }
        Ok(())
    }

    fn chargeback(&mut self, tx: &Transaction) -> Result<(), Error> {
        let old_tx = self
            .tx_storage
            .get_mut(&tx.transaction_id)
            .ok_or(Error::TxNonExistant)?;
        //attempt to set chargeback on tx
        old_tx.chargeback()?;
        match old_tx.typ {
            TransactionType::Deposit => {
                self.held -= old_tx.get_amount()?;
                self.locked = true;
            }
            TransactionType::Withdrawal => {
                self.held -= old_tx.get_amount()?;
                self.available += old_tx.get_amount()?;
                self.locked = true;
            }
            _ => return Err(Error::Other), //should never happen but just in case
        }
        Ok(())
    }
}

impl Serialize for Account {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let available = self.available.round_dp(4).normalize();
        let held = self.held.round_dp(4).normalize();
        let total = self.total().round_dp(4).normalize();

        let mut state = serializer.serialize_struct("Account", 5)?;
        state.serialize_field("client", &self.client_id)?;
        state.serialize_field("available", &available)?;
        state.serialize_field("held", &held)?;
        state.serialize_field("total", &total)?;
        state.serialize_field("locked", &self.locked)?;
        state.end()
    }
}

#[cfg(test)]
mod tests;
