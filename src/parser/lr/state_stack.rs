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

use super::State;
use super::Error;

#[derive(Debug, Default)]
pub struct StateStack {
    inner: Vec<State>,
    upper: usize,
}

impl StateStack {
    pub fn cbor() -> StateStack {
        StateStack {
            inner: vec![State::Init],
            upper: 16384,
        }
    }
    pub fn last(&self) -> Option<&State> {
        self.inner.last()
    }
    pub fn pop(&mut self) -> Option<State> {
        self.inner.pop()
    }
    pub fn push(&mut self, state: State) -> Result<(), Error> {
        if self.inner.len() < self.upper {
            Ok(self.inner.push(state))
        } else {
            Err(Error::InsufficientStackSize)
        }
    }
}

impl std::fmt::Display for StateStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self
            .inner
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(" | ");
        write!(f, "[{string}]")
    }
}
