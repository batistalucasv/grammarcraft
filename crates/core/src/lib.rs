//! Domain types for grammarcraft.
//!
//! Layer 0. This crate knows nothing about files, the terminal, JSON or a UI:
//! it holds the data and the invariants that must stay true whatever calls it.
//! Keeping it dependency-light is what makes the invariants testable in
//! isolation and keeps the dependency direction one-way.
//!
//! The crate never panics. Every failure is a [`CoreError`] returned through
//! [`CoreResult`], and input is treated as hostile (lengths are bounded before
//! anything is allocated).
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

mod doc;
mod error;

pub use doc::{Document, MAX_NOTE_BYTES, MAX_NOTES, Note, NoteId};
pub use error::{CoreError, CoreResult};

#[cfg(test)]
mod tests;
