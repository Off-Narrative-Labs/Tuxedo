use dex::*;
use tuxedo_core::{dynamic_typing::UtxoData, verifier::TestVerifier};

#[test]
fn order_type_has_right_fields() {
    Order::<TestVerifier> {
        ask_amount: 1,
        offer_amount: 1,
        payout_verifier: TestVerifier{ verifies: true },
    };
}

#[test]
fn order_implements_utxo_data() {
    let id = <Order::<TestVerifier> as UtxoData>::TYPE_ID;
    assert_eq!(id, *b"ordr");
}