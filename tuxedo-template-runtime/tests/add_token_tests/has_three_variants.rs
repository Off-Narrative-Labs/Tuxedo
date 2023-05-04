use tuxedo_template_runtime::OuterConstraintChecker::{self, *};

fn match_outer_constraint_checker(c: OuterConstraintChecker) {
    match c {
        Money(_) => (),
        RuntimeUpgrade(_) => (),
        SecondToken(_) => (),
    }
}