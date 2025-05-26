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

/// What could possibly go wrong when scanning binary data for CBOR encoded
/// information?
#[derive(Debug)]
pub enum Error {
    /// The scanner encountered an unexpected end-of-file
    UnexpectedEof,
    /// The scanner encountered an unexpected head
    UnexpectedHead(u8),
    /// The scanner encountered a byte count or item count that can not be
    /// represented in the `usize` of the platform.
    Excessive(u64),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            UnexpectedEof => write!(f, "Unexpected EOF"),
            UnexpectedHead(head) => write!(f, "Unexpected head: {head}"),
            Excessive(count) => write!(f, "Excessive count ({count})"),
        }
    }
}
