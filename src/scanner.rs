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
//! # Example
//!
//!     use std::io::Read;
//!
//!     let mut scanner = cbor::scanner::Scanner::default();
//!
//!     let values = vec![
//!         0x9f, 0x17, 0x18, 0x01, 0x19, 0x01, 0x02, 0x1a, 0x01, 0x02,
//!         0x03, 0x04, 0x1b, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
//!         0x08, 0xff,
//!     ];
//!
//!     for byte in values.bytes().map(|x| x.unwrap()) {
//!         if let Some(token) = scanner.consume(byte).unwrap() {
//!             println!("{token:?}");
//!         }
//!     }

use std::mem;

use crate::token::{Kind, Token};

pub mod error;
pub use error::Error;

/// CBOR lexical scanner
///
/// The fundamental operation of the scanner is to [consume](Scanner::consume) a
/// single byte, and maybe to produce a [token](crate::token). When consuming a
/// single byte does not produce a token, consuming more bytes might.
#[derive(Debug, Default)]
pub struct Scanner {
    state: ScanState,
}

impl Scanner {
    /// Consumes a byte, maybe produces a token.
    ///
    /// The method returns
    /// - `Ok(Some(token))` if the scanner has identified `token`; or
    /// - `Ok(None)` if the scanner needs more bytes to identify a token; or
    /// - `Err(scan_error)` if something went wrong.
    ///
    /// In the case of an error, the scanner will retain the current state. That
    /// state may or may not be usable. The safe thing would be to close
    /// whatever input is currently being read from, report the error, call
    /// [Scanner::reset()], and try again.
    ///
    /// See the [module documentation](self) for an example.
    pub fn consume(&mut self, byte: u8) -> Result<Option<Token>, Error> {
        use ScanResult::*;
        // TODO: Define invalid state to stand in for `self.state` until
        //       overwritten by `new_state`
        let state = mem::take(&mut self.state);
        match consume(state, byte) {
            Incomplete(scan_state) => {
                self.state = scan_state;
                Ok(None)
            }
            Complete(scan_state, token) => {
                self.state = scan_state;
                Ok(Some(token))
            }
            Error(scan_error) => Err(scan_error),
        }
    }

    /// Consumes a sequence of bytes through an iterator.
    ///
    /// The method returns
    /// - `Ok(Some(token))` if the scanner has identified `token`; or
    /// - `Ok(None)` if the scanner needs more bytes to identify a token; or
    /// - `Err(scan_error)` if something went wrong.
    pub fn consume_until_complete<'a, Iter>(
        &mut self,
        iter: &mut  Iter,
    ) -> Result<Option<Token>, Error>
    where
        Iter: Iterator<Item = &'a u8>,
    {
        while let Some(byte) = iter.next() {
            if let some @ Some(_) = self.consume(*byte)? {
                return Ok(some);
            }
        }
        Ok(None)
    }

    /// Resets the scanner.
    pub fn reset(&mut self) {
        self.state = ScanState::default()
    }
}

/// State of the scanner during during scanning.
///
/// As the scanner consumes byte after byte, it eventually decodes token after
/// token from those bytes. The [ScanState] captures the intermediate stages of
/// decoding those tokens.
///
/// The scanner is either expecting a head byte, an optional argument, or an
/// optional binary payload.
///
/// The type of a CBOR data item is represented in the most significant three
/// bits of the head. The encoded value of a CBOR data item is either encoded in
/// the least significant five bits of the head, or in the argument following
/// the head, or in the payload following the argument.
///
/// ### CBOR head
/// ```text
///  7 6 5 4 3 2 1 0
/// +-+-+-+-+-+-+-+-+
/// | Typ |   Arg   |
/// +-+-+-+-+-+-+-+-+
/// ```
///
/// ### CBOR item
/// ```text
/// +------+--------------+-------------+
/// | head | [argument …] | [payload …] |
/// +------+--------------+-------------+
/// ⋮
/// ```
///
/// A given state effectively describes what the scanner is expecting the next
/// byte to be.
#[derive(Clone, Debug)]
enum ScanState {
    /// Expecting the next byte to be a head.
    Head,
    /// Expecting the next byte to be part of the argument.
    Arg {
        /// Type of token being decoded
        kind: Kind,
        /// Argument being decoded
        arg: u64,
        /// Number of bytes pending
        pending: usize,
    },
    /// Expecting the next byte to be part of the binary payload.
    Pay {
        /// Type of token being decoded
        kind: Kind,
        /// Binary payload being decoded
        bytes: Vec<u8>,
        /// Number of bytes pending
        pending: usize,
    },
}

