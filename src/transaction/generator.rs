use super::*;

pub struct Simple {
    client_id: u16,
    count: u32,
}

impl Simple {
    pub fn new(client_id: u16) -> Self {
        let count = 0;
        Simple { client_id, count }
    }
}

impl Iterator for Simple {
    type Item = Transaction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == u32::MAX {
            None
        } else {
            self.count += 1;
            Some(Transaction::new(
                TransactionType::Deposit,
                self.client_id,
                self.count,
                Some(Decimal::new(1, 0)),
            ))
        }
    }
}
