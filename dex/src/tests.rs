//! Unit tests for the Dex piece

use super::*;
use tuxedo_core::dynamic_typing::testing::Bogus;

/// An simple dex config to use in unit tests.
struct TestConfig;
impl DexConfig for TestConfig {
    type Verifier = TestVerifier;
    type A = Coin<0>;
    type B = Coin<1>;
}

/// A concrete `Order` type. It uses the test config above.
type TestOrder = Order<TestConfig>;

/// A concrete `MakeOrder` constraint checker. It uses the test config above.
type MakeTestOrder = MakeOrder<TestConfig>;

