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
//! A CBOR *value* is one of the following:
//!
//! - a negative integer,
//! - a non-negative integer,
//! - a floating-point number,
//! - a byte string,
//! - a text string,
//! - a tagged value,
//! - a simple value,
//! - a sequence of CBOR values,
//! - a sequence of pairs of CBOR values.

use std::fmt;

/// Numerical representation of kinds of token.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Kind {
    /// Indicates a natural number
    Uint,
    /// Indicates a negative integer
    Nint,
    /// Indicates a sequence of bytes, indefinite length
    BstrX,
    /// Indicates a non-empty sequence of bytes, definite length
    Bstr,
    /// Indicates text string encoded in UTF-8, indefinite length
    TstrX,
    /// Indicates a non-empty text string encoded in UTF-8, definite length
    Tstr,
    /// Indicates a sequence of CBOR values, indefinite length
    ArrayX,
    /// Indicates a non-empty sequence of CBOR values, definite length
    Array,
    /// Indicates a sequence of pairs of CBOR values, indefinite length
    MapX,
    /// Indicates a non-empty sequence of pairs of CBOR values, definite length
    Map,
    /// Indicates a tagged CBOR value
    Tag,
    /// Indicates a simple value
    Simple,
    /// Indicates a rational value in binary floating-point representation
    Float,
    /// Indicates the end of an indefinite sequence
    Break,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = format!("%{self:?}").to_lowercase();
        f.write_str(&str)
    }
}

/// Structured representation of a token.
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Token {
    /// Token for a natural number
    Uint(u64),
    /// Token for a negative integer
    Nint(u64),
    /// Token for a byte string of indefinite length
    BstrX,
    /// Token for a byte string
    Bstr(Vec<u8>),
    /// Token for a text string of indefinite length
    TstrX,
    /// Token for a text string
    Tstr(Vec<u8>),
    /// Token for an array of indefinite length
    ArrayX,
    /// Token for an array of definite length
    Array(u64),
    /// Token for a map of indefinite length
    MapX,
    /// Token for a map
    Map(u64),
    /// Token for a tagged value
    Tag(u64),
    /// Token for a simple value
    Simple(u8),
    /// Token for a floating-point value
    Float(u64),
    /// Token for the end of a sequence of indefinite length
    Break,
}

impl Token {
    /// Returns the kind of a token.
    pub fn kind(&self) -> Kind {
        use Token::*;
        match self {
            Uint(_)   => Kind::Uint,
            Nint(_)   => Kind::Nint,
            BstrX     => Kind::BstrX,
            Bstr(_)   => Kind::Bstr,
            TstrX     => Kind::TstrX,
            Tstr(_)   => Kind::Tstr,
            ArrayX    => Kind::ArrayX,
            Array(_)  => Kind::Array,
            MapX      => Kind::MapX,
            Map(_)    => Kind::Map,
            Tag(_)    => Kind::Tag,
            Simple(_) => Kind::Simple,
            Float(_)  => Kind::Float,
            Break     => Kind::Break,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Token::*;
        match self {
            Uint(arg)   => write!(f, "%uint({arg})"),
            Nint(arg)   => write!(f, "%nint({arg})"),
            BstrX       => write!(f, "%bstrx"),
            Bstr(bytes) => write!(f, "%bstr{bytes:?}"),
            TstrX       => write!(f, "%tstrx"),
            Tstr(bytes) => write!(f, "%tstr{bytes:?}"),
            ArrayX      => write!(f, "%arrayx"),
            Array(arg)  => write!(f, "%array({arg})"),
            MapX        => write!(f, "%mapx"),
            Map(arg)    => write!(f, "%map({arg})"),
            Tag(arg)    => write!(f, "%tag({arg})"),
            Simple(arg) => write!(f, "%simple({arg})"),
            Float(arg)  => write!(f, "%float({arg})"),
            Break       => write!(f, "%break"),
        }
    }
}

impl From<u64> for Token {
    /// Returns `Token::Uint`
    fn from(value: u64) -> Self {
        Token::Uint(value)
    }
}

impl From<i64> for Token {
    /// If `value < 0` returns `Token::Nint`; otherwise returns `Token::Uint`
    fn from(value: i64) -> Self {
        if value < 0 {
            Token::Nint(u64::try_from(-1 - value).unwrap())
        } else {
            Token::Uint(value.try_into().unwrap())
        }
    }
}

impl From<Vec<u8>> for Token {
    /// Returns `Token::Bstr`
    fn from(value: Vec<u8>) -> Self {
        Token::Bstr(value)
    }
}

impl From<String> for Token {
    /// Returns `Token::Tstr`
    fn from(value: String) -> Self {
        Token::Tstr(value.into_bytes())
    }
}

#[cfg(test)]
mod tests;
