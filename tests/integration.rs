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

use cbor::parser::{self, *};
use cbor::scanner::{self, *};
use cbor::value::*;

const UNEXPECTED_EOF: Result<Value, parser::Error> =
    Err(parser::Error::Scanner(scanner::error::Error::UnexpectedEof));

fn decode<const N: usize>(
    scanner: &mut Scanner,
    parser: &mut impl Parser,
    &bytes: &[u8; N],
) -> Result<Value, parser::Error> {
    for byte in bytes {
        if let Some(token) =
            scanner.consume(byte).map_err(parser::Error::Scanner)?
        {
            if let Some(value) = parser.consume(token)? {
                return Ok(value);
            }
        }
    }
    UNEXPECTED_EOF
}

fn decode_uint_small(scanner: &mut Scanner, parser: &mut impl Parser) {
    let byte: u8 = rand::random_range(0x00..0x18);
    let token = scanner.consume(byte).unwrap().unwrap();
    let value = parser.consume(token).unwrap().unwrap();
    assert_eq!(Value::Uint(byte.into()), value)
}

fn decode_nint_small(scanner: &mut Scanner, parser: &mut impl Parser) {
    let val: u8 = rand::random_range(0x00..0x18);
    let byte = 0x20 | val;
    let token = scanner.consume(byte).unwrap().unwrap();
    let value = parser.consume(token).unwrap().unwrap();
    assert_eq!(Value::Nint(val.into()), value)
}

fn decode_tag_small(scanner: &mut Scanner, parser: &mut impl Parser) {
    let tag: u8 = rand::random_range(0x00..0x18);
    let val: u8 = rand::random_range(0x00..0x18);
    let bytes: [u8; 2] = [0xc0 | tag, val];
    let result = decode(scanner, parser, &bytes).unwrap();
    assert_eq!(
        Value::Tag(tag.into(), Box::new(Value::Uint(val.into()))),
        result
    );
}

fn decode_tag_u8(scanner: &mut Scanner, parser: &mut impl Parser) {
    let tag: u8 = rand::random_range(0x18..0xff);
    let val: u8 = rand::random_range(0x00..0x18);
    let bytes: [u8; 3] = [0xd8, tag, val];
    let result = decode(scanner, parser, &bytes).unwrap();
    assert_eq!(
        Value::Tag(tag.into(), Box::new(Value::Uint(val.into()))),
        result
    );
}

fn decode_tag_recursive(scanner: &mut Scanner, parser: &mut impl Parser) {
    let tag1: u8 = rand::random_range(0x00..0x18);
    let tag2: u8 = rand::random_range(0x00..0x18);
    let tag3: u8 = rand::random_range(0x00..0x18);
    let val: u8 = rand::random_range(0x00..0x18);
    let bytes: [u8; 4] = [0xc0 | tag1, 0xc0 | tag2, 0xc0 | tag3, val];
    let result = decode(scanner, parser, &bytes).unwrap();
    assert_eq!(
        Value::Tag(
            tag1.into(),
            Box::new(Value::Tag(
                tag2.into(),
                Box::new(Value::Tag(
                    tag3.into(),
                    Box::new(Value::Uint(val.into()))
                ))
            ))
        ),
        result
    );
}

fn decode_array0(scanner: &mut Scanner, parser: &mut impl Parser) {
    let array: u8 = 0x80;
    let bytes: [u8; 1] = [array];
    let result = decode(scanner, parser, &bytes).unwrap();
    assert_eq!(Value::Array(vec![]), result);
}

fn decode_array1(scanner: &mut Scanner, parser: &mut impl Parser) {
    let bytes: [u8; 2] = [0x81, 0x80];
    let result = decode(scanner, parser, &bytes).unwrap();
    assert_eq!(Value::Array(vec![Value::Array(vec![])]), result);
}

fn decode_array2(scanner: &mut Scanner, parser: &mut impl Parser) {
    let bytes: [u8; 5] = [0x82, 0x81, 0x00, 0x81, 0x01];
    let result = decode(scanner, parser, &bytes).unwrap();
    assert_eq!(Value::Array(vec![
        Value::Array(vec![Value::Uint(0)]),
        Value::Array(vec![Value::Uint(1)]),
    ]), result);
}

