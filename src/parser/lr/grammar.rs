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
//!
//! This is an experimental approach to a bottom-up parser for the CBOR grammar.
//!
//! # LR(1) grammar for CBOR
//!
//! ```ABNF
//! 00 start     = value;
//!
//! 01 value     = %uint;
//! 02 value     = %nint;
//! 03 value     = %simple;
//! 04 value     = %float;
//! 50 value     = %tag(n) value;
//! 06 value     = %array(n) {n}value;
//! 07 value     = %arrayx arrayxseq %break;
//! 08 value     = %map(n) {2n}value;
//! 09 value     = %mapx mapxseq %break;
//! 10 value     = bstr;
//! 11 value     = tstr;
//!
//! 12 arrayxseq = %empty;
//! 13 arrayxseq = arrayxseq value;
//!
//! 14 mapxseq   = %empty;
//! 15 mapxseq   = mapxseq value value;
//!
//! 16 bstr      = %bstr(bytes);
//! 17 bstr      = %bstrx bstrxseq;
//!
//! 18 bstrxseq  = %break;
//! 19 bstrxseq  = bstr bstrxseq;
//!
//! 20 tstr      = %tstr(bytes);
//! 21 tstr      = %tstrx tstrxseq %break;
//!
//! 22 tstrxseq  = %empty;
//! 23 tstrxseq  = tstrxseq tstr;
//! ```
