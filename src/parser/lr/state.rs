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

/// Parser states
#[derive(Clone, Debug, Default)]
pub enum State {
    /// The parser is in an invalid state.
    /// ```plain
    /// <ERROR> ∙
    /// ```
    #[default]
    Invalid,

    /// The parser is ready to receive its first token.
    /// ```plain
    /// <START> → ∙ <VALUE>
    /// ----------------------------------
    /// <VALUE> → ∙ %uint
    /// <VALUE> → ∙ %tag <VALUE>
    /// <VALUE> → ∙ %arrayxseq <ARRAYXSEQ>
    /// ```
    Init,

    /// The parser has accepted a complete CBOR value.
    /// ```plain
    /// <START> → <VALUE> ∙
    /// ```
    Accept,

    /// The parser has accepted an unsigned integer value.
    /// ```plain
    /// <VALUE> → %uint ∙
    /// ```
    ValueUint(u64),
    /// The parser has accepted a negative integer value.
    /// ```plain
    /// <VALUE> → %nint ∙
    /// ```
    ValueNint(u64),
    /// The parser has accepted a floating-point value.
    /// ```plain
    /// <VALUE> → %float(n) ∙
    /// ```
    ValueFloat(u64),
    /// The parser has accepted a byte string.
    /// ```plain
    /// <VALUE> → <BSTR> ∙
    /// ```
    ValueBstr,
    /// The parser has accepted a text string.
    /// ```plain
    /// <VALUE> → <TSTR> ∙
    /// ```
    ValueTstr,
    /// The parser has accepted a simple value.
    /// ```plain
    /// <VALUE> → %simple ∙
    /// ```
    ValueSimple(u8),
    /// The parser has accepted a tagged value.
    /// ```plain
    /// <VALUE> → %tag <VALUE> ∙
    /// ```
    ValueTag,
    /// The parser has accepted an indefinite-length array.
    /// ```plain
    /// <VALUE> → %arrayx <ARRAYXSEQ> ∙
    /// ```
    ValueArrayX,
    /// The parser has accepted an indefinite-length map.
    /// ```plain
    /// <VALUE> → %mapx <MAPXSEQ> ∙
    /// ```
    ValueMapX,

    /// The parser has started parsing an indefinite-length byte string.
    /// ```plain
    /// <BSTR> → %bstrx ∙ <BSTRXSEQ>
    /// ```
    BstrXSeqOpen,
    /// The parser has finished parsing an indefinite-length byte string.
    /// ```plain
    /// <BSTRXSEQ> → %break ∙
    /// ```
    BstrXSeqBreak,
    /// The parser has accepted a byte string while parsing an indefinite-length
    /// byte string.
    /// ```plain
    /// <BSTRXSEQ> → %bstr ∙ <BSTRXSEQ>
    /// ```
    BstrXSeqBstr(Vec<u8>),
    /// The parser has integrated the parsed byte string into the currently parsed
    /// indefinite-length byte string.
    /// ```plain
    /// <BSTRXSEQ> → <BSTR> <BSTRXSEQ> ∙
    /// ```
    BstrXSeqMore,
    /// The parser has accepted a definite-length byte string.
    /// ```plain
    /// <BSTR> → %bstr ∙
    /// ```
    BstrBstr(Vec<u8>),
    /// The parser has accepted an indefinite-length byte string.
    /// ```plain
    /// <BSTR> → %bstrx <BSTRXSEQ> ∙
    /// ```
    BstrBstrX,

    /// The parser has started parsing an indefinite-length text string.
    /// ```plain
    /// <TSTR> → %tstrx ∙ <TSTRXSEQ>
    /// ```
    TstrXSeqOpen,
    /// The parser has finished parsing an indefinite-length text string.
    /// ```plain
    /// <TSTRXSEQ> → %break ∙
    /// ```
    TstrXSeqBreak,
    /// The parser has accepted a text string while parsing an indefinite-length
    /// text string.
    /// ```plain
    /// <TSTRXSEQ> → %tstr ∙ <TSTRXSEQ>
    /// ```
    TstrXSeqTstr(Vec<u8>),
    /// The parser has integrated the parsed text string into the currently parsed
    /// indefinite-length text string.
    /// ```plain
    /// <TSTRXSEQ> → <TSTR> <TSTRXSEQ> ∙
    /// ```
    TstrXSeqMore,
    /// The parser has accepted a definite-length text string.
    /// ```plain
    /// <TSTR> → %tstr ∙
    /// ```
    TstrTstr(Vec<u8>),
    /// The parser has accepted an indefinite-length text string.
    /// ```plain
    /// <TSTR> → %tstrx <TSTRXSEQ> ∙
    /// ```
    TstrTstrX,

    /// The parser has started parsing a tagged value.
    /// ```plain
    /// <VALUE> → %tag ∙ <VALUE>
    /// ```
    TagNumber(u64),

    /// The parser is parsing a definite-length array.
    /// ```plain
    /// <VALUE> → %array(n) {k}<VALUE> ∙ {n-k}<VALUE>
    /// ```
    ValueArray(u64, u64),

    /// The parser has started parsing an indefinite-length array.
    /// ```plain
    /// <VALUE> → %arrayx ∙ <ARRAYXSEQ>
    /// ```
    ArrayXSeqOpen,
    /// The parser has finished parsing an indefinite-length array.
    /// ```plain
    /// <ARRAYXSEQ> → %break ∙
    /// ```
    ArrayXSeqBreak,
    /// The parser has accepted a CBOR value while parsing an indefinite-length
    /// array.
    /// ```plain
    /// <ARRAYXSEQ> → <VALUE> ∙ <ARRAYXSEQ>
    /// ```
    ArrayXSeqValue,
    /// The parser has integrated the parsed value into the currently parsed
    /// indefinite-length array.
    /// ```plain
    /// <ARRAYXSEQ> → <VALUE> <ARRAYXSEQ> ∙
    /// ```
    ArrayXSeqMore,

    /// The parser is parsing a definite-length map.
    /// ```plain
    /// <VALUE> → %map(n) {k}<VALUE> ∙ {2n-k}<VALUE>
    /// ```
    ValueMap(u64, u64),

    /// The parser has started parsing an indefinite-length map.
    /// ```plain
    /// <VALUE> → %mapx ∙ <MAPXSEQ>
    /// ```
    MapXSeqOpen,
    /// The parser has finished parsing an indefinite-length map.
    /// ```plain
    /// <MAPXSEQ> → %break ∙
    /// ```
    MapXSeqBreak,
    /// The parser has accepted a CBOR value while parsing an indefinite-length
    /// map.
    /// ```plain
    /// <MAPXSEQ> → <VALUE> ∙ <VALUE> <MAPXSEQ>
    /// ```
    MapXSeqValue1,
    /// The parser has accepted a CBOR value while parsing an indefinite-length
    /// map.
    /// ```plain
    /// <MAPXSEQ> → <VALUE> <VALUE> ∙ <MAPXSEQ>
    /// ```
    MapXSeqValue2,
    /// The parser has integrated the parsed values into the currently parsed
    /// indefinite-length map.
    /// ```plain
    /// <MAPXSEQ> → <VALUE> <VALUE> <MAPXSEQ> ∙
    /// ```
    MapXSeqMore,
}

impl State {
    pub fn array_next(k: u64, n: u64) -> State {
        if k < n {
            State::ValueArray(k + 1, n)
        } else {
            State::ValueArray(n, n)
        }
    }

    pub fn map_next(k: u64, n: u64) -> State {
        if k < n {
            State::ValueMap(k + 1, n)
        } else {
            State::ValueMap(n, n)
        }
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
