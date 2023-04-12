//! An Order-book based dex to swap between two hard-coded tokens A and B.
//! 
//! For simplicity, we don't allow partial fills right now.

use sp_arithmetic::per_things::Percent;
use tuxedo_core::dynamic_typing::UtxoData;

/// All the data that this piece could care to store. Here I'm choosing to use a
/// single enum to experiment with some stronger typing.
enum DexItem {
    /// A coin of token A
    TokenA(u128),
    /// A coin of token B
    TokenB(u128),
    /// An order in the order book
    Order(Order),
}

impl UtxoData for DexItem {
    const TYPE_ID: [u8; 4] = *b"$dex";
}

/// Which side of a trade the order maker is on
enum Side {
    /// The order maker wants to obtain more or token A (by selling some of token B)
    SeekingTokenA,
    /// The order maker wants to obtain more of token B (by selling some of token A)
    SeekingTokenB,
}

/// An order in the book consists of amounts of each token, A and B, as well as which side
/// of the trade the order maker is on.
struct Order {
    /// The amount of token A in this order
    token_a: u128,
    /// The amount of token B in this order
    token_b: u128,
    /// Which side of the trade this order maker is on
    side: Side,
}

/// The Constraint checking logic for the Dex. We are taking the approach of a single
/// constraint checker with multiple variants.
enum DexConstraitChecker {
    /// Open a new order in the order book
    MakeOrder,
    /// Match existing open orders against one another
    MatchOrders,
    /// Fullfil existing orders in the order book with the supplied funds.
    /// This is an atomic combination of making and order and matching it with
    /// an existing order.
    TakeOrders,
}

