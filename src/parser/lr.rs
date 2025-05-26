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

use crate::value::Value;

mod action;
use action::Action;

mod production;
use production::Production;

mod state;
use state::State;

mod state_stack;
use state_stack::StateStack;

mod value_stack;
use value_stack::ValueStack;

use super::grammar::non_term::NonTermExt;

/// CBOR parser.
#[derive(Debug, Default)]
pub struct Parser {
    states: StateStack,
    values: ValueStack,
}

impl super::Parser for Parser {
    fn consume(&mut self, term: Term) -> Result<Option<Value>, Error> {
        self.do_consume(Some(term))
    }
}

impl Parser {
    /// Returns a parser initialised for parsing a single CBOR value
    pub fn cbor() -> Parser {
        Parser {
            states: StateStack::cbor(),
            values: ValueStack::default(),
        }
    }

    fn do_consume(
        &mut self,
        term: Option<Term>,
    ) -> Result<Option<Value>, Error> {
        #[cfg(debug_assertions)]
        if let Some(term) = &term {
            eprintln!("{} {} ← {}", self.values, self.states, term);
        } else {
            eprintln!("{} {} ⊣", self.values, self.states);
        }
        let Some(current) = self.states.last() else {
            return Err(Error::Invalid);
        };
        let Some(action) = next_action(current, term)? else {
            return Ok(None);
        };
        match action {
            Action::Shift(state) => self.shift(state),
            Action::Reduce(rule) => self.reduce(rule),
            Action::Accept => self.accept(),
        }
    }

    fn shift(&mut self, state: State) -> Result<Option<Value>, Error> {
        #[cfg(debug_assertions)]
        eprintln!("Shift {}", state);
        // States = […]
        self.states.push(state)?;
        // States = [… state]
        self.do_consume(None)
    }

    fn reduce(&mut self, rule: &Production<'_>) -> Result<Option<Value>, Error> {
        #[cfg(debug_assertions)]
        eprintln!("Reduce {}", rule);
        let nt = (rule.reduce)(self)?;
        let next_state = goto(self.states.last(), nt)?;
        self.states.push(next_state)?;
        self.do_consume(None)
    }

    /// 0: <START> ← <VALUE>
    fn accept(&mut self) -> Result<Option<Value>, Error> {
        const NAME: &str = "Accept";
        // States: [Init Accept]
        // Values: [value]
        let Some(State::Accept) = self.states.pop() else {
            panic!("{NAME}: Expected state `Accept`");
        };
        let Some(result) = self.values.pop() else {
            panic!("{NAME}: Expected value stack [… value]");
        };
        // States: [Init]
        // Values: []
        #[cfg(debug_assertions)]
        {
            eprintln!("{NAME}");
            eprintln!("{} {}", self.values, self.states);
            eprintln!("⇒ {result}");
        }
        Ok(Some(result))
    }

    /// 1: <VALUE> ← %uint(n)
    fn reduce01(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 1";
        // States: [… ValueUint(n)]
        // Values: […]
        let Some(State::ValueUint(n)) = self.states.pop() else {
            panic!("{NAME}: Expected state `ValueUint`");
        };
        self.values.push(Value::Uint(n))?;
        // States: […]
        // Values: [… uint(n)]
        Ok(NonTerm::Value)
    }

    /// 2: <VALUE> ← %nint(n)
    fn reduce02(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 2";
        // States = [… ValueNint(n)]
        // Values = […]
        let Some(State::ValueNint(n)) = self.states.pop() else {
            panic!("{NAME}: Expected state `ValueNint`");
        };
        self.values.push(Value::Nint(n))?;
        // States = […]
        // Values = [… nint(n)]
        Ok(NonTerm::Value)
    }

    /// 3: <VALUE> ← %float
    fn reduce03(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 3";
        // States: [… ValueFloat(n)]
        // Values: […]
        let Some(State::ValueFloat(n)) = self.states.pop() else {
            panic!("{NAME}: Expected state `ValueFloat`");
        };
        self.values.push(Value::Float(n))?;
        // States: […]
        // Values: [… Float(n)]
        Ok(NonTerm::Value)
    }

    /// 4: <VALUE> ← <BSTR>
    fn reduce04(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 4";
        // States: [… ValueBstr]
        // Values: [… bstr]
        let Some(State::ValueBstr) = self.states.pop() else {
            panic!("{NAME}: Expected state `ValueBstr`");
        };
        // States: […]
        // Values: [… bstr]
        Ok(NonTerm::Value)
    }

    /// 5: <VALUE> ← <TSTR>
    fn reduce05(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 5";
        // States: [… ValueTstr]
        // Values: [… tstr]
        let Some(State::ValueTstr) = self.states.pop() else {
            panic!("{NAME}: Expected state `ValueTstr`");
        };
        // States: […]
        // Values: [… tstr]
        Ok(NonTerm::Value)
    }

