# Add a Token

Before we get to the dex logic itself, we will need some tokens to exchange, and this runtime only has one token at the moment.
Your task for this part is to extend the runtime's `OuterConstraintChecker` to have a third variant called `SecondToken` that uses token id 1.

When you believe you have completed this section, run `cargo test --test add_token`.
