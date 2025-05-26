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

use crate::parser::Parser as ParserTrait;

struct EprintlnVisitor {}

impl ParserVisitor for EprintlnVisitor {
    fn on_input(&self, context: &ContextStack, values: &ValueStack, input: &Term) {
        eprintln!("{values:?}  {context:?} <- {input}");
    }
    fn on_flush(&self, context: &ContextStack, values: &ValueStack) {
        eprintln!("{values:?}  {context:?}");
    }
    fn on_action(&self, _context: &ContextStack, _values: &ValueStack, name: &String) {
        eprintln!(" -=- {name} -=- ");
    }
}

#[test]
fn uint() {
    let mut parser = super::Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let uint_value: u64 = rand::random();
    let uint_result = parser
        .consume(Term::Uint(uint_value))
        .unwrap()
        .unwrap()
        .as_uint()
        .unwrap();
    assert_eq!(uint_value, uint_result);
}

#[test]
fn nint() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let nint_value: i64 = rand::random_range(i64::MIN..0);
    let uint_value: u64 = u64::try_from(-1 - nint_value).unwrap();
    let uint_result = parser
        .consume(Term::Nint(uint_value))
        .unwrap()
        .unwrap()
        .as_nint()
        .unwrap();
    let nint_result = -1 - i64::try_from(uint_result).unwrap();
    assert_eq!(nint_value, nint_result);
}

#[test]
fn simple() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let simple_value: u8 = rand::random();
    let simple_result = parser
        .consume(Term::Simple(simple_value))
        .unwrap()
        .unwrap()
        .as_simple()
        .unwrap();
    assert_eq!(simple_value, simple_result);
}

#[test]
fn bstrx0() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let bstr_result = parser.init()
        .and_then(|_| parser.consume(Term::BstrX))
        .and_then(|_| parser.consume(Term::Break))
        .unwrap()
        .unwrap()
        .as_bstr()
        .unwrap();
    assert_eq!(0, bstr_result.len());
}

#[test]
fn bstrx2() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let bytes0: Vec<u8> = vec![1, 2, 3];
    let bytes1: Vec<u8> = vec![4, 5, 6];
    let bstr_result = parser
        .init()
        .and_then(|_| parser.consume(Term::BstrX))
        .and_then(|_| parser.consume(Term::Bstr(bytes0)))
        .and_then(|_| parser.consume(Term::Bstr(bytes1)))
        .and_then(|_| parser.consume(Term::Break))
        .unwrap()
        .unwrap()
        .as_bstr()
        .unwrap();
    assert_eq!(6, bstr_result.len());
    assert_eq!(vec![1, 2, 3, 4, 5, 6], bstr_result);
}

#[test]
fn bstrx_nested() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let bytes0: Vec<u8> = vec![1, 2, 3];
    let bytes1: Vec<u8> = vec![4, 5, 6];
    let bstr_result = parser
        .init()
        .and_then(|_| parser.consume(Term::BstrX))
        .and_then(|_| parser.consume(Term::BstrX))
        .and_then(|_| parser.consume(Term::Bstr(bytes0)))
        .and_then(|_| parser.consume(Term::Break))
        .and_then(|_| parser.consume(Term::BstrX))
        .and_then(|_| parser.consume(Term::Bstr(bytes1)))
        .and_then(|_| parser.consume(Term::Break))
        .and_then(|_| parser.consume(Term::Break))
        .unwrap()
        .unwrap()
        .as_bstr()
        .unwrap();
    assert_eq!(6, bstr_result.len());
    assert_eq!(vec![1, 2, 3, 4, 5, 6], bstr_result);
}

#[test]
fn bstr0() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let octets: [u8; 6] = rand::random();
    let bstr_value = Vec::<u8>::from_iter(octets.into_iter());
    let bstr_result = parser
        .init()
        .and_then(|_| parser.consume(Term::Bstr(bstr_value.clone())))
        .unwrap()
        .unwrap()
        .as_bstr()
        .unwrap();
    assert_eq!(bstr_value, bstr_result);
}

#[test]
fn tstr() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let tstr_value = "The quick brown fox jumps over the lazy dog";
    let tstr_result = parser
        .init()
        .and_then(|_| parser.consume(Term::Tstr(tstr_value.into())))
        .unwrap()
        .unwrap()
        .as_tstr()
        .unwrap();
    assert_eq!(tstr_value, tstr_result);
}

#[test]
fn tstrx() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let str0 = "The quick brown fox";
    let str1 = "jumps over the lazy dog";
    let tstr_value0 = Term::Tstr(str0.into());
    let tstr_value1 = Term::Tstr(str1.into());
    let tstr_result = parser
        .init()
        .and_then(|_| parser.consume(Term::TstrX))
        .and_then(|_| parser.consume(Term::TstrX))
        .and_then(|_| parser.consume(tstr_value0))
        .and_then(|_| parser.consume(tstr_value1))
        .and_then(|_| parser.consume(Term::Break))
        .and_then(|_| parser.consume(Term::Break))
        .unwrap()
        .unwrap()
        .as_tstr()
        .unwrap();
    assert_eq!([str0, str1].concat(), tstr_result);
}

