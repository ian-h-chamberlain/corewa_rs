//! This module defines the phased_parser state machine. Each phase of the parser
//! is a submodule within this module.

use std::convert::{Infallible, TryFrom};
use std::str::FromStr;

mod clean;
mod deserialize;
mod expansion;

use crate::error::Error;
use crate::load_file;

/// The data type that is passed through the parser phases. This is a simple state
/// machine, which transitions to the next state by passing through a parser phase.
#[derive(Debug)]
pub struct Phase<PhaseState> {
    /// The original input to the parser, which can be used for spans / string views
    pub buffer: String,
    /// State specific to the current phase of the state machine
    pub state: PhaseState,
}

/// The initial state of [Buffer](struct.Buffer.html), before any preprocessing has occurred.
pub struct Raw;

impl FromStr for Phase<Raw> {
    type Err = Infallible;

    fn from_str(buf: &str) -> Result<Self, Infallible> {
        Ok(Phase {
            buffer: buf.to_string(),
            state: Raw,
        })
    }
}

/// The Phase after comments have been removed and metadata parsed from comments.
/// This phase also parses ORG and END, and removes any text after END
#[derive(Debug)]
pub struct Clean {
    lines: Vec<String>,
    metadata: clean::Info,
}

// TODO: Need to consider TryFrom instead of From? Some transitions could fail
impl From<Phase<Raw>> for Phase<Clean> {
    fn from(prev: Phase<Raw>) -> Self {
        let state = clean::Info::extract_from_string(&prev.buffer);
        Self {
            buffer: prev.buffer,
            state,
        }
    }
}

/// The phase in which labels are collected and expanded. Resulting struct
/// contains metadata from previous phase and the expanded lines
#[derive(Debug)]
pub struct Expand {
    lines: Vec<String>,
    metadata: clean::Info,
}

impl From<Phase<Clean>> for Phase<Expand> {
    fn from(prev: Phase<Clean>) -> Self {
        let lines = expansion::expand(prev.state.lines);

        Self {
            buffer: prev.buffer,
            state: Expand {
                lines,
                metadata: prev.state.metadata,
            },
        }
    }
}

/// The phase in which string-based lines are converted into in-memory data structures
/// for later simulation. This should be the final phase of parsing.
// TODO: rename? Or just return a `load_file::Program`
#[derive(Debug)]
pub struct Deserialized {
    metadata: clean::Info,
    pub instructions: load_file::Instructions,
}

impl TryFrom<Phase<Expand>> for Phase<Deserialized> {
    type Error = Error;
    fn try_from(prev: Phase<Expand>) -> Result<Self, Error> {
        let instructions = deserialize::deserialize(prev.state.lines)?;

        Ok(Self {
            buffer: prev.buffer,
            state: Deserialized {
                metadata: prev.state.metadata,
                instructions,
            },
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use textwrap_macros::dedent;

    // TODO: are tests really needed here? or can we just use integration tests
    // as "good enough"?

    #[test]
    fn transitions() {
        let raw_phase = Phase::<Raw>::from_str(
            dedent!(
                "
                ;redcode
                ; author Ian Chamberlain
                ORG start
                EQU thing 4
                MOV thing, 0
                start
                MOV thing, thing+1

                "
            )
            .trim(),
        )
        .unwrap();

        let _clean_phase = Phase::<Clean>::from(raw_phase);

        // TODO: expansion transition
    }
}