    /// 6: <VALUE> ← %simple
    fn reduce06(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 6";
        // States: [… ValueSimple(n)]
        // Values: […]
        let Some(State::ValueSimple(n)) = self.states.pop() else {
            panic!("{NAME}: Expected state `ValueSimple`");
        };
        self.values.push(Value::Simple(n))?;
        // States: […]
        // Values: [… Simple(n)]
        Ok(NonTerm::Value)
    }

    /// 7: <VALUE> ← %tag <VALUE>
    fn reduce07(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 7";
        // States = [… TagNumber(t) ValueTag]
        // Values = [… value]
        let Some(State::ValueTag) = self.states.pop() else {
            panic!("{NAME}: Expected state `ValueTag`")
        };
        let Some(State::TagNumber(t)) = self.states.pop() else {
            panic!("{NAME}: Expected state `TagNumber`")
        };
        self.values.to_tagged(t);
        // States = […]
        // Values = [… tag(t,value)]
        Ok(NonTerm::Value)
    }

    /// 8: <VALUE> ← %array(n) {n}<VALUE>
    fn reduce08(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 8";
        // States: [… Array(0,n)…Array(n,n)] #Array = n+1 times
        // Values: [… (Value…Value)] #Value = n times
        let n = {
            let Some(State::ValueArray(k, n)) = self.states.pop() else {
                panic!("{NAME}: Expected state `Array(_,_)`")
            };
            if k != n {
                panic!("{NAME}: Expected state `Array({n},{n})`")
            }
            n
        };
        // States: [… Array(0,n)…Array(n-1,n)]
        // Values: [… (Value…Value)] #Value = n times
        let Ok(sz) = usize::try_from(n) else {
            panic!("{NAME}: Excessive count {n}")
        };
        let mut result = Vec::<Value>::with_capacity(sz);
        for p in 0..n {
            let k = n - p - 1;
            let Some(State::ValueArray(k_, n_)) = self.states.pop() else {
                panic!("{NAME}: Expected state `Array({k},{n})`")
            };
            if k_ != k || n_ != n {
                panic!("{NAME}: Expected state `Array({k},{n})`")
            }
            let Some(value) = self.values.pop() else {
                panic!("{NAME}: Expected value stack [… Value]")
            };
            result.push(value);
        }
        result.reverse();
        self.values.push(Value::Array(result))?;
        // States: […]
        // Values: [… Array[Value…Value]] #Value = n times
        Ok(NonTerm::Value)
    }

    /// 9: <VALUE> ← %arrayx <ARRAYXSEQ>
    fn reduce09(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 9";
        // States = [… ArrayXOpen ValueArrayX]
        // Values = [… array]
        let Some(State::ValueArrayX) = self.states.pop() else {
            panic!("{NAME}: Expected state `ValueArrayX`")
        };
        let Some(State::ArrayXSeqOpen) = self.states.pop() else {
            panic!("{NAME}: Expected state `ArrayXOpen`")
        };
        self.values.reverse_array();
        // States = […]
        // Values = [… array]
        Ok(NonTerm::Value)
    }

    /// 10: <VALUE> ← %map(n) {2n}<VALUE>
    fn reduce10(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 10";
        // States: [… Map(0,n) … Map(n,n)] #Map = n+1 times
        // Values: [… Value … Value] #Value = n times
        let n = {
            let Some(State::ValueMap(k, n)) = self.states.pop() else {
                panic!("{NAME}: Expected state `ValueMap(_,_)`")
            };
            if k != n {
                panic!("{NAME}: Expected state `ValueMap({n},{n})`")
            }
            n
        };
        // States: [… Map(0,n) … Map(n-1,n)] #Map = n times
        // Values: [… Value … Value] #Value = n times
        assert_eq!(0, n % 2);
        let half_n = n / 2;
        let Ok(sz) = usize::try_from(half_n) else {
            panic!("{NAME}: Excessive count {half_n}")
        };
        let mut result = Vec::<(Value, Value)>::with_capacity(sz);
        let mut k = n;
        for _ in 0..half_n {
            let Some(State::ValueMap(_k, _n)) = self.states.pop() else {
                panic!("{NAME}: Expected state `ValueMap({k},{n})`")
            };
            if _k + 1 != k || _n != n {
                panic!("{NAME}: Expected state `ValueMap({k},{n})`")
            }
            k -= 1;
            let Some(State::ValueMap(_k, _n)) = self.states.pop() else {
                panic!("{NAME}: Expected state `ValueMap(k,{n})`")
            };
            if _k + 1 != k || _n != n {
                panic!("{NAME}: Expected state `ValueMap({k},{n})`")
            }
            k -= 1;
            let Some(value2) = self.values.pop() else {
                panic!("{NAME}: Expected value stack [… value value]")
            };
            let Some(value1) = self.values.pop() else {
                panic!("{NAME}: Expected value stack [… value value]")
            };
            result.push((value1, value2));
        }
        result.reverse();
        self.values.push(Value::Map(result))?;
        // States: […]
        // Values: [… map[(value,value)…)]] #Value = n times
        #[cfg(debug_assertions)]
        {
            let Some(value) = self.values.last() else {
                panic!("{NAME}: Failed to produce value stack [… map]")
            };
            let Some(map) = value.as_map_ref() else {
                panic!("{NAME}: Failed to produce value stack [… map]")
            };
            assert_eq!(map.len(), half_n.try_into().unwrap())
        }
        Ok(NonTerm::Value)
    }

