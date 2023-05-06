use dex::*;
use money::Coin;
use tuxedo_core::{verifier::TestVerifier, dynamic_typing::DynamicallyTypedData, types::Output, SimpleConstraintChecker, ConstraintChecker};

#[test]
fn error_enum_has_right_variants() {
    use DexError::*;
    fn _match_outer_constraint_checker(e: DexError) {
        match e {
            TypeError => (),
            OrderMissing => (),
            TooManyOutputsWhenMakingOrder => (),
            NotEnoughCollateralToOpenOrder => (),
            OrderAndPayoutCountDiffer => (),
            PayoutDoesNotSatisfyOrder => (),
            InsufficientTokenAForMatch => (),
            InsufficientTokenBForMatch => (),
            VerifierMismatchForTrade => (),
        }
    }
}
struct TestConfig;
impl DexConfig for TestConfig {
    type Verifier = TestVerifier;
    type A = Coin<0>;
    type B = Coin<1>;
}

type TestOrder = Order<TestConfig>;
type ReverseTestOrder = Order<OppositeSide<TestConfig>>;
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

fn b_for_a_order(offer_amount: u128, ask_amount: u128) -> ReverseTestOrder {
    Order {
        offer_amount,
        ask_amount,
        payout_verifier: TestVerifier { verifies: true },
        _ph_data: Default::default(),
    }
}

fn output_from<T: Into<DynamicallyTypedData>>(payload: T) -> Output<TestVerifier> {
    Output {
        payload: payload.into(),
        verifier: TestVerifier { verifies: true },
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

#[test]
fn matching_two_orders_together_works() {
    let order_a = a_for_b_order(100, 150);
    let order_b = b_for_a_order(150, 100);

    let payout_a = Coin::<1>(150);
    let payout_b = Coin::<0>(100);

    let result = <MatchTestOrders as ConstraintChecker<TestVerifier>>::check(
        &Default::default(),
        &vec![output_from(order_a), output_from(order_b)],
        &vec![output_from(payout_a), output_from(payout_b)],
    );
    assert_eq!(result, Ok(0));
}

#[test]
fn bad_match_insufficient_payout() {
    let order_a = a_for_b_order(100, 150);
    let order_b = b_for_a_order(100, 100);

    // Order a was asking for 150B. But the payout is for only 100B.
    let payout_a = Coin::<1>(100);
    let payout_b = Coin::<0>(100);

    let result = <MatchTestOrders as ConstraintChecker<TestVerifier>>::check(
        &Default::default(),
        &vec![output_from(order_a), output_from(order_b)],
        &vec![output_from(payout_a), output_from(payout_b)],
    );
    assert_eq!(result, Err(DexError::PayoutDoesNotSatisfyOrder));
}

#[test]
fn bad_match_payout_in_wrong_asset() {
    let order_a = a_for_b_order(100, 150);
    let order_b = b_for_a_order(100, 100);

    // Order a was asking for 150B. But the payout is for 100A.
    let payout_a = Coin::<0>(100);
    let payout_b = Coin::<0>(100);

    let result = <MatchTestOrders as ConstraintChecker<TestVerifier>>::check(
        &Default::default(),
        &vec![output_from(order_a), output_from(order_b)],
        &vec![output_from(payout_a), output_from(payout_b)],
    );
    assert_eq!(result, Err(DexError::TypeError));
}

#[test]
fn bad_match_not_enough_a() {
    let order_a = a_for_b_order(90, 150);
    let order_b = b_for_a_order(150, 100);

    let payout_a = Coin::<1>(150);
    let payout_b = Coin::<0>(100);

    let result = <MatchTestOrders as ConstraintChecker<TestVerifier>>::check(
        &Default::default(),
        &vec![output_from(order_a), output_from(order_b)],
        &vec![output_from(payout_a), output_from(payout_b)],
    );
    assert_eq!(result, Err(DexError::InsufficientTokenAForMatch));
}

#[test]
fn bad_match_not_enough_b() {
    let order_a = a_for_b_order(100, 150);
    let order_b = b_for_a_order(100, 100);

    let payout_a = Coin::<1>(150);
    let payout_b = Coin::<0>(100);

    let result = <MatchTestOrders as ConstraintChecker<TestVerifier>>::check(
        &Default::default(),
        &vec![output_from(order_a), output_from(order_b)],
        &vec![output_from(payout_a), output_from(payout_b)],
    );
    assert_eq!(result, Err(DexError::InsufficientTokenBForMatch));
}

#[test]
fn match_with_different_numbers_of_payouts_and_orders() {
    let order_a = a_for_b_order(90, 150);
    let order_b = b_for_a_order(150, 100);

    let payout_a = Coin::<1>(150);

    let result = <MatchTestOrders as ConstraintChecker<TestVerifier>>::check(
        &Default::default(),
        &vec![output_from(order_a), output_from(order_b)],
        &vec![output_from(payout_a)],
    );
    assert_eq!(result, Err(DexError::OrderAndPayoutCountDiffer));
}

#[test]
fn wrong_verifier_on_match_payout() {
    let order_a = a_for_b_order(90, 150);
    let order_b = b_for_a_order(150, 100);

    let payout_a = Coin::<1>(150);
    let payout_b = Coin::<0>(100);

    // We don't use the helper function to construct the full output
    // because we want to make sure the verifier does NOT match
    let payout_b_output = Output {
        payload: payout_b.into(),
        verifier: TestVerifier { verifies: false },
    };

    let result = <MatchTestOrders as ConstraintChecker<TestVerifier>>::check(
        &Default::default(),
        &vec![output_from(order_a), output_from(order_b)],
        &vec![output_from(payout_a), payout_b_output],
    );
    assert_eq!(result, Err(DexError::VerifierMismatchForTrade));
}