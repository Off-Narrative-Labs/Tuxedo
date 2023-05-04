//TODO write test functions to wrap the trybuild tests

#[test]
fn has_three_variants() {
    let t = trybuild::TestCases::new();
    t.pass("tests/add_token_tests/has_three_variants.rs");
}