    /// 11: <VALUE> ← %mapx <MAPXSEQ>
    fn reduce11(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 11";
        // States = [… MapXSeqOpen ValueMapX]
        // Values = [… map]
        let Some(State::ValueMapX) = self.states.pop() else {
            panic!("{NAME}: Expected state `ValueMapX`")
        };
        let Some(State::MapXSeqOpen) = self.states.pop() else {
            panic!("{NAME}: Expected state `MapXSeqOpen`")
        };
        self.values.reverse_map();
        // States = […]
        // Values = [… map]
        Ok(NonTerm::Value)
    }

    /// 12: <BSTR> ← %bstr
    fn reduce12(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 12";
        // States: [… BstrBstr(bytes)]
        // Values: […]
        let Some(State::BstrBstr(bytes)) = self.states.pop() else {
            panic!("{NAME}: Expected state `BstrBstr`");
        };
        self.values.push(Value::Bstr(bytes))?;
        // States: […]
        // Values: [… bstr(bytes)]
        Ok(NonTerm::Bstr)
    }

    /// 13: <BSTR> ← %bstrx <BSTRXSEQ>
    fn reduce13(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 13";
        // States: [… BstrXOpen BstrBstrX]
        // Values: [… bstr]
        let Some(State::BstrBstrX) = self.states.pop() else {
            panic!("{NAME}: Expected state `BstrBstrX`");
        };
        let Some(State::BstrXSeqOpen) = self.states.pop() else {
            panic!("{NAME}: Expected state `BstrXOpen`");
        };
        // States: […]
        // Values: [… bstr]
        Ok(NonTerm::Bstr)
    }

    /// 14: <TSTR> ← %tstr
    fn reduce14(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 14";
        // States: [… TstrTstr(text)]
        // Values: […]
        let Some(State::TstrTstr(text)) = self.states.pop() else {
            panic!("{NAME}: Expected state `TstrTstr`");
        };
        self.values.push(Value::Tstr(text))?;
        // States: […]
        // Values: [… tstr(text)]
        Ok(NonTerm::Tstr)
    }

    /// 15: <TSTR> ← %tstrx <TSTRXSEQ>
    fn reduce15(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 15";
        // States: [… TstrXOpen TstrTstrX]
        // Values: [… tstr]
        let Some(State::TstrTstrX) = self.states.pop() else {
            panic!("{NAME}: Expected state `TstrTstrX`");
        };
        let Some(State::TstrXSeqOpen) = self.states.pop() else {
            panic!("{NAME}: Expected state `TstrXOpen`");
        };
        // States: […]
        // Values: [… tstr]
        Ok(NonTerm::Tstr)
    }

    /// 16: <BSTRXSEQ> ← %break
    fn reduce16(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 16";
        // States = [… BstrXSeqBreak]
        // Values = […]
        let Some(State::BstrXSeqBreak) = self.states.pop() else {
            panic!("{NAME}: Expected state `BstrXSeqBreak`");
        };
        self.values.push(Value::Bstr(Vec::new()))?;
        // States = […]
        // Values = [… bstr]
        Ok(NonTerm::BstrXSeq)
    }

    /// 17: <BSTRXSEQ> ← <BSTR> <BSTRXSEQ>
    fn reduce17(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 17";
        // States: [… BstrXBstr[as…] BstrXPrepend]
        // Values: [… bstr[bs…]]
        let Some(State::BstrXSeqMore) = self.states.pop() else {
            panic!("{NAME}: Expected state `BstrXPrepend`");
        };
        let Some(State::BstrXSeqBstr(bytes)) = self.states.pop() else {
            panic!("{NAME}: Expected state `BstrXBytes`");
        };
        self.values.bstr_prepend(bytes);
        // States: […]
        // Values: [… bstr[as…bs…]]
        Ok(NonTerm::BstrXSeq)
    }

    /// 18: <TSTRXSEQ> ← %break
    fn reduce18(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 18";
        // States = [… TstrXSeqBreak]
        // Values = […]
        let Some(State::TstrXSeqBreak) = self.states.pop() else {
            panic!("{NAME}: Expected state `TstrXSeqBreak`");
        };
        self.values.push(Value::Tstr(Vec::new()))?;
        // States = […]
        // Values = [… tstr]
        Ok(NonTerm::TstrXSeq)
    }

