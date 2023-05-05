//! These macros are copied from frame-support. Substrate maintainers are not open to putting them in
//! a more sensible location. See https://github.com/paritytech/substrate/issues/13456

// Bring in the no_bound ones that are in another crate because they are proc macros
pub use derive_no_bound::{CloneNoBound, DebugNoBound, DefaultNoBound};

/// Return Err of the expression: `return Err($expression);`.
///
/// Used as `fail!(expression)`.
#[macro_export]
macro_rules! fail {
    ( $y:expr ) => {{
        return Err($y.into());
    }};
}

/// Evaluate `$x:expr` and if not true return `Err($y:expr)`.
///
/// Used as `ensure!(expression_to_ensure, expression_to_return_on_false)`.
#[macro_export]
macro_rules! ensure {
    ( $x:expr, $y:expr $(,)? ) => {{
        if !$x {
            $crate::fail!($y);
        }
    }};
}
