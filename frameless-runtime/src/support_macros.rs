//! These macros are defined in frame support, but they are useful more broadly than just frame.
//! TODO PR Substrate so that they live somewhere better so we can benefit from them.
//! Is there a runtime support crate? OR maybe sp runtime?

/// Return Err of the expression: `return Err($expression);`.
///
/// Used as `fail!(expression)`.
#[macro_export]
macro_rules! fail {
	( $y:expr ) => {{
		return Err($y.into())
	}};
}

#[macro_export]
macro_rules! ensure {
	( $x:expr, $y:expr $(,)? ) => {{
		if !$x {
			fail!($y);
		}
	}};
}