    /// 19: <TSTRXSEQ> ← <TSTR> <TSTRXSEQ>
    fn reduce19(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 19";
        // States: [… TstrXTstr[as…] TstrXPrepend]
        // Values: [… tstr[ts…]]
        let Some(State::TstrXSeqMore) = self.states.pop() else {
            panic!("{NAME}: Expected state `TstrXPrepend`");
        };
        let Some(State::TstrXSeqTstr(bytes)) = self.states.pop() else {
            panic!("{NAME}: Expected state `TstrXBytes`");
        };
        self.values.tstr_prepend(bytes);
        // States: […]
        // Values: [… tstr[as…bs…]]
        Ok(NonTerm::TstrXSeq)
    }

    /// 20: <ARRAYXSEQ> ← %break
    fn reduce20(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 20";
        // States = [… ArrayXSeqBreak]
        // Values = […]
        let Some(State::ArrayXSeqBreak) = self.states.pop() else {
            panic!("{NAME}: Expected state `ArrayXSeqBreak`");
        };
        self.values.push(Value::Array(Vec::new()))?;
        // States = […]
        // Values = [… array[]]
        Ok(NonTerm::ArrayXSeq)
    }

    /// 21: <ARRAYXSEQ> ← <VALUE> <ARRAYXSEQ>
    fn reduce21(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 21";
        // States = [… ArrayXElement ArrayXAppend]
        // Values = [… value array[…]]
        let Some(State::ArrayXSeqMore) = self.states.pop() else {
            panic!("{NAME}: Expected state `ArrayXAppend`");
        };
        let Some(State::ArrayXSeqValue) = self.states.pop() else {
            panic!("{NAME}: Expected state `ArrayXElement`");
        };
        self.values.merge_value_array();
        // States = […]
        // Values = [… array[… value]]
        Ok(NonTerm::ArrayXSeq)
    }

    /// 22: <MAPXSEQ> ← %break
    fn reduce22(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 22";
        // States = [… MapXSeqBreak]
        // Values = […]
        let Some(State::MapXSeqBreak) = self.states.pop() else {
            panic!("{NAME}: Expected state `MapXSeqBreak`");
        };
        self.values.push(Value::Map(Vec::new()))?;
        // States = […]
        // Values = [… map[]]
        Ok(NonTerm::MapXSeq)
    }

    /// 23: <MAPXSEQ> ← <VALUE> <VALUE> <MAPXSEQ>
    fn reduce23(&mut self) -> Result<NonTerm, Error> {
        const NAME: &str = "Reduce 23";
        // States = [… MapXSeqValue1 MapXSeqValue2 MapXSeqMore]
        // Values = [… value value map[…]]
        let Some(State::MapXSeqMore) = self.states.pop() else {
            panic!("{NAME}: Expected state `MapXSeqMore`");
        };
        let Some(State::MapXSeqValue2) = self.states.pop() else {
            panic!("{NAME}: Expected state `MapXSeqValue2`");
        };
        let Some(State::MapXSeqValue1) = self.states.pop() else {
            panic!("{NAME}: Expected state `MapXSeqValue1`");
        };
        self.values.merge_value_value_map();
        // States = […]
        // Values = [… map[… (value,value)]]
        #[cfg(debug_assertions)]
        {
            let Some(value) = self.values.last() else {
                panic!("{NAME}: Failed to produce value stack [… value]")
            };
            let Some(map) = value.as_map_ref() else {
                panic!("{NAME}: Failed to produce value stack [… map]")
            };
            let Some(_) = map.last() else {
                panic!("{NAME}: Failed to produce value stack [… map[… entry]]")
            };
        }
        Ok(NonTerm::MapXSeq)
    }
}

