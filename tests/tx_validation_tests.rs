use rusty_chain::core::mempool::Mempool;
use rusty_chain::core::types::Transaction;

#[test]
fn validate_basic_accepts_ok_tx() {
    let tx = Transaction {
        from: "alice".into(),
        to: "bob".into(),
        amount: 1,
        nonce: 0,
    };

    tx.validate_basic().unwrap();
}

#[test]
fn validate_basic_rejects_empty_from() {
    let tx = Transaction {
        from: "".into(),
        to: "bob".into(),
        amount: 1,
        nonce: 0,
    };

    let err = tx.validate_basic().unwrap_err().to_string();
    assert!(err.contains("from"), "unexpected error: {err}");
}

#[test]
fn validate_basic_rejects_zero_amount() {
    let tx = Transaction {
        from: "alice".into(),
        to: "bob".into(),
        amount: 0,
        nonce: 0,
    };

    let err = tx.validate_basic().unwrap_err().to_string();
    assert!(err.contains("amount"), "unexpected error: {err}");
}

#[test]
fn mempool_add_rejects_invalid_tx() {
    let mut mp = Mempool::default();
    let tx = Transaction {
        from: "alice".into(),
        to: "alice".into(),
        amount: 1,
        nonce: 0,
    };

    let err = mp.add_tx(tx).unwrap_err().to_string();
    assert!(err.contains("must differ"), "unexpected error: {err}");
    assert_eq!(mp.txs.len(), 0);
}
