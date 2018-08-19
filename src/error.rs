//! Consistent definitions for realize error types.
use failure;
use std::result;

/// A result whose error is `Error`.
pub type Result<A> = result::Result<A, Error>;

/// An error originating from a realize operation.
pub type Error = failure::Error;
