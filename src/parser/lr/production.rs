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

/// Production of a grammar
#[derive(Debug)]
#[allow(dead_code)]
pub struct Production<'a> {
    /// Numerical identifier
    pub num_id: usize,
    /// Left-hand side symbol
    pub left: NonTermExt,
    /// Right-hand side symbols
    pub right: &'a [&'a str],
    /// Reduction function that reduces the right-hand side to the left-hand side.
    pub reduce: fn(&mut Parser) -> Result<NonTerm, Error>,
}

impl<'a> std::fmt::Display for Production<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Production {
            num_id,
            left,
            right,
            reduce,
        } = self;
        let _ignored = &[reduce];
        write!(f, "{:3} {} → {}", num_id, left, right.join(" "))
    }
}