#[test]
fn array0() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let result = parser
        .init()
        .and_then(|_| parser.consume(Term::Array(0)))
        .unwrap()
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(0, result.len());
}

#[test]
fn array2() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let random0 = rand::random();
    let random1 = rand::random();
    let mut result = parser
        .init()
        .and_then(|_| parser.consume(Term::Array(2)))
        .and_then(|_| parser.consume(Term::Uint(random0)))
        .and_then(|_| parser.consume(Term::Uint(random1)))
        .unwrap()
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(2, result.len());
    assert_eq!(random0, result.remove(0).as_uint().unwrap());
    assert_eq!(random1, result.remove(0).as_uint().unwrap());
}

#[test]
fn array_nested() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let random0 = rand::random();
    let random1 = rand::random();
    let mut result = parser
        .init()
        .and_then(|_| parser.consume(Term::Array(2)))
        .and_then(|_| parser.consume(Term::Array(1)))
        .and_then(|_| parser.consume(Term::Uint(random0)))
        .and_then(|_| parser.consume(Term::Array(1)))
        .and_then(|_| parser.consume(Term::Uint(random1)))
        .unwrap()
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(2, result.len());
    assert_eq!(
        random0,
        result
            .remove(0)
            .as_array()
            .unwrap()
            .remove(0)
            .as_uint()
            .unwrap()
    );
    assert_eq!(
        random1,
        result
            .remove(0)
            .as_array()
            .unwrap()
            .remove(0)
            .as_uint()
            .unwrap()
    );
}

#[test]
fn arrayx0() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let result = parser
        .init()
        .and_then(|_| parser.consume(Term::ArrayX))
        .and_then(|_| parser.consume(Term::Break))
        .unwrap()
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(0, result.len());
}

#[test]
fn arrayx2() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let random0 = rand::random();
    let random1 = rand::random();
    let mut result = parser
        .init()
        .and_then(|_| parser.consume(Term::ArrayX))
        .and_then(|_| parser.consume(Term::Uint(random0)))
        .and_then(|_| parser.consume(Term::Uint(random1)))
        .and_then(|_| parser.consume(Term::Break))
        .unwrap()
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(2, result.len());
    assert_eq!(random0, result.remove(0).as_uint().unwrap());
    assert_eq!(random1, result.remove(0).as_uint().unwrap());
}

#[test]
fn arrayx_nested1() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let random0 = rand::random();
    let random1 = rand::random();
    let mut result = parser
        .init()
        .and_then(|_| parser.consume(Term::ArrayX))
        .and_then(|_| parser.consume(Term::Array(1)))
        .and_then(|_| parser.consume(Term::Uint(random0)))
        .and_then(|_| parser.consume(Term::Array(1)))
        .and_then(|_| parser.consume(Term::Uint(random1)))
        .and_then(|_| parser.consume(Term::Break))
        .unwrap()
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(2, result.len());
    assert_eq!(
        random0,
        result
            .remove(0)
            .as_array()
            .unwrap()
            .remove(0)
            .as_uint()
            .unwrap()
    );
    assert_eq!(
        random1,
        result
            .remove(0)
            .as_array()
            .unwrap()
            .remove(0)
            .as_uint()
            .unwrap()
    );
}

#[test]
fn arrayx_nested2() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let random0 = rand::random();
    let random1 = rand::random();
    let mut result = parser
        .init()
        .and_then(|_| parser.consume(Term::ArrayX))
        .and_then(|_| parser.consume(Term::ArrayX))
        .and_then(|_| parser.consume(Term::Uint(random0)))
        .and_then(|_| parser.consume(Term::Break))
        .and_then(|_| parser.consume(Term::ArrayX))
        .and_then(|_| parser.consume(Term::Uint(random1)))
        .and_then(|_| parser.consume(Term::Break))
        .and_then(|_| parser.consume(Term::Break))
        .unwrap()
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(2, result.len());
    assert_eq!(
        random0,
        result
            .remove(0)
            .as_array()
            .unwrap()
            .remove(0)
            .as_uint()
            .unwrap()
    );
    assert_eq!(
        random1,
        result
            .remove(0)
            .as_array()
            .unwrap()
            .remove(0)
            .as_uint()
            .unwrap()
    );
}

#[test]
fn tag_tdate() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let tdate = "1970-01-01T00:00:00.000000Z";
    let (tag, result) = parser
        .init()
        .and_then(|_| parser.consume(Term::Tag(0)))
        .and_then(|_| parser.consume(Term::Tstr(tdate.into())))
        .unwrap()
        .unwrap()
        .as_tag()
        .unwrap();
    assert_eq!(0, tag);
    assert_eq!(tdate, result.as_tstr().unwrap());
}

