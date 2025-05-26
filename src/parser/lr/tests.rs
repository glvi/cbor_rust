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
use crate::parser::Parser as _;

#[test]
fn decode_uint() {
    let mut parser = Parser::cbor();
    let val = random_uint();
    let expected = Value::Uint(val);
    let token = Term::from(val);
    let actual = parser.consume(token).unwrap().unwrap();
    assert_eq!(&expected, &actual);
}

#[test]
fn decode_nint() {
    let mut parser = Parser::cbor();
    let val = random_nint();
    let expected = Value::Nint(val);
    let token = Term::Nint(val);
    let actual = parser.consume(token).unwrap().unwrap();
    assert_eq!(&expected, &actual);
}

#[test]
fn decode_bstr() {
    let mut parser = Parser::cbor();
    let bytes = random_bytes();
    let expected = Value::Bstr(bytes.clone());
    let token = Term::Bstr(bytes);
    let actual = parser.consume(token).unwrap().unwrap();
    assert_eq!(&expected, &actual);
}

#[test]
fn decode_bstrx() {
    let mut parser = Parser::cbor();
    let bytes1 = random_bytes();
    let bytes2 = random_bytes();
    let bytes = [bytes1.clone(), bytes2.clone()].concat();
    let expected = Value::Bstr(bytes);
    let token1 = Term::BstrX;
    let token2 = Term::Bstr(bytes1);
    let token3 = Term::Bstr(bytes2);
    let token4 = Term::Break;
    let mut actual;
    actual = parser.consume(token1).unwrap();
    assert_eq!(None, actual);
    actual = parser.consume(token2).unwrap();
    assert_eq!(None, actual);
    actual = parser.consume(token3).unwrap();
    assert_eq!(None, actual);
    actual = parser.consume(token4).unwrap();
    assert_eq!(Some(expected), actual);
}

#[test]
fn decode_tstr() {
    let mut parser = Parser::cbor();
    let text1 = random_text();
    let text2 = random_text();
    let text = [text1.clone(), text2.clone()].concat();
    let expected = Value::Tstr(text);
    let token1 = Term::TstrX;
    let token2 = Term::Tstr(text1);
    let token3 = Term::Tstr(text2);
    let token4 = Term::Break;
    let mut actual;
    actual = parser.consume(token1).unwrap();
    assert_eq!(None, actual);
    actual = parser.consume(token2).unwrap();
    assert_eq!(None, actual);
    actual = parser.consume(token3).unwrap();
    assert_eq!(None, actual);
    actual = parser.consume(token4).unwrap();
    assert_eq!(Some(expected), actual);
}

#[test]
fn decode_tag55799() {
    let mut parser = Parser::cbor();
    let token1 = Term::Tag(55799);
    let token2 = Term::Uint(0);
    let expected = Value::Tag(55799, Box::new(Value::Uint(0)));
    let mut actual;
    actual = parser.consume(token1).unwrap();
    assert_eq!(None, actual);
    actual = parser.consume(token2).unwrap();
    assert_eq!(Some(expected), actual);
}

