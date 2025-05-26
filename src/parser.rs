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

use crate::value::Value;

/// Terminal and non-terminal symbols.
pub mod grammar;
use grammar::non_term::NonTerm;
use grammar::term::Kind;
use grammar::term::Term;

pub trait Parser {
    /// Consumes a `token`, maybe produces a CBOR value.
    ///
    /// The method returns
    /// - `Ok(Some(value))` if the parser has produced a CBOR `value`; or
    /// - `Ok(None)` if the parser needs more tokens to produce a CBOR value; or
    /// - `Err(parse_error)` if something went wrong.
    ///
    /// In the case of an internal error, the state of the parser is
    /// unusable. The safe thing to do would be to drop the parser.
    ///
    /// In the case of an invalid error, the parser is in a pre-initialization
    /// state. Once the parser is initialised, it can be used again.
    fn consume(&mut self, term: Term) -> Result<Option<Value>, Error>;
}

/// Parser errors
pub mod error;
pub use error::Error;

/// LL(1) top-down parser for CBOR
pub mod ll;

/// LR(1) bottom-up parser for CBOR
pub mod lr;

#[cfg(test)]
mod tests;
