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

use super::Token;

#[test]
fn display_uint() {
    let token = Token::Uint(12345u64);
    let expected = "%uint(12345)";
    let actual = format!("{token}");
    assert_eq!(expected, actual)
}

#[test]
fn display_nint() {
    let token = Token::Nint(12345u64);
    let expected = "%nint(12345)";
    let actual = format!("{token}");
    assert_eq!(expected, actual)
}

#[test]
fn display_bstrx() {
    let token = Token::BstrX;
    let expected = "%bstrx";
    let actual = format!("{token}");
    assert_eq!(expected, actual)
}

#[test]
fn display_bstr() {
    let token = Token::Bstr(vec![1, 2, 3, 4, 5]);
    let expected = "%bstr[1, 2, 3, 4, 5]";
    let actual = format!("{token}");
    assert_eq!(expected, actual)
}

#[test]
fn display_tstrx() {
    let token = Token::TstrX;
    let expected = "%tstrx";
    let actual = format!("{token}");
    assert_eq!(expected, actual)
}

#[test]
fn display_tstr() {
    let token = Token::Tstr(vec![1, 2, 3, 4, 5]);
    let expected = "%tstr[1, 2, 3, 4, 5]";
    let actual = format!("{token}");
    assert_eq!(expected, actual)
}

#[test]
fn display_arrayx() {
    let token = Token::ArrayX;
    let expected = "%arrayx";
    let actual = format!("{token}");
    assert_eq!(expected, actual)
}

#[test]
fn display_array() {
    let token = Token::Array(12345);
    let expected = "%array(12345)";
    let actual = format!("{token}");
    assert_eq!(expected, actual)
}

#[test]
fn display_mapx() {
    let token = Token::MapX;
    let expected = "%mapx";
    let actual = format!("{token}");
    assert_eq!(expected, actual)
}

#[test]
fn display_map() {
    let token = Token::Map(12345);
    let expected = "%map(12345)";
    let actual = format!("{token}");
    assert_eq!(expected, actual)
}

#[test]
fn display_tag() {
    let token = Token::Tag(12345);
    let expected = "%tag(12345)";
    let actual = format!("{token}");
    assert_eq!(expected, actual)
}

#[test]
fn display_simple() {
    let token = Token::Simple(123);
    let expected = "%simple(123)";
    let actual = format!("{token}");
    assert_eq!(expected, actual)
}

#[test]
fn display_float() {
    let token = Token::Float(12345);
    let expected = "%float(12345)";
    let actual = format!("{token}");
    assert_eq!(expected, actual)
}

#[test]
fn display_break() {
    let token = Token::Break;
    let expected = "%break";
    let actual = format!("{token}");
    assert_eq!(expected, actual)
}

#[test]
fn from_u64() {
    let value: u64 = rand::random();
    assert_eq!(Token::Uint(value), Token::from(value))
}

#[test]
fn from_neg_i64() {
    let value: i64 = rand::random_range(i64::MIN..0);
    let uint_value = u64::try_from(-1 - value).unwrap();
    assert_eq!(Token::Nint(uint_value), Token::from(value));
}

#[test]
fn from_nat_i64() {
    let value: i64 = rand::random_range(0..=i64::MAX);
    let uint_value = u64::try_from(value).unwrap();
    assert_eq!(Token::Uint(uint_value), Token::from(value));
}

#[test]
fn from_vec_u8() {
    let value: Vec<u8> = vec![1, 2, 3];
    let expected = value.clone();
    assert_eq!(Token::Bstr(expected), Token::from(value));
}
