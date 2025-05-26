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

/// Parsing context stack
///
/// The parsing context contains the latest contexts in reverse order.
///
/// The stack is protected against infinite recursion by an upper bound to its
/// length.
#[derive(Debug, Default)]
pub struct ContextStack {
    inner: Vec<Context>,
    upper: usize,
}

impl ContextStack {
    pub fn cbor() -> ContextStack {
        ContextStack {
            inner: vec![Context::NonTerminalSymbol(NonTerm::Value)],
            upper: 16384,
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn pop(&mut self) -> Option<Context> {
        self.inner.pop()
    }

    pub fn push_kind(&mut self, kind: Kind) -> Result<(), Error> {
        if self.inner.len() < self.upper {
            Ok(self.inner.push(Context::TerminalSymbol(kind)))
        } else {
            Err(Error::InsufficientStackSize)
        }
    }

    pub fn push_non_term(&mut self, non_term: NonTerm) -> Result<(), Error> {
        if self.inner.len() < self.upper {
            Ok(self.inner.push(Context::NonTerminalSymbol(non_term)))
        } else {
            Err(Error::InsufficientStackSize)
        }
    }

    pub fn push_multiple_non_term(
        &mut self,
        non_term: NonTerm,
        count: usize,
    ) -> Result<(), Error> {
        if count <= self.upper && self.inner.len() < self.upper - count {
            for _ in 0..count {
                self.inner.push(Context::NonTerminalSymbol(non_term));
            }
            Ok(())
        } else {
            Err(Error::InsufficientStackSize)
        }
    }

    pub fn push_action<Action>(
        &mut self,
        name: impl ToString,
        s: Action,
    ) -> Result<(), Error>
    where
        Action: Fn(&mut ContextStack, &mut ValueStack) + 'static,
    {
        if self.inner.len() < self.upper {
            Ok(self
                .inner
                .push(Context::Action(name.to_string(), Box::new(s))))
        } else {
            Err(Error::InsufficientStackSize)
        }
    }
}