/// Determines the action to take when the terminal symbol `term` is consumed
/// while in state `state`.
fn next_action<'a>(
    state: &State,
    term: Option<Term>,
) -> Result<Option<Action<'a>>, Error> {
    // Returns the _accept_ action.
    #[inline]
    fn accept<'b>() -> Result<Option<Action<'b>>, Error> {
        Ok(Some(Action::Accept))
    }

    // Returns a _reduce_ action using the reduction function of the specified
    // production `p`
    #[inline]
    fn reduce<'b>(p: &'b Production<'b>) -> Result<Option<Action<'b>>, Error> {
        Ok(Some(Action::Reduce(p)))
    }

    // Return a _shift_ action pushing the specified `state` onto the stack.
    #[inline]
    fn shift<'b>(state: State) -> Result<Option<Action<'b>>, Error> {
        Ok(Some(Action::Shift(state)))
    }

    // Default set of actions when reading a terminal symbol that introduces a
    // value.
    //
    // Invoke as `start_value!(term; _ => todo!())`.
    //
    // The first argument is an identifier of type `Option<Term>` that is being
    // matched upon.
    //
    // The second argument is a list of match arms of the form
    // `pattern => expression` that SHALL complete the match expression.
    //
    // The default set of actions matches on one of the following: `None`,
    // `Some(term)` where `term` matches anything but `Term::Break`. The second
    // argument must specify what to do when a `Term::Break` is encountered
    // when expecting a `Value`.
    macro_rules! start_value {
        ($id:ident; $( $pattern:pat => $expr:expr )+) => {
            match $id {
                None                  => Ok(None),
                Some(Term::Uint(n))   => shift(State::ValueUint(n)),
                Some(Term::Nint(n))   => shift(State::ValueNint(n)),
                Some(Term::Float(n))  => shift(State::ValueFloat(n)),
                Some(Term::Simple(n)) => shift(State::ValueSimple(n)),
                Some(Term::Tag(t))    => shift(State::TagNumber(t)),
                Some(Term::Bstr(b))   => shift(State::BstrBstr(b)),
                Some(Term::BstrX)     => shift(State::BstrXSeqOpen),
                Some(Term::Tstr(t))   => shift(State::TstrTstr(t)),
                Some(Term::TstrX)     => shift(State::TstrXSeqOpen),
                Some(Term::Array(n))  => shift(State::ValueArray(0, n)),
                Some(Term::ArrayX)    => shift(State::ArrayXSeqOpen),
                Some(Term::Map(n))    => shift(State::ValueMap(0, 2 * n)),
                Some(Term::MapX)      => shift(State::MapXSeqOpen),
                $( $pattern           => $expr, )+
            }
        }
    }

    // Default set of actions when reading a terminal symbol that introduces a
    // value. Panics when encountering an unexpected terminal symbol.
    macro_rules! start_value_or_panic {
        ($id:ident) => {
            start_value!($id;
                         Some(term) => panic!(
                             "Error: Unhandled action for state:{state} and input:{term}"
                         ))
        }
    }

    // Default set of actions when reading a terminal symbol that introduces a
    // byte string of indefinite length.
    //
    // Invoke as `start_bstrx!(term; _ => todo!())`.
    //
    // The first argument is an identifier of type `Option<Term>` that is being
    // matched upon.
    //
    // The second argument is a list of match arms of the form
    // `pattern => expression` that SHALL complete the match expression.
    //
    // The default set of actions matches on one of the following: `None`,
    // `Some(Term::Break)`, and `Some(Term::Bstr)`. The second argument must
    // specify what to do when another terminal symbol is encountered when
    // expecting a `Bstr`.
    macro_rules! start_bstrx {
        ($id: ident; $( $pattern:pat => $expr:expr )+) => {
            match term {
                None                    => Ok(None),
                Some(Term::Break)       => shift(State::BstrXSeqBreak),
                Some(Term::Bstr(bytes)) => shift(State::BstrXSeqBstr(bytes)),
                $( $pattern             => $expr, )+
            }
        }
    }

    // Default set of actions when reading a terminal symbol that introduces a
    // byte string of indefinite length. Panics when encountering an unexpected
    // terminal symbol.
    macro_rules! start_bstrx_or_panic {
        ($id:ident) => {
            start_bstrx!($id;
                         Some(term) => panic!(
                             "Error: Unhandled action for state:{state} and input:{term}"
                         ))
        }
    }

    // Default set of actions when reading a terminal symbol that introduces a
    // text string of indefinite length.
    //
    // Invoke as `start_tstrx!(term; _ => todo!())`.
    //
    // The first argument is an identifier of type `Option<Term>` that is being
    // matched upon.
    //
    // The second argument is a list of match arms of the form
    // `pattern => expression` that SHALL complete the match expression.
    //
    // The default set of actions matches on one of the following: `None`,
    // `Some(Term::Break)`, and `Some(Term::Tstr)`. The second argument must
    // specify what to do when another terminal symbol is encountered when
    // expecting a `Tstr`.
    macro_rules! start_tstrx {
        ($id: ident; $( $pattern:pat => $expr:expr )+) => {
            match term {
                None                    => Ok(None),
                Some(Term::Break)       => shift(State::TstrXSeqBreak),
                Some(Term::Tstr(bytes)) => shift(State::TstrXSeqTstr(bytes)),
                $( $pattern             => $expr, )+
            }
        }
    }

    // Default set of actions when reading a terminal symbol that introduces a
    // text string of indefinite length. Panics when encountering an unexpected
    // terminal symbol.
    macro_rules! start_tstrx_or_panic {
        ($id:ident) => {
            start_tstrx!($id;
                         Some(term) => panic!(
                             "Error: Unhandled action for state:{state} and input:{term}"
                         ))
        }
    }

    match state {
        State::Invalid                    => panic!("Error: Invalid state"),

        State::Accept                     => accept(),

        State::ValueUint(_)               => reduce(&PRODUCTION_01),
        State::ValueNint(_)               => reduce(&PRODUCTION_02),
        State::ValueFloat(_)              => reduce(&PRODUCTION_03),
        State::ValueBstr                  => reduce(&PRODUCTION_04),
        State::ValueTstr                  => reduce(&PRODUCTION_05),
        State::ValueSimple(_)             => reduce(&PRODUCTION_06),
        State::ValueTag                   => reduce(&PRODUCTION_07),
        State::ValueArray(k, n) if k == n => reduce(&PRODUCTION_08),
        State::ValueArrayX                => reduce(&PRODUCTION_09),
        State::ValueMap(k, n) if k == n   => reduce(&PRODUCTION_10),
        State::ValueMapX                  => reduce(&PRODUCTION_11),

        State::BstrBstr(_)                => reduce(&PRODUCTION_12),
        State::BstrBstrX                  => reduce(&PRODUCTION_13),

        State::TstrTstr(_)                => reduce(&PRODUCTION_14),
        State::TstrTstrX                  => reduce(&PRODUCTION_15),

        State::BstrXSeqBreak              => reduce(&PRODUCTION_16),
        State::BstrXSeqMore               => reduce(&PRODUCTION_17),

        State::TstrXSeqBreak              => reduce(&PRODUCTION_18),
        State::TstrXSeqMore               => reduce(&PRODUCTION_19),

        State::ArrayXSeqBreak             => reduce(&PRODUCTION_20),
        State::ArrayXSeqMore              => reduce(&PRODUCTION_21),

        State::MapXSeqBreak               => reduce(&PRODUCTION_22),
        State::MapXSeqMore                => reduce(&PRODUCTION_23),

        State::Init                       => start_value_or_panic!(term),
        State::TagNumber(_)               => start_value_or_panic!(term),
        State::ValueArray(k, m) if k < m  => start_value_or_panic!(term),
        State::ValueMap(k, m) if k < m    => start_value_or_panic!(term),
        State::MapXSeqValue1              => start_value_or_panic!(term),
        State::BstrXSeqOpen               => start_bstrx_or_panic!(term),
        State::BstrXSeqBstr(_)            => start_bstrx_or_panic!(term),
        State::TstrXSeqOpen               => start_tstrx_or_panic!(term),
        State::TstrXSeqTstr(_)            => start_tstrx_or_panic!(term),

        State::ArrayXSeqOpen              => start_value!(term;
            Some(Term::Break)             => shift(State::ArrayXSeqBreak)
        ),

        State::ArrayXSeqValue             => start_value!(term;
            Some(Term::Break)             => shift(State::ArrayXSeqBreak)
        ),

        State::MapXSeqOpen                => start_value!(term;
            Some(Term::Break)             => shift(State::MapXSeqBreak)
        ),

        State::MapXSeqValue2              => start_value!(term;
            Some(Term::Break)             => shift(State::MapXSeqBreak)
        ),

        State::ValueArray(k, n) if k > n => shift(State::Invalid),
        State::ValueArray(_, _)          => shift(State::Invalid),

        State::ValueMap(k, n) if k > n   => shift(State::Invalid),
        State::ValueMap(_, _)            => shift(State::Invalid),

    }
}

