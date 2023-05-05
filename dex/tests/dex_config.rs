use dex::*;
use money::Coin;
use tuxedo_core::{SimpleConstraintChecker, verifier::TestVerifier};

struct TestConfig;
impl DexConfig for TestConfig {
    type Verifier = TestVerifier;
    type A = Coin<0>;
    type B = Coin<1>;
}

type TestOrder = Order<TestConfig>;
type ReverseTestOrder = Order<ReverseConfig<TestConfig>>;
type MakeTestOrder = MakeOrder<TestConfig>;
type MatchTestOrders = MatchOrders<TestConfig>;

fn a_for_b_order(offer_amount: u128, ask_amount: u128) -> TestOrder {
    Order {
        offer_amount,
        ask_amount,
        payout_verifier: TestVerifier { verifies: true },
        _ph_data: Default::default(),
    }
}

#[test]
fn opening_an_order_seeking_a_works() {
    let order = a_for_b_order(100, 150);
    let input = Coin::<0>(100);

    let result = <MakeTestOrder as SimpleConstraintChecker>::check(
        &Default::default(),
        &vec![input.into()],
        &vec![order.into()],
    );
    assert!(result.is_ok());
}

#[test]
fn opening_order_with_no_inputs_fails() {
    let order = a_for_b_order(100, 150);

    let result = <MakeTestOrder as SimpleConstraintChecker>::check(
        &Default::default(),
        &vec![],
        &vec![order.into()],
    );
    assert_eq!(result, Err(DexError::NotEnoughCollateralToOpenOrder));
}

#[test]
fn opening_order_with_no_outputs_fails() {
    let input = Coin::<0>(100);

    let result = <MakeTestOrder as SimpleConstraintChecker>::check(
        &Default::default(),
        &vec![input.into()],
        &vec![],
    );
    assert_eq!(result, Err(DexError::OrderMissing));
}

#[test]
fn opening_order_with_insufficient_collateral_fails() {
    // Collateral is only worth 50, but order is for 100
    let input = Coin::<0>(50);
    let order = a_for_b_order(100, 150);

    let result = <MakeTestOrder as SimpleConstraintChecker>::check(
        &Default::default(),
        &vec![input.into()],
        &vec![order.into()],
    );

    assert_eq!(result, Err(DexError::NotEnoughCollateralToOpenOrder));
}

#[test]
fn opening_order_with_collateral_in_wrong_asset() {
    // Collateral is in Token B but order offered token A
    let input = Coin::<1>(100);
    let order = a_for_b_order(100, 150);

    let result = <MakeTestOrder as SimpleConstraintChecker>::check(
        &Default::default(),
        &vec![input.into()],
        &vec![order.into()],
    );

    assert_eq!(result, Err(DexError::TypeError));
}