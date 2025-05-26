// -*- mode: rust; coding: utf-8-unix; -*-
/*
cbor: Utilities for decoding Concise Binary Object Notation
Copyright (C) 2025 GLVI Gesellschaft für Luftverkehrsinformatik mbH.

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or (at
your option) any later version.

This program is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use super::*;

use crate::scanner::Error as ScanError;

/// What could possibly go wrong with parsing?
#[derive(Debug, Default)]
pub enum Error {
    /// The parser is in an invalid state.
    ///
    /// This is usually the case immediately after construction, and before the
    /// parser has been properly set up.
    #[default]
    Invalid,
    /// The parser is in order, but it needs to see more tokens before it can
    /// accept or reject its input.
    Incomplete,
    /// The parser in its current state has encountered an unexpected terminal
    /// symbol.
    UnexpectedT(Vec<Kind>, Term),
    /// The parser in its current state has encountered an unexpected non-terminal
    /// symbol. (LR parser only)
    UnexpectedNT(Vec<NonTerm>, NonTerm),
    /// The parser in its current state encountered unexpected circumstances
    /// described in the argument
    Unexpected(String),
    /// More input than necessary provided to the parser.
    TrailingInput,
    /// Error propagated from the scanner.
    Scanner(ScanError),
    /// Parsing a token would require more stack size than is available; refers
    /// to context stack size for an LL parser, or state stack size for an LR
    /// parser.
    InsufficientStackSize,
    /// Internal error
    Internal,
    /// To do (for development purposes only)
    Todo(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Invalid => write!(f, "The parser is in invalid state"),
            Error::Incomplete => {
                write!(f, "The parser needs to see more tokens")
            }
            Error::UnexpectedT(expected, actual) => {
                f.write_str("The parser encountered ")?;
                f.write_fmt(format_args!("{actual}"))?;
                f.write_str(" when it was expecting one of [")?;
                let mut iter = expected.into_iter().map(|t| format!("{t}"));
                if let Some(str) = iter.next() {
                    f.write_str(&str)?;
                    for str in iter {
                        f.write_str(", ")?;
                        f.write_str(&str)?;
                    }
                }
                f.write_str("]")?;
                Ok(())
            }
            Error::UnexpectedNT(expected, actual) => {
                f.write_str("The parser encountered ")?;
                f.write_fmt(format_args!("{actual}"))?;
                f.write_str(" when it was expecting one of [")?;
                let mut iter = expected.into_iter().map(|nt| format!("{nt}"));
                if let Some(str) = iter.next() {
                    f.write_str(&str)?;
                    for str in iter {
                        f.write_str(", ")?;
                        f.write_str(&str)?;
                    }
                }
                f.write_str("]")?;
                Ok(())
            }
            Error::Unexpected(message) => write!(f, "{message}"),
            Error::TrailingInput => write!(f, "Trailing input"),
            Error::Scanner(scan_error) => write!(f, "{scan_error}"),
            Error::InsufficientStackSize => write!(f, "507 Insufficient stack size"),
            Error::Internal => write!(f, "500 Internal Error"),
            Error::Todo(string) => write!(f, "TODO: {string}"),
        }
    }
}

impl std::error::Error for Error {}
