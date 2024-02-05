//! Crate prelude

pub use crate::error::TmuxError;

pub type Result<T> = std::result::Result<T, TmuxError>;

// Generic wrapper tuple struct for newtype pattern
pub struct W<T>(pub T);

// Personal prefenrences
pub use std::format as f;
