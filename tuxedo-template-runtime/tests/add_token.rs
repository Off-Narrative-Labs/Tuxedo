use tuxedo_template_runtime::OuterConstraintChecker::{self, *};

#[test]
fn has_three_variants() {
    fn match_outer_constraint_checker(c: OuterConstraintChecker) {
        match c {
            Money(_) => (),
            RuntimeUpgrade(_) => (),
            SecondToken(_) => (),
        }
    }
}

#[test]
fn second_token_is_id_1() {
    let coin_1_checker = money::MoneyConstraintChecker::<1>::Mint;
    let outer_checker = SecondToken(coin_1_checker);
}