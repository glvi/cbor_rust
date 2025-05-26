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

use rand::prelude::*;

mod uint {

    use super::*;

    /// Tests decoding all natural numbers in the range `0..=23`.
    #[test]
    fn decode_small() {
        let mut scanner = Scanner::default();
        let mut bytes = [0u8; 1];
        for value in 0x00..=0x17 {
            bytes[0] = value;
            let mut iter = bytes.iter();
            let token =
                scanner.consume_until_complete(&mut iter).unwrap().unwrap();
            assert_eq!(None, iter.next());
            assert_eq!(Token::Uint(value.into()), token);
        }
    }

    /// Tests decoding all natural numbers in the range `0..=255`.
    #[test]
    fn decode_8_bit() {
        let mut scanner = Scanner::default();
        let mut bytes = [0u8; 2];
        for value in 0x00..=0xff {
            bytes[0] = 0x18;
            bytes[1..2].copy_from_slice(&[value]);
            let mut iter = bytes.iter();
            let token =
                scanner.consume_until_complete(&mut iter).unwrap().unwrap();
            assert_eq!(None, iter.next());
            assert_eq!(Token::Uint(value.into()), token)
        }
    }

    /// Tests decoding 256 natural numbers chosen at random from the
    /// range `0..=65_535`.
    #[test]
    fn decode_16_bit() {
        let mut scanner = Scanner::default();
        let mut bytes = [0u8; 3];
        for _ in 0x00..=0xff {
            let value: u16 = rand::random();
            let value_be_bytes = value.to_be_bytes();
            bytes[0] = 0x19;
            bytes[1..3].copy_from_slice(&value_be_bytes);
            let mut iter = bytes.iter();
            let token =
                scanner.consume_until_complete(&mut iter).unwrap().unwrap();
            assert_eq!(None, iter.next());
            assert_eq!(Token::Uint(value.into()), token)
        }
    }

    /// Tests decoding 256 natural numbers chosen at random from the
    /// range `0..=4_294_967_295`.
    #[test]
    fn decode_32_bit() {
        let mut scanner = Scanner::default();
        let mut bytes = [0u8; 5];
        for _ in 0x00..=0xff {
            let value: u32 = rand::random();
            let value_be_bytes = value.to_be_bytes();
            bytes[0] = 0x1a;
            bytes[1..5].copy_from_slice(&value_be_bytes);
            let mut iter = bytes.iter();
            let token =
                scanner.consume_until_complete(&mut iter).unwrap().unwrap();
            assert_eq!(None, iter.next());
            assert_eq!(Token::Uint(value.into()), token)
        }
    }

    /// Tests decoding 256 natural numbers chosen at random from the
    /// range `0..=18_446_744_073_709_551_615`.
    #[test]
    fn decode_64_bit() {
        let mut scanner = Scanner::default();
        let mut bytes = [0u8; 9];
        for _ in 0x00..=0xff {
            let value: u64 = rand::random();
            let value_be_bytes = value.to_be_bytes();
            bytes[0] = 0x1b;
            bytes[1..9].copy_from_slice(&value_be_bytes);
            let mut iter = bytes.iter();
            let token =
                scanner.consume_until_complete(&mut iter).unwrap().unwrap();
            assert_eq!(None, iter.next());
            assert_eq!(Token::Uint(value.into()), token)
        }
    }
}

mod bstr {
    use std::mem;

    use super::*;

    #[test]
    fn decode_bstr0() {
        let mut scanner = Scanner::default();
        let token = scanner.consume(0x40).unwrap().unwrap();
        assert_eq!(Token::Bstr(vec![]), token)
    }

    #[test]
    fn decode_bstrx() {
        let mut scanner = Scanner::default();
        let token = scanner.consume(0x5f).unwrap().unwrap();
        assert_eq!(Token::BstrX, token)
    }

    #[test]
    fn decode_bstr_8_bit() {
        let mut scanner = Scanner::default();
        let mut bytes: [u8; 256] = rand::random();
        bytes[0] = 0x58;
        bytes[1] = (bytes.len() - 2).try_into().unwrap();
        let bytes = bytes;
        let mut iter = bytes.iter();
        let expected = bytes[2..].to_vec();
        let token = scanner.consume_until_complete(&mut iter).unwrap().unwrap();
        assert_eq!(None, iter.next());
        assert_eq!(Token::Bstr(expected), token)
    }

    #[test]
    fn decode_bstr_16_bit() {
        let mut scanner = Scanner::default();
        let mut bytes: [u8; 256] = rand::random();
        let len: u16 = (bytes.len() - 3).try_into().unwrap();
        let len_be_bytes = len.to_be_bytes();
        bytes[0] = 0x59;
        bytes[1..3].copy_from_slice(&len_be_bytes);
        let bytes = bytes;
        let mut iter = bytes.iter();
        let expected = bytes[3..].to_vec();
        let token = scanner.consume_until_complete(&mut iter).unwrap().unwrap();
        assert_eq!(None, iter.next());
        assert_eq!(Token::Bstr(expected), token)
    }

    #[test]
    fn decode_bstr_32_bit() {
        let mut scanner = Scanner::default();
        let mut bytes: [u8; 256] = rand::random();
        let len: u32 = (bytes.len() - 5).try_into().unwrap();
        let len_be_bytes = len.to_be_bytes();
        bytes[0] = 0x5a;
        bytes[1..5].copy_from_slice(&len_be_bytes);
        let bytes = bytes;
        let mut iter = bytes.iter();
        let expected = bytes[5..].to_vec();
        let token = scanner.consume_until_complete(&mut iter).unwrap().unwrap();
        assert_eq!(None, iter.next());
        assert_eq!(Token::Bstr(expected), token)
    }

    #[test]
    fn decode_bstr_64_bit() {
        type LenT = u64;
        const HEAD: u8 = 0x5b; // Major type BSTR w/ 8argument
        const HEAD_SIZE: usize = mem::size_of::<u8>();
        const ARG_SIZE: usize = mem::size_of::<LenT>();
        const STX: usize = HEAD_SIZE + ARG_SIZE;
        const ETX: usize = 256;

        let bytes = {
            let mut rng = rand::rng();
            let mut bytes: [u8; ETX] = rng.random();
            let len = LenT::try_from(bytes.len() - STX).unwrap();
            let len_be_bytes = len.to_be_bytes();
            bytes[0] = HEAD;
            bytes[1..STX].copy_from_slice(&len_be_bytes);
            bytes
        };
        let mut iter = bytes.iter();
        let expected = bytes[STX..].to_vec();
        let mut scanner = Scanner::default();
        let token = scanner.consume_until_complete(&mut iter).unwrap().unwrap();
        assert_eq!(None, iter.next());
        assert_eq!(Token::Bstr(expected), token)
    }
}
