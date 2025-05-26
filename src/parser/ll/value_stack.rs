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

/// Parsing value stack
///
/// The value stack keeps track of partial results as the final result is being
/// contructed.
#[derive(Debug, Default)]
pub struct ValueStack {
    inner: Vec<Value>,
}

impl ValueStack {
    /// Returns the number of values in a value stack.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Removes the last value from a value stack and returns it, or [None] if
    /// it is empty.
    pub fn pop(&mut self) -> Option<Value> {
        self.inner.pop()
    }

    /// Appends a value to the back of a value stack.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` _bytes_.
    pub fn push(&mut self, value: Value) {
        self.inner.push(value)
    }

    /// Removes the last `n` values from a value stack, and replaces them with
    /// an array of those values.
    ///
    /// # Panics
    ///
    /// Panics if there are less than `n` values on the value stack.
    ///
    /// Panics if `n` exceeds `isize::MAX`.
    pub fn do_array_collect(&mut self, n: u64) {
        let sz = usize::try_from(n).unwrap();
        let mut values: Vec<Value> = Vec::with_capacity(sz);
        for _ in 0..n {
            let value = self.pop().unwrap();
            values.push(value);
        }
        values.reverse();
        self.push(Value::Array(values));
    }

    /// Removes the last value from a value stack. Removes the then last value
    /// from that stack. Presumes that the latter value is an array. Appends the
    /// former value to the latter array, and pushes the modified array back
    /// onto the stack.
    ///
    /// # Panics
    ///
    /// Panics if the value stack has less than two elements.
    ///
    /// Panics if the second value is not an array.
    pub fn do_array_push(&mut self) {
        let value = self.pop().unwrap();
        let mut values = self.pop().unwrap().as_array().unwrap();
        values.push(value);
        self.push(Value::Array(values));
    }

    pub fn do_bstr_append(&mut self) {
        let mut bytes = self.pop().unwrap().as_bstr().unwrap();
        let mut parent = self.pop().unwrap().as_bstr().unwrap();
        parent.append(&mut bytes);
        self.push(Value::Bstr(parent));
    }

    pub fn do_map_collect(&mut self, n: u64) {
        let sz = usize::try_from(n).unwrap();
        let mut values: Vec<(Value, Value)> = Vec::with_capacity(sz);
        for _ in 0..n {
            let value = self.pop().unwrap();
            let label = self.pop().unwrap();
            values.push((label, value));
        }
        values.reverse();
        self.push(Value::Map(values));
    }

    pub fn do_map_push(&mut self) {
        let value = self.pop().unwrap();
        let label = self.pop().unwrap();
        let mut values = self.pop().unwrap().as_map().unwrap();
        values.push((label, value));
        self.push(Value::Map(values));
    }

    pub fn do_tag_set(&mut self, tag: u64) {
        let value = self.pop().unwrap();
        self.push(Value::Tag(tag, Box::new(value)));
    }

    pub fn do_tstr_append(&mut self) {
        let bytes = self.pop().unwrap().as_tstr().unwrap();
        let mut parent = self.pop().unwrap().as_tstr().unwrap();
        parent.push_str(&bytes);
        self.push(Value::Tstr(parent.into_bytes()));
    }
}
