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
//! Apart from providing terminal and non-terminal symbols, this module provides
//! no other functionality. Its primary purpose is to document the grammar that
//! is implemented in the parsers.
//!
//! # Grammar
//!
//! A CBOR encoding satisfies the following grammar.
//!
//! Symbols starting with '%' are terminal symbols; symbols bracketed by '<…>'
//! are non-terminal symbols.
//!
//! Terminal symbols may have payload:
//! - `(n)` indicates an unsigned integer as payload;
//! - `(bytes)` indicates a sequence of bytes (octets) as payload.
//!
//! Non-terminal symbols may be prefixed with an indefinite multiplier:
//! - `{n}` indicates to repeat the following non-terminal `n` times,
//!   where `n` is a parameter, see [Indeterminate rules](#indeterminate-rules) below
//!
//! ```ABNF
//! <VALUE>     = %uint(n) / %nint(n) / %float(n) /
//!               %tag(n) <VALUE> / %simple(n) /
//!               %array(n) {n}<VALUE> / %arrayx <ARRAYXSEQ> /
//!               %map(n) {2n}<VALUE> / %mapx <MAPXSEQ> /
//!               <BSTR> / <TSTR>;
//! <BSTR>      = %bstr(bytes) / %bstrx <BSTRXSEQ>;
//! <TSTR>      = %tstr(bytes) / %tstrx <TSTRXSEQ>;
//! <BSTRXSEQ>  = %break / <BSTR> <BSTRXSEQ>;
//! <TSTRXSEQ>  = %break / <TSTR> <TSTRXSEQ>;
//! <ARRAYXSEQ> = %break / <VALUE> <ARRAYXSEQ>;
//! <MAPXSEQ>   = %break / <VALUE> <VALUE> <MAPXSEQ>;
//! ```
//!
//! ## Start symbol
//!
//! ```ABNF
//! <START> = <VALUE>
//! ```
//!
//! ## Terminal symbols
//!
//! ```text
//! T = {%array, %arrayx, %break, %bstr, %bstrx, %float, %map, %mapx, %nint,
//!      %simple, %tag, %tstr, %tstrx, %uint}
//! ```
//!
//! ## Non-terminal symbols
//!
//! ```text
//! N = {<ARRAY>, <ARRAYXSEQ>, <BSTR>, <BSTRXSEQ>, <MAP>, <MAPXSEQ>, <TAG>,
//!      <TSTR>, <TSTRXSEQ>, <VALUE>}
//! ```
//!
//! ## Indeterminate rules
//!
//! Two rules are parameterised on their payloads:
//!
//! ```ABNF
//! <ARRAY> = %array(n) {n}<VALUE>;
//! <MAP>   = %map(n) {2n}<VALUE>;
//! ```
//!
//! In parser speak, when the parser processes the terminal symbol `%array(n)`,
//! it will push the non-terminal `value` onto the parser stack exactly _n_
//! times -- meaning that it expects to find `%array(n)` followed by _n_
//! `value`s.
//!
//! Similarly for `%map(n)` except that the parser expect _2n_ `value`s.
//!
//! The grammar is indeterminate with respect to arrays and maps, until the
//! parser sees an actual `%array` or `%map` terminal symbol.
//!
//! The rule
//!
//! ```ABNF
//! <ARRAY> = %array(n) {n}<VALUE>;
//! ```
//!
//! is equivalent to the countably infinite set of rules
//!
//! ```ABNF
//! <ARRAY> = %array(0)                                   ; no value
//! <ARRAY> = %array(1) <VALUE>                           ;  1 value
//! <ARRAY> = %array(2) <VALUE> <VALUE>                   ;  2 values
//! <ARRAY> = %array(3) <VALUE> <VALUE> <VALUE>           ;  3 values
//!       ⋮
//! <ARRAY> = %array(n) <VALUE> <VALUE> <VALUE> … <VALUE> ;  n values
//!       ⋮
//! ```
//!
//! Similarly, the rule
//!
//! ```ABNF
//! <MAP> = %map(n) {2*n}<VALUE>;
//! ```
//!
//! is equivalent to the countably infinite set of rules
//!
//! ```ABNF
//! <MAP> = %map(0)                                                                   ;  no values
//! <MAP> = %map(1) <VALUE> <VALUE>                                                   ;   2 values
//! <MAP> = %map(2) <VALUE> <VALUE> <VALUE> <VALUE>                                   ;   4 values
//! <MAP> = %map(3) <VALUE> <VALUE> <VALUE> <VALUE> <VALUE> <VALUE>                   ;   6 values
//!       ⋮
//! <MAP> = %map(n) <VALUE> <VALUE> <VALUE> <VALUE> <VALUE> <VALUE> … <VALUE> <VALUE> ; 2×n values
//!       ⋮
//! ```
//!
//! ### First (Fi)
//!
//! _Fi_ : _N_ → 𝒫(_T_) denotes the set of terminal symbols that can start a
//! given non-terminal symbol, where 𝒫(_T_) denotes the power set of _T_.
//!
//! ```plain
//! N         | Fi(N) ∊ 𝒫(T)
//! ----------|---------------------
//! value     | T ∖ {%break}
//! bstr      | {%bstr, %bstrx}
//! bstrxseq  | {%bstr, %bstrx, %break}
//! tstr      | {%tstr, %tstrx}
//! tstrxseq  | {%tstr, %tstrx, %break}
//! tag       | {%tag}
//! array     | {%array, %arrayx}
//! arrayxseq | T
//! map       | {%map, %mapx}
//! mapxseq   | T
//! ```