fn goto(state: Option<&State>, nt: NonTerm) -> Result<State, Error> {
    match state {
        None => Err(Error::Unexpected(format!("Not handled: goto state:⊥, nt:{nt}"))),
        Some(state) => goto2(state, nt),
    }
}

fn goto2(state: &State, nt: NonTerm) -> Result<State, Error> {
    macro_rules! unexpected {
        (was: $id:ident, expected: [$($exp:path),+]) => {
            Err(Error::UnexpectedNT(vec![$($exp),*], $id))
        }
    }
    macro_rules! goto {
        ($id:ident; [ $( $path:path => $expr:expr ),* $(,)?]) => {
            match $id {
                $( $path => $expr, )*
                other    => unexpected!(was: other, expected: [$($path),*]),
            }
        };
        ($id:ident; $path:path => $expr:expr $(,)?) => {
            match $id {
                $path => $expr,
                other => unexpected!(was: other, expected: [$path]),
            }
        }
    }
    use State::*;
    match state {
        Init => goto!(nt; [
            NonTerm::Value => Ok(Accept),
            NonTerm::Bstr => Ok(ValueBstr),
            NonTerm::Tstr => Ok(ValueTstr),
        ]),
        TagNumber(_) => goto!(nt; NonTerm::Value => Ok(ValueTag)),
        BstrXSeqOpen => goto!(nt; NonTerm::BstrXSeq => Ok(BstrBstrX)),
        BstrXSeqBstr(_) => goto!(nt; NonTerm::BstrXSeq => Ok(BstrXSeqMore)),
        TstrXSeqOpen => goto!(nt; NonTerm::TstrXSeq => Ok(TstrTstrX)),
        TstrXSeqTstr(_) => goto!(nt; NonTerm::TstrXSeq => Ok(TstrXSeqMore)),
        ArrayXSeqOpen => goto!(nt; [
            NonTerm::ArrayXSeq => Ok(ValueArrayX),
            NonTerm::Value => Ok(ArrayXSeqValue),
        ]),
        ArrayXSeqValue => goto!(nt; [
            NonTerm::ArrayXSeq => Ok(ArrayXSeqMore),
            NonTerm::Value => Ok(ArrayXSeqValue),
        ]),
        ValueArray(k, n) => goto!(nt; NonTerm::Value => Ok(State::array_next(*k, *n)),),
        MapXSeqOpen => goto!(nt; [
            NonTerm::MapXSeq => Ok(ValueMapX),
            NonTerm::Value => Ok(MapXSeqValue1),
        ]),
        MapXSeqValue1 => goto!(nt; NonTerm::Value => Ok(MapXSeqValue2)),
        MapXSeqValue2 => goto!(nt; [
            NonTerm::MapXSeq => Ok(MapXSeqMore),
            NonTerm::Value => Ok(MapXSeqValue1),
        ]),
        ValueMap(k, n) => goto!(nt; NonTerm::Value => Ok(State::map_next(*k, *n))),
        other => Err(Error::Unexpected(format!("Not handled: goto state:{other}, nt:{nt}"))),
    }
}

