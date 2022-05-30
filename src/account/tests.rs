use super::*;

#[test]
fn basic() {
    let mut acc = Account::new(1);
    acc.available = Decimal::new(3, 0);
    acc.held = Decimal::new(2, 0);
    assert_eq!(Decimal::new(5, 0), acc.total());
    let tx1 = Transaction::new(TransactionType::Deposit, 3, 200, Some(Decimal::new(10, 0)));
    assert!(matches!(
        acc.add_transaction(tx1).err(),
        Some(Error::InvalidClientId)
    ));
}

#[test]
fn addition() {
    let mut acc = Account::new(2);
    let tx1 = Transaction::new(
        TransactionType::Deposit,
        acc.client_id,
        200,
        Some(Decimal::new(10, 0)),
    );
    let tx2 = Transaction::new(
        TransactionType::Withdrawal,
        acc.client_id,
        1,
        Some(Decimal::new(12, 1)),
    );
    assert!(acc.add_transaction(tx1).is_ok());
    assert!(acc.add_transaction(tx2).is_ok());
    assert_eq!(acc.total(), Decimal::new(88, 1));
    let tx3 = Transaction::new(
        TransactionType::Deposit,
        acc.client_id,
        91,
        Some(Decimal::new(12001, 4)),
    );
    assert!(acc.add_transaction(tx3).is_ok());
    assert_eq!(acc.total(), Decimal::new(100001, 4));
    assert_eq!(acc.total().to_string(), "10.0001".to_string());
    let tx4 = Transaction::new(
        TransactionType::Withdrawal,
        acc.client_id,
        73,
        Some(Decimal::new(1, 4)),
    );
    assert!(acc.add_transaction(tx4).is_ok());
    assert_eq!(acc.total(), Decimal::new(10, 0));
    assert_eq!(acc.total().to_string(), "10.0000".to_string());

    //check duplicate
    let tx5 = Transaction::new(
        TransactionType::Deposit,
        acc.client_id,
        73,
        Some(Decimal::new(1, 0)),
    );
    assert!(matches!(
        acc.add_transaction(tx5).err(),
        Some(Error::TxDuplicate)
    ));
    let tx6 = Transaction::new(
        TransactionType::Withdrawal,
        acc.client_id,
        73,
        Some(Decimal::new(1, 0)),
    );
    assert!(matches!(
        acc.add_transaction(tx6).err(),
        Some(Error::TxDuplicate)
    ));
}

#[test]
fn overdraw() {
    let mut acc = Account::new(2);
    let tx1 = Transaction::new(
        TransactionType::Deposit,
        acc.client_id,
        200,
        Some(Decimal::new(10, 0)),
    );
    assert!(acc.add_transaction(tx1).is_ok());
    let tx2 = Transaction::new(
        TransactionType::Withdrawal,
        acc.client_id,
        201,
        Some(Decimal::new(100001, 4)),
    );
    assert!(matches!(
        acc.add_transaction(tx2).err().unwrap(),
        Error::InsufficientFunds
    ));
    let tx3 = Transaction::new(
        TransactionType::Withdrawal,
        acc.client_id,
        202,
        Some(Decimal::new(100000, 4)),
    );
    assert!(acc.add_transaction(tx3).is_ok());
    assert_eq!(acc.total(), Decimal::ZERO);
}

#[test]
fn dispute() {
    let mut acc = Account::new(66);

    let tx1 = Transaction::new(
        TransactionType::Deposit,
        acc.client_id,
        200,
        Some(Decimal::new(10, 0)),
    );
    assert!(acc.add_transaction(tx1).is_ok());
    assert_eq!(Decimal::new(10, 0), acc.total());
    let tx2 = Transaction::new(TransactionType::Dispute, acc.client_id, 200, None);
    assert!(acc.add_transaction(tx2).is_ok());
    assert_eq!(Decimal::ZERO, acc.available);
    assert_eq!(Decimal::new(10, 0), acc.total());

    //fail on non existent dispute
    let tx1 = Transaction::new(TransactionType::Dispute, acc.client_id, 1, None);
    assert!(matches!(
        acc.add_transaction(tx1).err(),
        Some(Error::TxNonExistant)
    ));

    //fail if dispute is repeated
    let tx3 = Transaction::new(TransactionType::Dispute, acc.client_id, 200, None);
    assert!(matches!(
        acc.add_transaction(tx3).err(),
        Some(Error::TxAlreadyDisputed)
    ));
}

#[test]
fn dispute_withdrawal() {
    let mut acc = Account::new(66);

    let tx1 = Transaction::new(
        TransactionType::Deposit,
        acc.client_id,
        200,
        Some(Decimal::new(10, 0)),
    );
    assert!(acc.add_transaction(tx1).is_ok());
    assert_eq!(Decimal::new(10, 0), acc.total());
    let tx2 = Transaction::new(
        TransactionType::Withdrawal,
        acc.client_id,
        201,
        Some(Decimal::new(5, 0)),
    );
    assert!(acc.add_transaction(tx2).is_ok());
    assert_eq!(Decimal::new(5, 0), acc.total());

    let tx3 = Transaction::new(TransactionType::Dispute, acc.client_id, 201, None);
    assert!(acc.add_transaction(tx3).is_ok());
    assert_eq!(Decimal::new(10, 0), acc.total());
    assert_eq!(Decimal::new(5, 0), acc.available);
    assert_eq!(Decimal::new(5, 0), acc.held);
}

