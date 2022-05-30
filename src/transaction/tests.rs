use super::*;

#[test]
fn state_changes() {
    let mut tx = Transaction::new(TransactionType::Deposit, 1, 1, Some(Decimal::new(10, 0)));
    assert!(tx.dispute().is_ok());
    assert!(matches!(tx.dispute().err(), Some(Error::TxAlreadyDisputed)));
    assert!(tx.resolve().is_ok());
    assert!(matches!(tx.resolve().err(), Some(Error::TxNotDisputed)));
    assert!(tx.dispute().is_ok());
    assert!(tx.chargeback().is_ok());
    assert!(matches!(
        tx.chargeback().err(),
        Some(Error::TxAlreadyChargedBack)
    ));
    assert!(matches!(
        tx.resolve().err(),
        Some(Error::TxAlreadyChargedBack)
    ));
    assert!(matches!(
        tx.dispute().err(),
        Some(Error::TxAlreadyChargedBack)
    ));
}
