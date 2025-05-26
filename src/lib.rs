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
//! Utilities for decoding Concise Binary Object Representation (CBOR).
//!
//! CBOR ("SEE-boar") is a data format aimed at having very compact encoders and
//! decoders. CBOR encodes as primitives: byte strings, UTF-8 text strings,
//! binary floating point numbers, and integers; and as composites: arrays,
//! maps, and tagged and simple values.
//!
//! CBOR is Internet Standard 94 [[STD94](#std94)].
//!
//! This crate provides a [parser] together with a [scanner]. The scanner
//! consumes bytes, and produces [tokens](token), which can then be consumed by
//! the parser. The parser consumes tokens, and may produce as output a CBOR
//! [value].
//!
//! The scanner and parser have been designed for use with serial lines where
//! bytes trickle rather than stream.  The scanner works with all forms of
//! streams or collection of bytes, but there is no optimisation when used with
//! buffers or files.
//!
//! # References
//! <dl>
//! <dt id="std94">[STD94]</dt><dd>
//!
//! Internet Standard 94, <https://www.rfc-editor.org/info/std94>.  At the time
//! of writing, this STD comprises the following:
//!
//! Bormann, C. and P. Hoffman, "Concise Binary Object Representation (CBOR)",
//! STD 94, RFC 8949, DOI 10.17487/RFC8949, December 2020,
//! <https://www.rfc-editor.org/info/rfc8949>.
//!
//! </dd></dl>
//!
//! # Examples
//! ## Simplest case
//! Assuming an LL(1)-parser
//! ```
//! use cbor::scanner::Scanner;
//! use cbor::parser::*;
//! let mut scanner = Scanner::default();
//! let mut parser = ll::Parser::cbor();
//! let byte = 0x00;
//! let token = scanner.consume(byte).unwrap().unwrap();
//! let value = parser.consume(token).unwrap().unwrap();
//! assert_eq!(cbor::value::Value::Uint(0), value)
//! ```
//!
//! ## Parse byte buffer
//! Assuming an LL(1)-parser
//! ```
//! use cbor::scanner::Scanner;
//! use cbor::parser::*;
//!
//! fn decode(values: Vec<u8>) -> Result<cbor::value::Value, Error> {
//!     let mut scanner = Scanner::default();
//!     let mut parser = ll::Parser::cbor();
//!     for byte in values {
//!         let Some(token) = scanner
//!             .consume(byte)
//!             .map_err(Error::Scanner)?
//!         else {
//!             continue;
//!         };
//!         let Some(value) = parser.consume(token)? else {
//!             continue;
//!         };
//!         return Ok(value);
//!     }
//!     Err(Error::Incomplete)
//! }
//!
//! let values = vec![
//!     0x9f, 0x17, 0x18, 0x01, 0x19, 0x01, 0x02, 0x1a, 0x01, 0x02, 0x03, 0x04,
//!     0x1b, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0xff,
//! ];
//!
//! let expected = cbor::value::Value::Array(vec![
//!     cbor::value::Value::Uint(0x17),
//!     cbor::value::Value::Uint(0x01),
//!     cbor::value::Value::Uint(0x0102),
//!     cbor::value::Value::Uint(0x01020304),
//!     cbor::value::Value::Uint(0x0102030405060708),
//! ]);
//!
//! assert_eq!(expected, decode(values).unwrap());
//! ```

/// Tokens exchanged between [scanner] and [parser].
pub mod token;

/// Everything about scanning [bytes](u8), and producing [tokens](token) in the process.
pub mod scanner;

/// Everything about parsing [tokens](token), and producing a CBOR [value] in
/// the process.
pub mod parser;

/// Everything about CBOR values.
pub mod value;

// TODO
// mod render;

#[cfg(test)]
mod tests;