#[test]
fn decode_simple() {
    let mut parser = Parser::cbor();
    let token = Term::Simple(0xf7);
    let expected = Value::Simple(0xf7);
    let actual = parser.consume(token).unwrap().unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn decode_array0() {
    let mut parser = Parser::cbor();
    let token = Term::Array(0);
    let expected = Value::Array(vec![]);
    let actual;
    actual = parser.consume(token).unwrap();
    assert_eq!(Some(expected), actual);
}

#[test]
fn decode_array() {
    let mut parser = Parser::cbor();
    let value1 = Value::Uint(1);
    let value2 = Value::Nint(2);
    let value3 = Value::Tag(3, Value::Simple(4).into());
    let value4 =
        Value::Array(vec![value1.clone(), value2.clone(), value3.clone()]);
    let expected = Value::Array(vec![value1, value2, value3, value4]);
    let tokens = [
        Term::Array(4),
        Term::Uint(1),
        Term::Nint(2),
        Term::Tag(3),
        Term::Simple(4),
        Term::ArrayX,
        Term::Uint(1),
        Term::Nint(2),
        Term::Tag(3),
        Term::Simple(4),
        Term::Break,
    ];
    for token in tokens {
        if let Some(actual) = parser.consume(token).unwrap() {
            assert_eq!(expected, actual);
            return;
        }
    }
    panic!("Test fell through the end");
}

#[test]
fn decode_arrayx() {
    let mut parser = Parser::cbor();
    let value1 = Value::Uint(1);
    let value2 = Value::Nint(2);
    let value3 = Value::Tag(3, Value::Simple(4).into());
    let value4 =
        Value::Array(vec![value1.clone(), value2.clone(), value3.clone()]);
    let expected = Value::Array(vec![value1, value2, value3, value4]);
    let tokens = [
        Term::ArrayX,
        Term::Uint(1),
        Term::Nint(2),
        Term::Tag(3),
        Term::Simple(4),
        Term::Array(3),
        Term::Uint(1),
        Term::Nint(2),
        Term::Tag(3),
        Term::Simple(4),
        Term::Break,
    ];
    for token in tokens {
        if let Some(actual) = parser.consume(token).unwrap() {
            assert_eq!(expected, actual);
            return;
        }
    }
    panic!("Test fell through the end");
}

#[test]
fn decode_map0() {
    let mut parser = Parser::cbor();
    let token = Term::Map(0);
    let expected = Value::Map(vec![]);
    let actual;
    actual = parser.consume(token).unwrap();
    assert_eq!(Some(expected), actual);
}

#[test]
fn decode_map() {
    let mut parser = Parser::cbor();
    let value1 = Value::Uint(1);
    let value2 = Value::Nint(2);
    let value3 = Value::Tag(3, Value::Simple(4).into());
    let expected =
        Value::Map(vec![
            (value1.clone(), value1.clone()),
            (value2.clone(), value2.clone()),
            (value3.clone(), value3.clone()),
        ]);
    let tokens = [
        Term::Map(3),
        Term::Uint(1),
        Term::Uint(1),
        Term::Nint(2),
        Term::Nint(2),
        Term::Tag(3),
        Term::Simple(4),
        Term::Tag(3),
        Term::Simple(4),
    ];
    for token in tokens {
        if let Some(actual) = parser.consume(token).unwrap() {
            assert_eq!(expected, actual);
            return;
        }
    }
    panic!("Test fell through the end");
}

#[test]
fn decode_mapx() {
    let mut parser = Parser::cbor();
    let value1 = Value::Uint(1);
    let value2 = Value::Nint(2);
    let value3 = Value::Tag(3, Value::Simple(4).into());
    let expected =
        Value::Map(vec![
            (value1.clone(), value1.clone()),
            (value2.clone(), value2.clone()),
            (value3.clone(), value3.clone()),
        ]);
    let tokens = [
        Term::MapX,
        Term::Uint(1),
        Term::Uint(1),
        Term::Nint(2),
        Term::Nint(2),
        Term::Tag(3),
        Term::Simple(4),
        Term::Tag(3),
        Term::Simple(4),
        Term::Break,
    ];
    for token in tokens {
        if let Some(actual) = parser.consume(token).unwrap() {
            assert_eq!(expected, actual);
            return;
        }
    }
    panic!("Test fell through the end");
}

// =============================================================================
// Utilities
// =============================================================================
use rand;

fn random_uint() -> u64 {
    rand::random()
}

fn random_nint() -> u64 {
    u64::try_from(-1 - rand::random_range(i64::MIN..0)).unwrap()
}

fn random_bytes() -> Vec<u8> {
    const CAPACITY: usize = 8;
    let mut result: Vec<u8> = Vec::with_capacity(CAPACITY);
    let mut iter = rand::random_iter::<u8>();
    while let Some(value) = iter.next() {
        result.push(value);
        if result.len() >= CAPACITY {
            return result;
        }
    }
    result
}

fn random_text() -> Vec<u8> {
    const CAPACITY: usize = 8;
    let mut buffer: [u8; CAPACITY] = [0; CAPACITY];
    let mut index: usize = 0;
    while index < CAPACITY {
        let value: u32 = rand::random_range(33..127);
        let c = char::from_u32(value).unwrap();
        c.encode_utf8(&mut buffer[index..]);
        index += 1;
        // Note: Each character in the range of UNICODE codepoints U+33 up to
        // and including U+126 requires exactly one byte in its UTF-8 encoding.
    }
    Vec::from(&buffer[0..index])
}

// (end of file)
