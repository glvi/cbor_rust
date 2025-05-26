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

use crate::scanner::Scanner;

use crate::parser::Parser as ParserTrait;

use crate::value::Value;

#[test]
#[ignore]
fn test_cbor_print() {
    let values = vec![
        0x00, 0x01, 0x20, 0x21, 0x5f, 0x40, 0x41, 0xef, 0xff, 0x7f, 0x60, 0x61,
        0x40, 0xff, 0x9f, 0x80, 0x81, 0x00, 0xff, 0xbf, 0xa0, 0xa1, 0x00, 0x00,
        0xff, 0xc0, 0x41, 0x30, 0xe0, 0xe1, 0xf8, 0x30, 0xf9, 0x00, 0x00, 0xfa,
        0x00, 0x00, 0x00, 0x00, 0xfb, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ];
    let mut scanner = Scanner::default();
    for byte in values {
        if let Ok(Some(token)) = scanner.consume(byte) {
            println!("{}", token)
        }
    }
}

#[test]
fn scanner_parser() {
    let values = vec![
        0x9f, 0x17, 0x18, 0x01, 0x19, 0x01, 0x02, 0x1a, 0x01, 0x02, 0x03, 0x04,
        0x1b, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0xff,
    ];
    let mut scanner = Scanner::default();
    let mut parser = Parser::cbor();
    let decode = |s: &mut Scanner, p: &mut Parser| {
        for byte in values {
            if let Some(token) = s.consume(byte).unwrap() {
                if let result @ Some(_) = p.consume(token).unwrap() {
                    return result;
                }
            }
        }
        None
    };
    _ = parser.init();
    let result = decode(&mut scanner, &mut parser).unwrap();
    let mut items = result.as_array().unwrap();
    assert_eq!(5, items.len());
    assert_eq!(0x17, items.remove(0).as_uint().unwrap());
    assert_eq!(0x01, items.remove(0).as_uint().unwrap());
    assert_eq!(0x0102, items.remove(0).as_uint().unwrap());
    assert_eq!(0x01020304, items.remove(0).as_uint().unwrap());
    assert_eq!(0x0102030405060708, items.remove(0).as_uint().unwrap());
}

#[test]
fn scanner_parser_2() {
    fn decode(values: Vec<u8>) -> Result<Value, Error> {
        let mut scanner = Scanner::default();
        let mut parser = Parser::cbor();
        for byte in values {
            let Some(token) =
                scanner.consume(byte).map_err(Error::Scanner)?
            else {
                continue;
            };
            let Some(value) = parser.consume(token)? else {
                continue;
            };
            return Ok(value);
        }
        Err(Error::Incomplete)
    }

    let values = vec![
        0x9f, 0x17, 0x18, 0x01, 0x19, 0x01, 0x02, 0x1a, 0x01, 0x02, 0x03, 0x04,
        0x1b, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0xff,
    ];
    let expected = Value::Array(vec![
        Value::Uint(0x17),
        Value::Uint(0x01),
        Value::Uint(0x0102),
        Value::Uint(0x01020304),
        Value::Uint(0x0102030405060708),
    ]);
    assert_eq!(expected, decode(values).unwrap());
}