/******************************************************************************\
 *                                                                             *
 * Productions                                                                 *
 *                                                                             *
 * 01: <VALUE>     → %uint                                                     *
 * 02: <VALUE>     → %nint                                                     *
 * 03: <VALUE>     → %float                                                    *
 * 04: <VALUE>     → <BSTR>                                                    *
 * 05: <VALUE>     → <TSTR>                                                    *
 * 06: <VALUE>     → %simple                                                   *
 * 07: <VALUE>     → %tag value                                                *
 * 08: <VALUE>     → %array(n) {n}<VALUE>                                      *
 * 09: <VALUE>     → %arrayx <ARRAYXSEQ>                                       *
 * 10: <VALUE>     → %map(n) {2n}<VALUE>                                       *
 * 11: <VALUE>     → %mapx <MAPXSEQ>                                           *
 * 12: <BSTR>      → %bstr                                                     *
 * 13: <BSTR>      → %bstrx <BSTRXSEQ>                                         *
 * 14: <TSTR>      → %tstr                                                     *
 * 15: <TSTR>      → %tstrx <TSTRXSEQ>                                         *
 * 16: <BSTRXSEQ>  → %break                                                    *
 * 17: <BSTRXSEQ>  → <BSTR> <BSTRXSEQ>                                         *
 * 18: <TSTRXSEQ>  → %break                                                    *
 * 19: <TSTRXSEQ>  → <TSTR> <TSTRXSEQ>                                         *
 * 20: <ARRAYXSEQ> → %break                                                    *
 * 21: <ARRAYXSEQ> → <VALUE> <ARRAYXSEQ>                                       *
 * 22: <MAPXSEQ>   → %break                                                    *
 * 23: <MAPXSEQ>   → <VALUE> <VALUE> <MAPXSEQ>                                 *
 *                                                                             *
 * Productions, extended                                                       *
 *                                                                             *
 * 00: <START> → <VALUE>                                                       *
 *                                                                             *
\******************************************************************************/

macro_rules! production {
    ( $left:path => [ $($right:path),* ] ) => {};
    ( $left:path => [ $($right:path),* ; $mul:expr ] ) => {};
}

production!(NonTerm::Value     => [Term::Uint]);
production!(NonTerm::Value     => [Term::Nint]);
production!(NonTerm::Value     => [Term::Float]);
production!(NonTerm::Value     => [NonTerm::Bstr]);
production!(NonTerm::Value     => [NonTerm::Tstr]);
production!(NonTerm::Value     => [Term::Simple]);
production!(NonTerm::Value     => [Term::Tag, NonTerm::Value]);
production!(NonTerm::Value     => [Term::Array(n), NonTerm::Value; n]);
production!(NonTerm::Value     => [Term::ArrayX, NonTerm::ArrayXSeq]);
production!(NonTerm::Value     => [Term::Map(n), NonTerm::Value; 2*n]);
production!(NonTerm::Value     => [Term::MapX, NonTerm::MapXSeq]);
production!(NonTerm::Bstr      => [Term::Bstr]);
production!(NonTerm::Bstr      => [Term::BstrX, NonTerm::BstrXSeq]);
production!(NonTerm::Tstr      => [Term::Tstr]);
production!(NonTerm::Tstr      => [Term::TstrX, NonTerm::TstrXSeq]);
production!(NonTerm::BstrXSeq  => [Term::Break]);
production!(NonTerm::BstrXSeq  => [NonTerm::Bstr, NonTerm::BstrXSeq]);
production!(NonTerm::TstrXSeq  => [Term::Break]);
production!(NonTerm::TstrXSeq  => [NonTerm::Tstr, NonTerm::TstrXSeq]);
production!(NonTerm::ArrayXSeq => [Term::Break]);
production!(NonTerm::ArrayXSeq => [NonTerm::Value, NonTerm::ArrayXSeq]);
production!(NonTerm::MapXSeq   => [Term::Break]);
production!(NonTerm::MapXSeq   => [NonTerm::Value, NonTerm::MapXSeq]);

