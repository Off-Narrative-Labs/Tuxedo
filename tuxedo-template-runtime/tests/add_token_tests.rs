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