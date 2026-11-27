//! The engine: a [`Session`], its command registry and the error type every
//! command returns.
//!
//! Layering: this crate is layer 1 and may depend on layer 0 (`grammarcraft-core`)
//! only. It does not know whether the caller is a terminal, a UI or an agent —
//! that is the point of routing every action through a command id.
//!
//! Everything user-visible is a [`CommandSpec`]. A front end never reaches into
//! the document directly; it builds JSON parameters and calls
//! [`Session::run`]. That single entry point is where parameters are validated,
//! which is why the hostile-parameter test below can cover every command at
//! once.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

mod cmd;
mod error;

pub use cmd::{CommandSpec, Registry, registry};
pub use error::{CmdError, CmdResult};

use grammarcraft_core::{Document, WritingGoals};
use serde_json::Value;

/// One editing session: the document, writing goals and state that outlives a command.
#[derive(Debug, Default)]
pub struct Session {
    doc: Document,
    goals: WritingGoals,
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn doc(&self) -> &Document {
        &self.doc
    }

    pub fn goals(&self) -> &WritingGoals {
        &self.goals
    }

    /// Runs a command by id.
    ///
    /// Unknown ids, disabled commands and bad parameters are errors, never
    /// panics: the id and the parameters can both come from a file, a socket
    /// or an agent.
    pub fn run(&mut self, id: &str, params: &Value) -> CmdResult {
        let spec = registry()
            .get(id)
            .ok_or_else(|| CmdError::UnknownCommand(id.to_owned()))?;
        if let Some(reason) = (spec.enabled)(self) {
            return Err(CmdError::Disabled(reason.to_owned()));
        }
        (spec.run)(self, params)
    }

    fn doc_mut(&mut self) -> &mut Document {
        &mut self.doc
    }

    fn goals_mut(&mut self) -> &mut WritingGoals {
        &mut self.goals
    }
}

#[cfg(test)]
mod tests;