fn decode_arrayx_empty(scanner: &mut Scanner, parser: &mut impl Parser) {
    let arrayx: u8 = 0x9f;
    let r#break: u8 = 0xff;
    let bytes: [u8; 2] = [arrayx, r#break];
    let result = decode(scanner, parser, &bytes).unwrap();
    assert_eq!(Value::Array(vec![]), result);
}

fn decode_arrayx(scanner: &mut Scanner, parser: &mut impl Parser) {
    let arrayx: u8 = 0x9f;
    let val1: u8 = rand::random_range(0x00..0x18);
    let val2: u8 = rand::random_range(0x00..0x18);
    let val3: u8 = rand::random_range(0x00..0x18);
    let r#break: u8 = 0xff;
    let bytes: [u8; 5] = [arrayx, val1, val2, val3, r#break];
    let result = decode(scanner, parser, &bytes).unwrap();
    assert_eq!(
        Value::Array(vec![
            Value::Uint(val1.into()),
            Value::Uint(val2.into()),
            Value::Uint(val3.into()),
        ]),
        result
    );
}

fn decode_arrayx_recursive(scanner: &mut Scanner, parser: &mut impl Parser) {
    let arrayx: u8 = 0x9f;
    let r#break: u8 = 0xff;
    let bytes: [u8; 6] = [arrayx, arrayx, arrayx, r#break, r#break, r#break];
    let result = decode(scanner, parser, &bytes).unwrap();
    assert_eq!(
        Value::Array(vec![Value::Array(vec![Value::Array(vec![])])]),
        result
    );
}

fn decode_bstrx_empty(scanner: &mut Scanner, parser: &mut impl Parser) {
    let bstrx: u8 = 0x5f;
    let r#break: u8 = 0xff;
    let bytes: [u8; 2] = [bstrx, r#break];
    let result = decode(scanner, parser, &bytes).unwrap();
    assert_eq!(Value::Bstr(vec![]), result);
}

fn decode_bstrx(scanner: &mut Scanner, parser: &mut impl Parser) {
    let bstrx: u8 = 0x5f;
    let r#break: u8 = 0xff;
    let bytes: [u8; 8] = [bstrx, 0x42, 1, 2, 0x42, 3, 4, r#break];
    let result = decode(scanner, parser, &bytes).unwrap();
    assert_eq!(Value::Bstr(vec![1,2,3,4]), result);
}

fn decode_tstrx_empty(scanner: &mut Scanner, parser: &mut impl Parser) {
    let tstrx: u8 = 0x7f;
    let r#break: u8 = 0xff;
    let bytes: [u8; 2] = [tstrx, r#break];
    let result = decode(scanner, parser, &bytes).unwrap();
    assert_eq!(Value::Tstr(vec![]), result);
}

fn decode_tstrx(scanner: &mut Scanner, parser: &mut impl Parser) {
    let tstrx: u8 = 0x7f;
    let r#break: u8 = 0xff;
    let bytes: [u8; 8] = [tstrx, 0x62, 0x41, 0x42, 0x62, 0x43, 0x44, r#break];
    let result = decode(scanner, parser, &bytes).unwrap();
    assert_eq!(Value::Tstr(vec![0x41, 0x42, 0x43, 0x44]), result);
}

mod ll {
    use super::*;
    use cbor::parser::ll;

    #[test]
    fn decode_uint_small() {
        let mut scanner = Scanner::default();
        let mut parser = ll::Parser::cbor();
        super::decode_uint_small(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_nint_small() {
        let mut scanner = Scanner::default();
        let mut parser = ll::Parser::cbor();
        super::decode_nint_small(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_tag_small() {
        let mut scanner = Scanner::default();
        let mut parser = ll::Parser::cbor();
        super::decode_tag_small(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_tag_u8() {
        let mut scanner = Scanner::default();
        let mut parser = ll::Parser::cbor();
        super::decode_tag_u8(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_tag_recursive() {
        let mut scanner = Scanner::default();
        let mut parser = ll::Parser::cbor();
        super::decode_tag_recursive(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_array0() {
        let mut scanner = Scanner::default();
        let mut parser = ll::Parser::cbor();
        super::decode_array0(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_array1() {
        let mut scanner = Scanner::default();
        let mut parser = ll::Parser::cbor();
        super::decode_array1(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_array2() {
        let mut scanner = Scanner::default();
        let mut parser = ll::Parser::cbor();
        super::decode_array2(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_arrayx_empty() {
        let mut scanner = Scanner::default();
        let mut parser = ll::Parser::cbor();
        super::decode_arrayx_empty(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_arrayx() {
        let mut scanner = Scanner::default();
        let mut parser = ll::Parser::cbor();
        super::decode_arrayx(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_arrayx_recursive() {
        let mut scanner = Scanner::default();
        let mut parser = ll::Parser::cbor();
        super::decode_arrayx_recursive(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_bstrx_empty() {
        let mut scanner = Scanner::default();
        let mut parser = ll::Parser::cbor();
        super::decode_bstrx_empty(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_bstrx() {
        let mut scanner = Scanner::default();
        let mut parser = ll::Parser::cbor();
        super::decode_bstrx(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_tstrx_empty() {
        let mut scanner = Scanner::default();
        let mut parser = ll::Parser::cbor();
        super::decode_tstrx_empty(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_tstrx() {
        let mut scanner = Scanner::default();
        let mut parser = ll::Parser::cbor();
        super::decode_tstrx(&mut scanner, &mut parser);
    }
}

mod lr {
    use super::*;
    use cbor::parser::lr;

    #[test]
    fn decode_uint_small() {
        let mut scanner = Scanner::default();
        let mut parser = lr::Parser::cbor();
        super::decode_uint_small(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_nint_small() {
        let mut scanner = Scanner::default();
        let mut parser = lr::Parser::cbor();
        super::decode_nint_small(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_tag_small() {
        let mut scanner = Scanner::default();
        let mut parser = lr::Parser::cbor();
        super::decode_tag_u8(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_tag_u8() {
        let mut scanner = Scanner::default();
        let mut parser = lr::Parser::cbor();
        super::decode_tag_u8(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_tag_recursive() {
        let mut scanner = Scanner::default();
        let mut parser = lr::Parser::cbor();
        super::decode_tag_recursive(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_array0() {
        let mut scanner = Scanner::default();
        let mut parser = lr::Parser::cbor();
        super::decode_array0(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_array1() {
        let mut scanner = Scanner::default();
        let mut parser = lr::Parser::cbor();
        super::decode_array1(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_array2() {
        let mut scanner = Scanner::default();
        let mut parser = lr::Parser::cbor();
        super::decode_array2(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_arrayx_empty() {
        let mut scanner = Scanner::default();
        let mut parser = lr::Parser::cbor();
        super::decode_arrayx_empty(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_arrayx() {
        let mut scanner = Scanner::default();
        let mut parser = lr::Parser::cbor();
        super::decode_arrayx(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_arrayx_recursive() {
        let mut scanner = Scanner::default();
        let mut parser = lr::Parser::cbor();
        super::decode_arrayx_recursive(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_bstrx_empty() {
        let mut scanner = Scanner::default();
        let mut parser = lr::Parser::cbor();
        super::decode_bstrx_empty(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_bstrx() {
        let mut scanner = Scanner::default();
        let mut parser = lr::Parser::cbor();
        super::decode_bstrx(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_tstrx_empty() {
        let mut scanner = Scanner::default();
        let mut parser = lr::Parser::cbor();
        super::decode_tstrx_empty(&mut scanner, &mut parser);
    }

    #[test]
    fn decode_tstrx() {
        let mut scanner = Scanner::default();
        let mut parser = lr::Parser::cbor();
        super::decode_tstrx(&mut scanner, &mut parser);
    }
}
