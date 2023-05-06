# Additional Ideas

Congratulations! You have reached the end of the Tuxedo Order Book Dex tutorial.
If you have a project in mind, now is a great time to start building it.
Or you may further your learning by continuing to extend and adapt this Dex piece.

Here are some ideas for how to make the Dex more user friendly or realistic:

## `CancelOrder` Constraint checker

Perhaps a trader opened an order a while ago, and it hasn't filled.
They would now like to close the order and get the collateral back.
Add a constraint checker to allow canceling open orders.

You will need _some_ way to make sure that users cannot cancel each others' orders.
Three ideas come to mind for this
1. Perhaps they need to produce a redeemer that satisfies the `payout_Verifier`.
2. Or perhaps you add another field for the `calcelation_verifier`.
3. When an order is opened, two new UTXOs are created.
  The first is the order, as before.
  The second is a special cancel privledge.
  Then the `CancelOrder` constraint checker will only perform the cancellation if it can also consume the cancel privledge.

## Partial Matching

So far the Dex requires orders to be filled entirely in  order for a match to execute.
It would be better if matches could be partially filled.
For example, consider these orders:

Trader | Offer | Ask
-------|-------|-----
Alice  | 4 A   | 2 B
Bob    | 1 B   | 2 A

Alice and Bob agree on the price and could be counterparties, except that Alice is seeing twice the volume as Bob.
It would be nice if we could match Bob's order with part of Alice's.
After such a partial match the outputs would be
* A payout of 2A to fill Bob's order
* A payout of 1B to partially fill Alice's order
* An order from Alice offering 2A for 1B (the remainder of her order that wasn't filled).

## Change When Opening Orders

Currently the `MakeOrder` constraint checker requires the user to put up the exact amount of collateral required for the order.
In some cases, the user will not have a UTXO for the exact amount.
It would be nicer to allow the user to put up extra collateral, and give an additional output representing change that goes back to themself.

## Matchers Fees

Currently the only incentive users have to match orders is to fill their own trades.
Incentivize a marketplace of order matchers by allowing them to claim any leftovers when matching orders.

For example, consider these orders:

Trader | Offer | Ask
-------|-------|-----
Alice  | 4 A   | 2 B
Bob    | 3 B   | 3 A

If these orders are matched together, Alice and bob can both get their payouts, and there will be 1A and 1B leftover.
The matcher should be able to assign these profits to themself by adding two additional outputs to the transaction.

## Proper Prioritization

Currently all the transactions have the same priority, namely, `0`.
Allow users to specify a tip for the block producer when making orders to get higher priority.
This would work well in conjunction with the previous enhancement about change.
Allow matchers to provide some proceeds as a tip for the block producer when matching orders.
This would work well in conjunction with the previous enhancement about matchers fees.

## Dynamic Tokens and Trading Pairs

The current Dex design is very strongly typed and static.
We require each token and trading pair to be declared at compile time.
Some use cases may require a more dynamic approach where new tokens and trading pairs can be created at any moment by users themselves.
Modify the dex and the money piece (you will need to copy the money piece into this repo to modify it) so that the token ids are fields rather than compile-time constants.

## Many Token Orders and Matches

The dex currently imposes unnecessary structure on the orders and matches.
There is no reason that an order must be between exactly two tokens.
It is perfectly reasonable for a trader to make an offer like "two wheat and one sheep for two brick".
Generalize the piece so that it allows for such orders and matches.
This would work well with the previous enhancement about dynamic tokens and trading pairs.