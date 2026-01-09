#![allow(clippy::option_if_let_else)]
//! Helpers to keep binding entrypoints panic-safe.
//!
//! Binding layers (`PyO3`, NAPI-RS, ext-php-rs, WASM, FFI) must not allow Rust
//! panics to unwind into foreign runtimes. `guard_panic` wraps conversion calls,
//! converts panics into `ConversionError::Panic`, and preserves the original
//! error handling path for the caller.

use std::any::Any;
use std::panic::{self, AssertUnwindSafe, UnwindSafe};

use crate::error::{ConversionError, Result};

/// Run a fallible operation while preventing panics from unwinding across FFI.
///
/// Panics are captured and surfaced as `ConversionError::Panic` so bindings can
/// translate them into language-native errors instead of aborting.
#[allow(clippy::missing_errors_doc)]
pub fn guard_panic<F, T>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T> + UnwindSafe,
{
    if std::env::var("HTML_TO_MARKDOWN_FAST_FFI")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes"))
    {
        return f();
    }

    match panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(payload) => Err(ConversionError::Panic(panic_message(payload))),
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
#[allow(clippy::needless_pass_by_value)]
fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(msg) = payload.downcast_ref::<&str>() {
        (*msg).to_string()
    } else if let Some(msg) = payload.downcast_ref::<String>() {
        msg.clone()
    } else {
        "unexpected panic without message".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_panic_converts_panic_to_error() {
        let err = guard_panic::<_, ()>(|| -> Result<()> {
            panic!("boom");
        })
        .unwrap_err();

        match err {
            ConversionError::Panic(message) => assert_eq!(message, "boom"),
            other => panic!("expected panic error, got {:?}", other),
        }
    }

    #[test]
    fn guard_panic_forwards_ok() {
        let value = guard_panic(|| Ok::<_, ConversionError>(42)).unwrap();
        assert_eq!(value, 42);
    }
}