#[test]
fn map0() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let result = parser
        .init()
        .and_then(|_| parser.consume(Term::Map(0)))
        .unwrap()
        .unwrap()
        .as_map()
        .unwrap();
    assert_eq!(0, result.len());
}

#[test]
fn map2() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let random0 = rand::random();
    let random1 = rand::random();
    let random2 = rand::random();
    let random3 = rand::random();
    let mut result = parser
        .init()
        .and_then(|_| parser.consume(Term::Map(2)))
        .and_then(|_| parser.consume(Term::Uint(random0)))
        .and_then(|_| parser.consume(Term::Uint(random1)))
        .and_then(|_| parser.consume(Term::Uint(random2)))
        .and_then(|_| parser.consume(Term::Uint(random3)))
        .unwrap()
        .unwrap()
        .as_map()
        .unwrap();
    assert_eq!(2, result.len());
    let (label0, item0) = result.remove(0);
    let (label1, item1) = result.remove(0);
    assert_eq!(random0, label0.as_uint().unwrap());
    assert_eq!(random1, item0.as_uint().unwrap());
    assert_eq!(random2, label1.as_uint().unwrap());
    assert_eq!(random3, item1.as_uint().unwrap());
}

#[test]
fn map_nested() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let random_outer_label_0 = rand::random();
    let random_outer_label_1 = rand::random();
    let random_inner_label_0 = rand::random();
    let random_inner_label_1 = rand::random();
    let random_inner_item_0 = rand::random();
    let random_inner_item_1 = rand::random();
    let mut result_outer = parser
        .init()
        .and_then(|_| parser.consume(Term::Map(2)))
        .and_then(|_| parser.consume(Term::Uint(random_outer_label_0)))
        .and_then(|_| parser.consume(Term::Map(1)))
        .and_then(|_| parser.consume(Term::Uint(random_inner_label_0)))
        .and_then(|_| parser.consume(Term::Uint(random_inner_item_0)))
        .and_then(|_| parser.consume(Term::Uint(random_outer_label_1)))
        .and_then(|_| parser.consume(Term::Map(1)))
        .and_then(|_| parser.consume(Term::Uint(random_inner_label_1)))
        .and_then(|_| parser.consume(Term::Uint(random_inner_item_1)))
        .unwrap()
        .unwrap()
        .as_map()
        .unwrap();
    assert_eq!(2, result_outer.len());
    let (outer_label0, item0) = result_outer.remove(0);
    let (outer_label1, item1) = result_outer.remove(0);
    assert_eq!(random_outer_label_0, outer_label0.as_uint().unwrap());
    assert_eq!(random_outer_label_1, outer_label1.as_uint().unwrap());
    let (inner_label_0, inner_item_0) = item0.as_map().unwrap().remove(0);
    let (inner_label_1, inner_item_1) = item1.as_map().unwrap().remove(0);
    assert_eq!(random_inner_label_0, inner_label_0.as_uint().unwrap());
    assert_eq!(random_inner_item_0, inner_item_0.as_uint().unwrap());
    assert_eq!(random_inner_label_1, inner_label_1.as_uint().unwrap());
    assert_eq!(random_inner_item_1, inner_item_1.as_uint().unwrap());
}

#[test]
fn mapx0() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let result = parser
        .init()
        .and_then(|_| parser.consume(Term::MapX))
        .and_then(|_| parser.consume(Term::Break))
        .unwrap()
        .unwrap()
        .as_map()
        .unwrap();
    assert_eq!(0, result.len());
}

#[test]
fn mapx1() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let mut result = parser
        .init()
        .and_then(|_| parser.consume(Term::MapX))
        .and_then(|_| parser.consume(Term::Nint(0)))
        .and_then(|_| parser.consume(Term::Uint(0)))
        .and_then(|_| parser.consume(Term::Break))
        .unwrap()
        .unwrap()
        .as_map()
        .unwrap();
    assert_eq!(1, result.len());
    let (label, item) = result.remove(0);
    assert_eq!(0, label.as_nint().unwrap());
    assert_eq!(0, item.as_uint().unwrap());
}

#[test]
fn mapx2() {
    let mut parser = Parser::cbor();
    parser.set_visitor(EprintlnVisitor {});
    let mut result = parser
        .init()
        .and_then(|_| parser.consume(Term::MapX))
        .and_then(|_| parser.consume(Term::Nint(0)))
        .and_then(|_| parser.consume(Term::Uint(0)))
        .and_then(|_| parser.consume(Term::Nint(1)))
        .and_then(|_| parser.consume(Term::Uint(1)))
        .and_then(|_| parser.consume(Term::Break))
        .unwrap()
        .unwrap()
        .as_map()
        .unwrap();
    assert_eq!(2, result.len());
    let (label0, item0) = result.remove(0);
    let (label1, item1) = result.remove(0);
    assert_eq!(0, label0.as_nint().unwrap());
    assert_eq!(0, item0.as_uint().unwrap());
    assert_eq!(1, label1.as_nint().unwrap());
    assert_eq!(1, item1.as_uint().unwrap());
}