impl Default for ScanState {
    /// Initially, scanner expects the next byte to be a head.
    fn default() -> ScanState {
        ScanState::Head
    }
}

/// The result of the scanner having consumed a byte, or having suffered from an
/// error.
#[derive(Debug)]
enum ScanResult {
    /// The scanner has consumed the byte, but the token is still incomplete.
    Incomplete(ScanState),
    /// The scanner has consumed the byte, and completed the token.
    Complete(ScanState, Token),
    /// The scanner has suffered from an error. The byte may or may not have
    /// been consumed.
    Error(Error),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
enum Argc {
    N1 = 1,
    N2 = 2,
    N4 = 4,
    N8 = 8,
}

impl From<Argc> for usize {
    fn from(value: Argc) -> usize {
        usize::from(value as u8)
    }
}

fn token(kind: Kind, argument: u64, payload: Vec<u8>) -> ScanResult {
    ScanResult::Complete(
        ScanState::Head,
        match kind {
            Kind::Uint => Token::Uint(argument),
            Kind::Nint => Token::Nint(argument),
            Kind::BstrX => Token::BstrX,
            Kind::Bstr => Token::Bstr(payload),
            Kind::TstrX => Token::TstrX,
            Kind::Tstr => Token::Tstr(payload),
            Kind::ArrayX => Token::ArrayX,
            Kind::Array => Token::Array(argument),
            Kind::MapX => Token::MapX,
            Kind::Map => Token::Map(argument),
            Kind::Tag => Token::Tag(argument),
            Kind::Simple => Token::Simple(argument.try_into().unwrap()),
            Kind::Float => Token::Float(argument),
            Kind::Break => Token::Break,
        },
    )
}

fn token_uint(arg: u64) -> ScanResult {
    token(Kind::Uint, arg, Vec::new())
}

fn token_nint(arg: u64) -> ScanResult {
    token(Kind::Nint, arg, Vec::new())
}

fn token_bstr_empty() -> ScanResult {
    token(Kind::Bstr, 0, Vec::new())
}

fn token_bstr_indef() -> ScanResult {
    token(Kind::BstrX, 0xff, Vec::new())
}

fn token_tstr_empty() -> ScanResult {
    token(Kind::Tstr, 0, Vec::new())
}

fn token_tstr_indef() -> ScanResult {
    token(Kind::TstrX, 0xff, Vec::new())
}

fn token_array_empty() -> ScanResult {
    token(Kind::Array, 0, Vec::new())
}

fn token_array(arg: u64) -> ScanResult {
    token(Kind::Array, arg, Vec::new())
}

fn token_array_indef() -> ScanResult {
    token(Kind::ArrayX, 0xff, Vec::new())
}

fn token_map_empty() -> ScanResult {
    token(Kind::Map, 0, Vec::new())
}

fn token_map(arg: u64) -> ScanResult {
    token(Kind::Map, arg, Vec::new())
}

fn token_map_indef() -> ScanResult {
    token(Kind::MapX, 0xff, Vec::new())
}

fn token_tag(arg: u64) -> ScanResult {
    token(Kind::Tag, arg, Vec::new())
}

fn token_simple(arg: u64) -> ScanResult {
    token(Kind::Simple, arg, Vec::new())
}

fn token_break() -> ScanResult {
    token(Kind::Break, 0xff, Vec::new())
}

fn gather_argument(kind: Kind, count: Argc) -> ScanResult {
    ScanResult::Incomplete(ScanState::Arg {
        kind,
        arg: 0,
        pending: count.into(),
    })
}

fn gather_bytes(kind: Kind, count: u64) -> ScanResult {
    if let Ok(sz) = count.try_into() {
        ScanResult::Incomplete(ScanState::Pay {
            kind,
            bytes: Vec::with_capacity(sz),
            pending: sz,
        })
    } else {
        ScanResult::Error(Error::Excessive(count))
    }
}

/// Consume a `byte`.
///
/// If the `byte` completes the token currently being read,
/// returns the token; otherwise, returns `Incomplete`, signalling
/// to the caller to provide more bytes. In case of an error,
/// returns the error.
fn consume(state: ScanState, byte: u8) -> ScanResult {
    match state {
        ScanState::Head => match byte {
            // UINT
            0x00..=0x17 => token_uint(byte.into()),
            0x18 => gather_argument(Kind::Uint, Argc::N1),
            0x19 => gather_argument(Kind::Uint, Argc::N2),
            0x1a => gather_argument(Kind::Uint, Argc::N4),
            0x1b => gather_argument(Kind::Uint, Argc::N8),
            // NINT
            0x20..=0x37 => token_nint((byte - 0x20).into()),
            0x38 => gather_argument(Kind::Nint, Argc::N1),
            0x39 => gather_argument(Kind::Nint, Argc::N2),
            0x3a => gather_argument(Kind::Nint, Argc::N4),
            0x3b => gather_argument(Kind::Nint, Argc::N8),
            // BSTR
            0x40 => token_bstr_empty(),
            0x41..=0x57 => gather_bytes(Kind::Bstr, (byte - 0x40).into()),
            0x58 => gather_argument(Kind::Bstr, Argc::N1),
            0x59 => gather_argument(Kind::Bstr, Argc::N2),
            0x5a => gather_argument(Kind::Bstr, Argc::N4),
            0x5b => gather_argument(Kind::Bstr, Argc::N8),
            0x5f => token_bstr_indef(),
            // TSTR
            0x60 => token_tstr_empty(),
            0x61..=0x77 => gather_bytes(Kind::Tstr, (byte - 0x60).into()),
            0x78 => gather_argument(Kind::Tstr, Argc::N1),
            0x79 => gather_argument(Kind::Tstr, Argc::N2),
            0x7a => gather_argument(Kind::Tstr, Argc::N4),
            0x7b => gather_argument(Kind::Tstr, Argc::N8),
            0x7f => token_tstr_indef(),
            // ARRAY
            0x80 => token_array_empty(),
            0x81..=0x97 => token_array((byte - 0x80).into()),
            0x98 => gather_argument(Kind::Array, Argc::N1),
            0x99 => gather_argument(Kind::Array, Argc::N2),
            0x9a => gather_argument(Kind::Array, Argc::N4),
            0x9b => gather_argument(Kind::Array, Argc::N8),
            0x9f => token_array_indef(),
            // MAP
            0xa0 => token_map_empty(),
            0xa1..=0xb7 => token_map((byte - 0xa0).into()),
            0xb8 => gather_argument(Kind::Map, Argc::N1),
            0xb9 => gather_argument(Kind::Map, Argc::N2),
            0xba => gather_argument(Kind::Map, Argc::N4),
            0xbb => gather_argument(Kind::Map, Argc::N8),
            0xbf => token_map_indef(),
            // TAGGED
            0xc0..=0xd7 => token_tag((byte - 0xc0).into()),
            0xd8 => gather_argument(Kind::Tag, Argc::N1),
            0xd9 => gather_argument(Kind::Tag, Argc::N2),
            0xda => gather_argument(Kind::Tag, Argc::N4),
            0xdb => gather_argument(Kind::Tag, Argc::N8),
            // SIMPLE
            0xe0..=0xf7 => token_simple((byte - 0xe0).into()),
            0xf8 => gather_argument(Kind::Simple, Argc::N1),
            // FLOAT
            0xf9 => gather_argument(Kind::Float, Argc::N2),
            0xfa => gather_argument(Kind::Float, Argc::N4),
            0xfb => gather_argument(Kind::Float, Argc::N8),
            // BREAK
            0xff => token_break(),
            // Unexpected cases
            other => ScanResult::Error(Error::UnexpectedHead(other)),
        },
        ScanState::Arg {
            kind,
            mut arg,
            mut pending,
        } => {
            assert!(pending > 0);
            arg <<= 8;
            arg |= u64::from(byte);
            pending -= 1;
            if pending > 0 {
                ScanResult::Incomplete(ScanState::Arg { kind, arg, pending })
            } else if arg == 0 {
                match kind {
                    Kind::Bstr => token_bstr_empty(),
                    Kind::Tstr => token_tstr_empty(),
                    Kind::Array => token_array_empty(),
                    Kind::Map => token_map_empty(),
                    _ => token(kind, arg, Vec::new()),
                }
            } else {
                match kind {
                    Kind::Bstr | Kind::Tstr => gather_bytes(kind, arg),
                    _ => token(kind, arg, Vec::new()),
                }
            }
        }
        ScanState::Pay {
            kind,
            mut bytes,
            mut pending,
        } => {
            assert!(pending > 0);
            bytes.push(byte);
            pending -= 1;
            if pending > 0 {
                ScanResult::Incomplete(ScanState::Pay {
                    kind,
                    bytes,
                    pending,
                })
            } else {
                match kind {
                    Kind::Bstr | Kind::Tstr => token(
                        kind,
                        bytes.len().try_into().unwrap(),
                        bytes.to_vec(),
                    ),
                    _ => panic!("Gathering bytes for {:?}", kind),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
