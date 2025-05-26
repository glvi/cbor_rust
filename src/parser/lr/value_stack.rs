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
use super::Error;

/// Holds intermediate results as values are being reconstructed from tokens.
#[derive(Clone, Debug)]
pub struct ValueStack {
    inner: Vec<Value>,
    upper: usize,
}

impl Default for ValueStack {
    fn default() -> ValueStack {
        ValueStack {
            inner: Default::default(),
            upper: 16384,
        }
    }
}

impl ValueStack {
    /// Allows a peek at the value on top of the stack
    #[cfg(debug_assertions)]
    pub fn last(&self) -> Option<&Value> {
        self.inner.last()
    }

    /// Pops a value from the stack
    pub fn pop(&mut self) -> Option<Value> {
        self.inner.pop()
    }

    /// Pushes `value`
    pub fn push(&mut self, value: Value) -> Result<(), Error> {
        if self.inner.len() < self.upper {
            Ok(self.inner.push(value))
        } else {
            Err(Error::InsufficientStackSize)
        }
    }

    /// Merges the two elements on top of the stack into an array.
    ///
    /// Expects `value array[…]` on top.
    ///
    /// Pops `value`, and replaces `array[…]` with `array[… value]`.
    pub fn merge_value_array(&mut self) {
        let Some(array) = self.inner.pop() else {
            panic!("Expected value stack [… value array[…]]");
        };
        let Some(mut elements) = array.as_array() else {
            panic!("Expected value stack [… value array[…]]");
        };
        let Some(value) = self.inner.pop() else {
            panic!("Expected value stack [… value array[…]]");
        };
        elements.push(value);
        self.inner.push(Value::Array(elements));
    }

    /// Merges the three elements on top of the stack into a map.
    ///
    /// Expects `value value map[…]` on top.
    ///
    /// Pops `value` and `value`, and replaces `map[…]` with `map[… (value,value)]`.
    pub fn merge_value_value_map(&mut self) {
        let Some(map) = self.inner.pop() else {
            panic!("Expected value stack [… value value map[…]]");
        };
        let Some(mut entries) = map.as_map() else {
            panic!("Expected value stack [… value value map[…]]");
        };
        let Some(value2) = self.inner.pop() else {
            panic!("Expected value stack [… value value map[…]]");
        };
        let Some(value1) = self.inner.pop() else {
            panic!("Expected value stack [… value value map[…]]");
        };
        entries.push((value1, value2));
        self.inner.push(Value::Map(entries));
    }

    /// Reverses the array on top of the stack.
    ///
    /// Expects `array` on top.
    pub fn reverse_array(&mut self) {
        let Some(coll) = self.inner.pop() else {
            panic!("Expected value stack [… array[…]]");
        };
        let Some(mut elements) = coll.as_array() else {
            panic!("Expected value stack [… array[…]]");
        };
        elements.reverse();
        self.inner.push(Value::Array(elements));
    }

    /// Reverses the map on top of the stack.
    ///
    /// Expects `map` on top.
    pub fn reverse_map(&mut self) {
        let Some(coll) = self.inner.pop() else {
            panic!("Expected value stack [… map[…]]");
        };
        let Some(mut elements) = coll.as_map() else {
            panic!("Expected value stack [… map[…]]");
        };
        elements.reverse();
        self.inner.push(Value::Map(elements));
    }

    /// Prepends the `bytes` to the bstr on top of the stack
    ///
    /// Expects `bstr` on top.
    pub fn bstr_prepend(&mut self, bytes: Vec<u8>) {
        let Some(value) = self.inner.pop() else {
            panic!("Expected value stack [… bstr]");
        };
        let Some(more_bytes) = value.as_bstr() else {
            panic!("Expected value stack [… bstr]");
        };
        self.inner.push(Value::Bstr([bytes, more_bytes].concat()));
    }

    /// Prepends the `bytes` to the tstr on top of the stack
    ///
    /// Expects `tstr` on top.
    pub fn tstr_prepend(&mut self, bytes: Vec<u8>) {
        let Some(value) = self.inner.pop() else {
            panic!("Expected value stack [… tstr]");
        };
        let Some(more_bytes) = value.as_tstr() else {
            panic!("Expected value stack [… tstr]");
        };
        self.inner.push(Value::Tstr([bytes, more_bytes.into()].concat()));
    }

    /// Promotes the value on top of the stack to a tagged value
    ///
    /// Expects a value on top of the stack
    pub fn to_tagged(&mut self, tag: u64) {
        let Some(value) = self.inner.pop() else {
            panic!("Expected value stack [… value]");
        };
        self.inner.push(Value::Tag(tag, Box::new(value)));
    }
}

impl std::fmt::Display for ValueStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self
            .inner
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{{{string}}}")
    }
}
