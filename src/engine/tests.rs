use super::*;
use rust_decimal::Decimal;

#[tokio::test]
async fn engine_with_multiple_producers() {
    let (handle, ch) = start().await;
    let ch2 = ch.clone();
    let ch3 = ch.clone();
    tokio::spawn(async move {
        let iter = crate::transaction::generator::Simple::new(5).take(100000);
        for tx in iter {
            if let Err(_) = ch.send(tx).await {
                eprintln!("receiver dropped");
                break;
            }
        }
    });
    tokio::spawn(async move {
        let iter = crate::transaction::generator::Simple::new(42).take(100000);
        for tx in iter {
            if let Err(_) = ch2.send(tx).await {
                eprintln!("receiver dropped");
                break;
            }
        }
    });
    tokio::spawn(async move {
        let iter = crate::transaction::generator::Simple::new(771).take(100000);
        for tx in iter {
            if let Err(_) = ch3.send(tx).await {
                eprintln!("receiver dropped");
                break;
            }
        }
    });

    let result = handle.await;
    assert!(result.is_ok());
    let accounts: Vec<Account> = result.unwrap().into_values().collect();
    assert_eq!(accounts.len(), 3);
    for acc in &accounts {
        assert_eq!(acc.total(), Decimal::new(100000, 0));
    }
}