const PRODUCTION_01: Production = Production {
    num_id: 1,
    left: NonTermExt::Value,
    right: &["%uint"],
    reduce: Parser::reduce01,
};

const PRODUCTION_02: Production = Production {
    num_id: 2,
    left: NonTermExt::Value,
    right: &["%nint"],
    reduce: Parser::reduce02,
};

const PRODUCTION_03: Production = Production {
    num_id: 3,
    left: NonTermExt::Value,
    right: &["%float"],
    reduce: Parser::reduce03,
};

const PRODUCTION_04: Production = Production {
    num_id: 4,
    left: NonTermExt::Value,
    right: &["<BSTR>"],
    reduce: Parser::reduce04,
};

const PRODUCTION_05: Production = Production {
    num_id: 5,
    left: NonTermExt::Value,
    right: &["<TSTR>"],
    reduce: Parser::reduce05,
};

const PRODUCTION_06: Production = Production {
    num_id: 6,
    left: NonTermExt::Value,
    right: &["%simple"],
    reduce: Parser::reduce06,
};

const PRODUCTION_07: Production = Production {
    num_id: 7,
    left: NonTermExt::Value,
    right: &["%tag", "<VALUE>"],
    reduce: Parser::reduce07,
};

const PRODUCTION_08: Production = Production {
    num_id: 8,
    left: NonTermExt::Value,
    right: &["%array(n)", "{n}<VALUE>"],
    reduce: Parser::reduce08,
};

const PRODUCTION_09: Production = Production {
    num_id: 9,
    left: NonTermExt::Value,
    right: &["%arrayx", "<ARRAYXSEQ>"],
    reduce: Parser::reduce09,
};

const PRODUCTION_10: Production = Production {
    num_id: 10,
    left: NonTermExt::Value,
    right: &["%map(n)", "{2n}<VALUE>"],
    reduce: Parser::reduce10,
};

const PRODUCTION_11: Production = Production {
    num_id: 11,
    left: NonTermExt::Value,
    right: &["%mapx", "<MAPXSEQ>"],
    reduce: Parser::reduce11,
};

const PRODUCTION_12: Production = Production {
    num_id: 12,
    left: NonTermExt::Bstr,
    right: &["%bstr"],
    reduce: Parser::reduce12,
};

const PRODUCTION_13: Production = Production {
    num_id: 13,
    left: NonTermExt::Bstr,
    right: &["%bstrx", "<BSTRXSEQ>"],
    reduce: Parser::reduce13,
};

const PRODUCTION_14: Production = Production {
    num_id: 14,
    left: NonTermExt::Tstr,
    right: &["%tstr"],
    reduce: Parser::reduce14,
};

const PRODUCTION_15: Production = Production {
    num_id: 15,
    left: NonTermExt::Tstr,
    right: &["%tstrx", "<TSTRXSEQ>"],
    reduce: Parser::reduce15,
};

const PRODUCTION_16: Production = Production {
    num_id: 16,
    left: NonTermExt::BstrXSeq,
    right: &["%break"],
    reduce: Parser::reduce16,
};

const PRODUCTION_17: Production = Production {
    num_id: 17,
    left: NonTermExt::BstrXSeq,
    right: &["<BSTR>", "<BSTRXSEQ>"],
    reduce: Parser::reduce17,
};

const PRODUCTION_18: Production = Production {
    num_id: 18,
    left: NonTermExt::TstrXSeq,
    right: &["%break"],
    reduce: Parser::reduce18,
};

const PRODUCTION_19: Production = Production {
    num_id: 19,
    left: NonTermExt::TstrXSeq,
    right: &["<TSTR>", "<TSTRXSEQ>"],
    reduce: Parser::reduce19,
};

const PRODUCTION_20: Production = Production {
    num_id: 20,
    left: NonTermExt::ArrayXSeq,
    right: &["%break"],
    reduce: Parser::reduce20,
};

const PRODUCTION_21: Production = Production {
    num_id: 21,
    left: NonTermExt::ArrayXSeq,
    right: &["<VALUE>", "<ARRAYXSEQ>"],
    reduce: Parser::reduce21,
};

const PRODUCTION_22: Production = Production {
    num_id: 22,
    left: NonTermExt::MapXSeq,
    right: &["%break"],
    reduce: Parser::reduce22,
};

const PRODUCTION_23: Production = Production {
    num_id: 23,
    left: NonTermExt::MapXSeq,
    right: &["<VALUE>", "<VALUE>", "<MAPXSEQ>"],
    reduce: Parser::reduce23,
};

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
