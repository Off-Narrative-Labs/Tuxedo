use tuxedo_template_runtime::OuterConstraintChecker::{self, *};

#[test]
fn has_five_variants() {
    fn _match_outer_constraint_checker(c: OuterConstraintChecker) {
        match c {
            Money(_) => (),
            RuntimeUpgrade(_) => (),
            SecondToken(_) => (),
            MakeOrder01(_) => (),
            MakeOrder10(_) => (),
        }
    }
}

//TODO is there some way to make sure that the config is the expected one?