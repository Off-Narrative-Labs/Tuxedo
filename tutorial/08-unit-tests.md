# Unit Testing the Dex

If you were developing this pallet from your own designs, now would be a great time to stop and write unit tests for your constraint checker.
Because this is a tutorial, many [`MakeOrder` tests](../dex/tests/dex_config.rs) are already written, and you should inspect them carefully.

Although there are several tests provided, we will still discuss Tuxedo tests, and practice adding them.
One very nice thing about writing tests for `ConstraintChecker`s is that they are pure functions.
They do not need to read from state or write to it; all of that is passed in as inputs and outputs.
Those who are familiar with FRAME will recognize that this is a lot less overhead than setting up a mock runtime and building test externalities.

Typically the tests will live in a dedicated file called `tests.rs` in the piece's `src` folder until there are enough that they warrant being split into multiple files.
A [`dex/src/tests.rs`](../dex/src/tests.rs) file already exists in this repository and is mostly empty.
Let's use it to add two new tests to the runtime.

For our first test case, we will ensure that opening a simple order to trade 100 A for 150 B works when the user uses two different input coins to sum to the total collateral.
This test is mostly written for you with just a single thing `todo!()`.
```rust
#[test]
fn summing_two_coins_for_collateral_works() {
    let order = TestOrder {
        offer_amount: 100,
        ask_amount: 150,
        payout_verifier: TestVerifier { verifies: true },
        _ph_data: Default::default(),
    };

    let first_coin = Coin::<0>(40);
    let second_coin = todo!("create a second coin worth 60 units");

    let result = MakeTestOrder::default().check(
        &vec![first_coin.into(), second_coin.into()],
        &vec![order.into()],
    );
    assert!(result.is_ok());
}
```

> In order to run these tests, you may need to move or delete some of the old integration tests for earlier parts of the tutorial.
> One way to do this is `mv dex/tests dex/_tests`.
> Then when you are done with this section, move them back.

You can now run this test with `cargo test -p dex`

For our second test case we want to see what happens when you take a transaction very similar to our previous test, but swap the inputs with the outputs.
That is to say we put the order as the input and the collateral as output.
We can intuit that this test should fail, but what error do you expect?
Complete this test and see if you expected the correct error.

```rust
#[test]
fn making_order_with_inputs_and_outputs_reversed_fails() {
    let order = TestOrder {
        offer_amount: 100,
        ask_amount: 150,
        payout_verifier: TestVerifier { verifies: true },
        _ph_data: Default::default(),
    };

    todo!("make some coins")

    let result = MakeTestOrder::default().check(
        todo!("put the outputs here where the inputs should go"),
        todo!("put the inputs here where the outputs should go"),
    );

    assert_eq!(result, Err(todo!("What error are you expecting")));
}
```

When you believe you have completed this section, run `cargo test -p dex` to run the tests you have just written.

> If you disabled the integration tests in this section, you should re-enable them before moving on to the next section.