#[test]
fn resolve() {
    let mut acc = Account::new(66);

    let tx1 = Transaction::new(
        TransactionType::Deposit,
        acc.client_id,
        200,
        Some(Decimal::new(10, 0)),
    );
    assert!(acc.add_transaction(tx1).is_ok());
    assert_eq!(Decimal::new(10, 0), acc.total());
    let tx2 = Transaction::new(TransactionType::Dispute, acc.client_id, 200, None);
    assert!(acc.add_transaction(tx2).is_ok());
    assert_eq!(Decimal::ZERO, acc.available);
    assert_eq!(Decimal::new(10, 0), acc.total());
    let tx3 = Transaction::new(TransactionType::Resolve, acc.client_id, 200, None);
    assert!(acc.add_transaction(tx3).is_ok());
    assert_eq!(Decimal::ZERO, acc.held);
    assert_eq!(Decimal::new(10, 0), acc.total());
    let tx4 = Transaction::new(TransactionType::Resolve, acc.client_id, 200, None);
    assert!(matches!(
        acc.add_transaction(tx4).err(),
        Some(Error::TxNotDisputed)
    ));
}

#[test]
fn resolve_withdrawal() {
    let mut acc = Account::new(66);

    let tx1 = Transaction::new(
        TransactionType::Deposit,
        acc.client_id,
        200,
        Some(Decimal::new(10, 0)),
    );
    assert!(acc.add_transaction(tx1).is_ok());
    assert_eq!(Decimal::new(10, 0), acc.total());
    let tx2 = Transaction::new(
        TransactionType::Withdrawal,
        acc.client_id,
        201,
        Some(Decimal::new(5, 0)),
    );
    assert!(acc.add_transaction(tx2).is_ok());
    assert_eq!(Decimal::new(5, 0), acc.total());

    let tx3 = Transaction::new(TransactionType::Dispute, acc.client_id, 201, None);
    assert!(acc.add_transaction(tx3).is_ok());
    assert_eq!(Decimal::new(10, 0), acc.total());
    assert_eq!(Decimal::new(5, 0), acc.available);
    assert_eq!(Decimal::new(5, 0), acc.held);
    let tx4 = Transaction::new(TransactionType::Resolve, acc.client_id, 201, None);
    assert!(acc.add_transaction(tx4).is_ok());
    assert_eq!(Decimal::new(5, 0), acc.total());
    assert_eq!(Decimal::new(5, 0), acc.available);
    assert_eq!(Decimal::new(0, 0), acc.held);
}

#[test]
fn chargeback() {
    let mut acc = Account::new(66);
    let tx1 = Transaction::new(
        TransactionType::Deposit,
        acc.client_id,
        200,
        Some(Decimal::new(10, 0)),
    );
    assert!(acc.add_transaction(tx1).is_ok());
    assert_eq!(Decimal::new(10, 0), acc.total());
    let tx2 = Transaction::new(TransactionType::Dispute, acc.client_id, 200, None);
    assert!(acc.add_transaction(tx2).is_ok());
    assert_eq!(Decimal::ZERO, acc.available);
    assert_eq!(Decimal::new(10, 0), acc.total());
    let tx3 = Transaction::new(TransactionType::Chargeback, acc.client_id, 200, None);
    assert!(acc.add_transaction(tx3).is_ok());
    assert_eq!(Decimal::ZERO, acc.held);
    assert_eq!(Decimal::ZERO, acc.total());
    let tx4 = Transaction::new(TransactionType::Resolve, acc.client_id, 200, None);
    assert!(acc.locked);
    assert!(matches!(
        acc.add_transaction(tx4).err(),
        Some(Error::AccountLocked)
    ));
}

#[test]
fn chargeback_withdrawal() {
    let mut acc = Account::new(66);

    let tx1 = Transaction::new(
        TransactionType::Deposit,
        acc.client_id,
        200,
        Some(Decimal::new(10, 0)),
    );
    assert!(acc.add_transaction(tx1).is_ok());
    assert_eq!(Decimal::new(10, 0), acc.total());
    let tx2 = Transaction::new(
        TransactionType::Withdrawal,
        acc.client_id,
        201,
        Some(Decimal::new(5, 0)),
    );
    assert!(acc.add_transaction(tx2).is_ok());
    assert_eq!(Decimal::new(5, 0), acc.total());

    let tx3 = Transaction::new(TransactionType::Dispute, acc.client_id, 201, None);
    assert!(acc.add_transaction(tx3).is_ok());
    assert_eq!(Decimal::new(10, 0), acc.total());
    assert_eq!(Decimal::new(5, 0), acc.available);
    assert_eq!(Decimal::new(5, 0), acc.held);
    let tx4 = Transaction::new(TransactionType::Chargeback, acc.client_id, 201, None);
    assert!(acc.add_transaction(tx4).is_ok());
    assert_eq!(Decimal::new(10, 0), acc.total());
    assert_eq!(Decimal::new(10, 0), acc.available);
    assert_eq!(Decimal::new(0, 0), acc.held);
    //check locked account
    let tx5 = Transaction::new(
        TransactionType::Deposit,
        acc.client_id,
        220,
        Some(Decimal::new(5, 0)),
    );
    assert!(matches!(
        acc.add_transaction(tx5).err(),
        Some(Error::AccountLocked)
    ));
}

#[test]
fn many_transactions() {
    let size: usize = 1000000;
    let mut acc = Account::new(11);
    let iter = crate::transaction::generator::Simple::new(acc.client_id).take(size);
    for tx in iter {
        let res = acc.add_transaction(tx);
        assert!(res.is_ok());
    }
    assert_eq!(acc.total(), Decimal::new(i64::try_from(size).unwrap(), 0));
}