/// Terminal symbols
pub mod term {

    /// Terminal symbol
    pub type Term = crate::token::Token;

    /// Kind of terminal symbol
    pub type Kind = crate::token::Kind;
}

/// Non-terminal symbols
pub mod non_term {

    /// Non-terminal symbols of the CBOR grammar.
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub enum NonTerm {
        /// Array
        Array,
        /// Tail of an array
        ArrayXSeq,
        /// Byte string
        Bstr,
        /// Tail of a byte string
        BstrXSeq,
        /// Map
        Map,
        /// Tail of a map
        MapXSeq,
        /// Tag
        Tag,
        /// Text string
        Tstr,
        /// Tail of a text string
        TstrXSeq,
        /// Value
        Value,
    }

    impl std::fmt::Display for NonTerm {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let str = format!("<{self:?}>").to_uppercase();
            f.write_str(&str)
        }
    }

    /// Extended non-terminal symbols of the CBOR grammar.
    ///
    /// This includes all non-terminal symbols of the grammar, plus the symbols
    /// [Error](NonTermExt::Error) and [Start](NonTermExt::Start).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
    pub enum NonTermExt {
        #[default]
        /// Error
        Error,
        /// Start
        Start,
        /// Regular non-terminal symbol
        NonTerm(NonTerm)
    }

    impl NonTermExt {
        #[allow(non_upper_case_globals)]
        pub const ArrayXSeq: NonTermExt = Self::NonTerm(NonTerm::ArrayXSeq);
        #[allow(non_upper_case_globals)]
        pub const MapXSeq: NonTermExt = Self::NonTerm(NonTerm::MapXSeq);
        #[allow(non_upper_case_globals)]
        pub const Bstr: NonTermExt = Self::NonTerm(NonTerm::Bstr);
        #[allow(non_upper_case_globals)]
        pub const BstrXSeq: NonTermExt = Self::NonTerm(NonTerm::BstrXSeq);
        #[allow(non_upper_case_globals)]
        pub const Tstr: NonTermExt = Self::NonTerm(NonTerm::Tstr);
        #[allow(non_upper_case_globals)]
        pub const TstrXSeq: NonTermExt = Self::NonTerm(NonTerm::TstrXSeq);
        #[allow(non_upper_case_globals)]
        pub const Value: NonTermExt = Self::NonTerm(NonTerm::Value);
    }

    impl std::fmt::Display for NonTermExt {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                NonTermExt::Start => write!(f, "<START>"),
                NonTermExt::Error => write!(f, "<ERROR>"),
                NonTermExt::NonTerm(non_term) => non_term.fmt(f),
            }
        }
    }

    impl From<NonTerm> for NonTermExt {
        fn from(value: NonTerm) -> Self {
            Self::NonTerm(value)
        }
    }

    impl TryFrom<NonTermExt> for NonTerm {
        type Error = FromNonTermExtError;

        fn try_from(value: NonTermExt) -> Result<Self, Self::Error> {
            if let NonTermExt::NonTerm(nt) = value {
                Ok(nt)
            } else {
                Err(FromNonTermExtError(value))
            }
        }
    }

    /// Indicates that downcasting from a [NonTermExt] to a [NonTerm] failed.
    pub struct FromNonTermExtError(NonTermExt);

    impl std::fmt::Display for FromNonTermExtError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "conversion of {} to `NonTerm` failed", self.0)
        }
    }
}
