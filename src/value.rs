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
//!
//! A CBOR value is one of the following:
//! - a non-negative integer; or
//! - a negative integer; or
//! - a floating-point number; or
//! - a byte string; or
//! - a text string; or
//! - a tagged value; or
//! - a simple value; or
//! - a sequence of values; or
//! - a sequence of pairs of values.

use crate::token;
use std::error;
use std::fmt;

/// CBOR value
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Non-negative integer
    Uint(u64),
    /// Negative integer
    Nint(u64),
    /// Floating-point number
    Float(u64),
    /// Byte string
    Bstr(Vec<u8>),
    /// Text string
    Tstr(Vec<u8>),
    /// Simple value
    Simple(u8),
    /// Tagged value
    Tag(u64, Box<Value>),
    /// Sequence of values
    Array(Vec<Value>),
    /// Sequence of pairs of values
    Map(Vec<(Value, Value)>),
}

impl Value {
    /// Return value as array, or `None`
    pub fn as_array(self) -> Option<Vec<Value>> {
        if let Value::Array(elements) = self {
            Some(elements)
        } else {
            None
        }
    }

    /// Return value as byte string, or `None`
    pub fn as_bstr(self) -> Option<Vec<u8>> {
        if let Value::Bstr(bytes) = self {
            Some(bytes)
        } else {
            None
        }
    }

    /// Return value as map, or `None`
    pub fn as_map(self) -> Option<Vec<(Value, Value)>> {
        if let Value::Map(entries) = self {
            Some(entries)
        } else {
            None
        }
    }

    /// Return value as map, or `None`
    pub fn as_map_ref(&self) -> Option<&Vec<(Value, Value)>> {
        if let Value::Map(entries) = self {
            Some(entries)
        } else {
            None
        }
    }

    /// Return value as negative integer, or `None`
    pub fn as_nint(self) -> Option<u64> {
        if let Value::Nint(number) = self {
            Some(number) // TODO: return -1 - number here, but get the conversion right
        } else {
            None
        }
    }

    /// Return value as simple value, or `None`
    pub fn as_simple(self) -> Option<u8> {
        if let Value::Simple(number) = self {
            Some(number)
        } else {
            None
        }
    }

    /// Return value as tagged value, or `None`
    pub fn as_tag(self) -> Option<(u64, Value)> {
        if let Value::Tag(tag, tagged) = self {
            Some((tag, *tagged))
        } else {
            None
        }
    }

    /// Return value as text string, or `None`
    pub fn as_tstr(self) -> Option<String> {
        if let Value::Tstr(bytes) = self {
            Some(String::from_utf8(bytes).unwrap())
        } else {
            None
        }
    }

    /// Return value as unsigned integer, or `None`
    pub fn as_uint(self) -> Option<u64> {
        if let Value::Uint(number) = self {
            Some(number)
        } else {
            None
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Uint(n) => write!(f, "uint({n})"),
            Value::Nint(n) => write!(f, "nint({n})"),
            Value::Float(n) => write!(f, "float({n})"),
            Value::Bstr(bs) => {
                write!(
                    f,
                    "bstr[{}]",
                    bs.iter()
                        .map(|x| format!("{:x}", x))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Value::Tstr(_) => write!(f, "tstr(…)"),
            Value::Simple(n) => write!(f, "simple({n})"),
            Value::Tag(t, v) => write!(f, "tag({t}, {v})"),
            Value::Array(elements) => {
                write!(
                    f,
                    "array({})",
                    elements
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Value::Map(entries) => {
                write!(
                    f,
                    "map({})",
                    entries
                        .iter()
                        .map(entry_to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}

fn entry_to_string(lv: &(Value, Value)) -> String {
    format!("{}: {}", lv.0.to_string(), lv.1.to_string())
}

impl TryFrom<token::Token> for Value {
    type Error = TryFromTokenError;

    /// Constructs a CBOR value from a parser token.
    ///
    /// The construction is not defined for all tokens. For those tokens an
    /// error is returned.
    fn try_from(token: token::Token) -> Result<Value, Self::Error> {
        use token::Token::*;
        match token {
            tok @ Array(_) | tok @ Map(_) | tok @ Tag(_) | tok @ Break => {
                Err(TryFromTokenError(tok.kind()))
            }
            Uint(n) => Ok(Value::Uint(n)),
            Nint(n) => Ok(Value::Nint(n)),
            BstrX => Ok(Value::Bstr(Vec::new())),
            Bstr(bytes) => Ok(Value::Bstr(bytes)),
            TstrX => Ok(Value::Tstr(Vec::new())),
            Tstr(bytes) => Ok(Value::Tstr(bytes)),
            ArrayX => Ok(Value::Array(Vec::new())),
            MapX => Ok(Value::Map(Vec::new())),
            Simple(s) => Ok(Value::Simple(s)),
            Float(_) => todo!(),
        }
    }
}

/// Indicates that constructing a value from a token failed
#[derive(Debug)]
pub struct TryFromTokenError(token::Kind);

impl fmt::Display for TryFromTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = self.0;
        write!(
            f,
            "Construction of Value from token {kind:?} is not defined"
        )
    }
}

impl error::Error for TryFromTokenError {}
