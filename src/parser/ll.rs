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
use context_stack::ContextStack;
use std::fmt;
use value_stack::ValueStack;

mod context_stack;
mod value_stack;

/// CBOR parser.
#[derive(Debug, Default)]
pub struct Parser {
    inner: State,
}

impl super::Parser for Parser {
    fn consume(&mut self, term: Term) -> Result<Option<Value>, Error> {
        self.inner.do_consume(term)?;
        if self.inner.cxt_stack.len() > 0 {
            Ok(None)
        } else if self.inner.val_stack.len() > 1 {
            Err(Error::Internal)
        } else if let Some(value) = self.inner.val_stack.pop() {
            Ok(Some(value))
        } else {
            Err(Error::Invalid)
        }
    }
}

impl Parser {

    /// Returns a parser for a single CBOR value
    pub fn cbor() -> Parser {
        Parser {
            inner: State {
                cxt_stack: ContextStack::cbor(),
                val_stack: ValueStack::default(),
                parse_visitor: None,
            },
        }
    }

    /// Attaches a visitor to the parser
    pub fn set_visitor<Visitor>(&mut self, visitor: Visitor)
    where
        Visitor: ParserVisitor + 'static,
    {
        self.inner.parse_visitor = Some(Box::new(visitor));
    }

    /// Initialises the parser.
    pub fn init(&self) -> Result<Option<Value>, Error> {
        if let Some(v) = &self.inner.parse_visitor {
            v.on_init(&self.inner.cxt_stack, &self.inner.val_stack);
        }
        Ok(None)
    }
}

#[derive(Default)]
struct State {
    cxt_stack: ContextStack,
    val_stack: ValueStack,
    parse_visitor: Option<Box<dyn ParserVisitor>>,
}

impl State {
    #[inline]
    fn do_consume(&mut self, term: Term) -> Result<(), Error> {
        do_consume(
            &self.parse_visitor,
            &mut self.val_stack,
            &mut self.cxt_stack,
            term,
        )
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("State")
            .field("cxt_stack", &self.cxt_stack)
            .field("val_stack", &self.val_stack)
            .finish()
    }
}

/// Parsing context
///
/// The parsing context denotes the value that is currently being constructed
/// according to the underlying grammar.
#[allow(private_interfaces)]
pub enum Context {
    /// A parsing action for transforming the value stack.
    Action(String, Box<dyn Fn(&mut ContextStack, &mut ValueStack)>),
    /// The parser recognised this terminal symbol
    TerminalSymbol(Kind),
    /// The parser recognised this non-terminal symbol
    NonTerminalSymbol(NonTerm),
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Context::*;
        match self {
            Action(name, _) => {
                write!(f, "{name}()")
            }
            TerminalSymbol(kind) => {
                let kind_str = format!("{kind:?}").to_lowercase();
                write!(f, "%{kind_str}")
            }
            NonTerminalSymbol(non_term) => {
                write!(f, "<{non_term:?}>")
            }
        }
    }
}

/// The `ParserVisitor` trait allows for interested clients to be informed when
/// the parser transitions from one state to another.
pub trait ParserVisitor {
    /// Invoked when the parser is initialised.
    fn on_init(&self, _: &ContextStack, _: &ValueStack) {}
    /// Invoked when the parser is transitioning as it consumes tokens.
    fn on_input(&self, _: &ContextStack, _: &ValueStack, _: &Term) {}
    /// Invoked when the parser is transitioning through intermediate states.
    fn on_flush(&self, _: &ContextStack, _: &ValueStack) {}
    /// Invoked when the parser executes a named action.
    fn on_action(&self, _: &ContextStack, _: &ValueStack, _: &String) {}
}

/// Runs the parser until it can no longer apply productions.
fn do_flush(
    parse_visitor: &Option<Box<dyn ParserVisitor>>,
    cxt_stack: &mut ContextStack,
    val_stack: &mut ValueStack,
) -> Result<(), Error> {
    if let Some(visitor) = parse_visitor {
        visitor.on_flush(cxt_stack, val_stack);
    }
    if let Some(context) = cxt_stack.pop() {
        do_flush_(parse_visitor, val_stack, cxt_stack, context)
    } else {
        Ok(())
    }
}

fn do_flush_(
    parse_visitor: &Option<Box<dyn ParserVisitor>>,
    val_stack: &mut ValueStack,
    cxt_stack: &mut ContextStack,
    context: Context,
) -> Result<(), Error> {
    use Context::*;
    match context {
        Action(name, action) => {
            if let Some(visitor) = &parse_visitor {
                visitor.on_action(&cxt_stack, &val_stack, &name);
            }
            action(cxt_stack, val_stack);
            do_flush(parse_visitor, cxt_stack, val_stack)
        }
        TerminalSymbol(kind) => {
            cxt_stack.push_kind(kind)
        }
        NonTerminalSymbol(non_term) => {
            cxt_stack.push_non_term(non_term)
        }
    }
}

fn do_consume(
    parse_visitor: &Option<Box<dyn ParserVisitor>>,
    val_stack: &mut ValueStack,
    cxt_stack: &mut ContextStack,
    input: Term,
) -> Result<(), Error> {
    if let Some(visitor) = parse_visitor {
        visitor.on_input(cxt_stack, val_stack, &input);
    }
    if let Some(context) = cxt_stack.pop() {
        do_consume_(parse_visitor, val_stack, cxt_stack, context, input)
    } else {
        Err(Error::TrailingInput)
    }
}

fn do_consume_(
    parse_visitor: &Option<Box<dyn ParserVisitor>>,
    val_stack: &mut ValueStack,
    cxt_stack: &mut ContextStack,
    context: Context,
    input: Term,
) -> Result<(), Error> {
    use Context::*;
    match context {
        Action(name, action) => {
            if let Some(visitor) = &parse_visitor {
                visitor.on_action(&cxt_stack, &val_stack, &name);
            }
            action(cxt_stack, val_stack);
            do_consume(parse_visitor, val_stack, cxt_stack, input)
        }

        TerminalSymbol(kind) if kind == input.kind() => {
            // Create value from consumed input. If conversion fails, ignore
            // the error, but consume the token.
            if let Ok(value) = Value::try_from(input) {
                val_stack.push(value);
            }
            do_flush(parse_visitor, cxt_stack, val_stack)
        }

        TerminalSymbol(kind) => {
            // Put context back on stack, and complain about unexpected input
            cxt_stack.push_kind(kind)?;
            Err(Error::UnexpectedT(vec![kind], input))
        }

        NonTerminalSymbol(NonTerm::Value) => match input {
            Term::Break => {
                cxt_stack.push_non_term(NonTerm::Value)?;
                Err(Error::UnexpectedT(
                    vec![
                        Kind::Array,
                        Kind::ArrayX,
                        Kind::Bstr,
                        Kind::BstrX,
                        Kind::Float,
                        Kind::Map,
                        Kind::MapX,
                        Kind::Nint,
                        Kind::Simple,
                        Kind::Tag,
                        Kind::Tstr,
                        Kind::TstrX,
                        Kind::Uint,
                    ],
                    input,
                ))
            }
            // Production: value = %uint / %nint / %simple / %float
            Term::Uint(_)
            | Term::Nint(_)
            | Term::Simple(_)
            | Term::Float(_) => {
                cxt_stack.push_kind(input.kind())?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
            // Production: value = bstr
            Term::Bstr(_) | Term::BstrX => {
                cxt_stack.push_non_term(NonTerm::Bstr)?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
            // Production: value = tstr
            Term::Tstr(_) | Term::TstrX => {
                cxt_stack.push_non_term(NonTerm::Tstr)?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
            // Production: value = array
            Term::Array(_) | Term::ArrayX => {
                cxt_stack.push_non_term(NonTerm::Array)?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
            // Production: value = map
            Term::Map(_) | Term::MapX => {
                cxt_stack.push_non_term(NonTerm::Map)?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
            // Production: value = tag
            Term::Tag(_) => {
                cxt_stack.push_non_term(NonTerm::Tag)?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
        },

        NonTerminalSymbol(NonTerm::Array) => match input {
            // Production: array = %array(n) value ... value ; n times
            Term::Array(n) => {
                cxt_stack.push_action("collect_array", array_collect(n))?;
                cxt_stack.push_multiple_non_term(NonTerm::Value, n.try_into().unwrap())?;
                cxt_stack.push_kind(Kind::Array)?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
            // Production: array = %arrayx arrayxseq
            Term::ArrayX => {
                cxt_stack.push_non_term(NonTerm::ArrayXSeq)?;
                cxt_stack.push_kind(Kind::ArrayX)?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
            // Error
            _ => {
                cxt_stack.push_non_term(NonTerm::Array)?;
                Err(Error::UnexpectedT(vec![Kind::Array, Kind::ArrayX], input))
            }
        },

        NonTerminalSymbol(NonTerm::ArrayXSeq) => match input {
            // Production: arrayxseq = %break
            Term::Break => {
                cxt_stack.push_kind(Kind::Break)?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
            // Production: arrayxseq = value arrayxseq
            _ => {
                cxt_stack.push_non_term(NonTerm::ArrayXSeq)?;
                cxt_stack.push_action("array_push", array_push())?;
                cxt_stack.push_non_term(NonTerm::Value)?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
        },

        NonTerminalSymbol(NonTerm::Bstr) => match input {
            // Production: bstr = %bstr(payload)
            Term::Bstr(_) => {
                cxt_stack.push_kind(Kind::Bstr)?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
            // Production: bstr = %bstrx bstrxseq
            Term::BstrX => {
                cxt_stack.push_non_term(NonTerm::BstrXSeq)?;
                cxt_stack.push_kind(Kind::BstrX)?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
            // Error
            _ => {
                cxt_stack.push_non_term(NonTerm::Bstr)?;
                Err(Error::UnexpectedT(vec![Kind::Bstr, Kind::BstrX], input))
            }
        },

        NonTerminalSymbol(NonTerm::BstrXSeq) => {
            match input {
                // Production: bstrxseq = %break
                Term::Break => {
                    cxt_stack.push_kind(Kind::Break)?;
                    do_consume(parse_visitor, val_stack, cxt_stack, input)
                }
                // Production: bstrxseq = bstr bstrxseq
                Term::Bstr(_) | Term::BstrX => {
                    cxt_stack.push_non_term(NonTerm::BstrXSeq)?;
                    cxt_stack.push_action("bstr_append", bstr_append())?;
                    cxt_stack.push_non_term(NonTerm::Bstr)?;
                    do_consume(parse_visitor, val_stack, cxt_stack, input)
                }
                // Error
                _ => {
                    cxt_stack.push_non_term(NonTerm::BstrXSeq)?;
                    Err(Error::UnexpectedT(
                        vec![Kind::Break, Kind::Bstr, Kind::BstrX],
                        input,
                    ))
                }
            }
        }

        NonTerminalSymbol(NonTerm::Map) => match input {
            // Production: map = %map(n) {n}value
            Term::Map(n) => {
                cxt_stack.push_action("map_collect", map_collect(n))?;
                cxt_stack.push_multiple_non_term(NonTerm::Value, (2 * n).try_into().unwrap())?;
                cxt_stack.push_kind(Kind::Map)?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
            // Production: map = %mapx mapxseq
            Term::MapX => {
                cxt_stack.push_non_term(NonTerm::MapXSeq)?;
                cxt_stack.push_kind(Kind::MapX)?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
            // Error
            _ => {
                cxt_stack.push_non_term(NonTerm::Map)?;
                Err(Error::UnexpectedT(vec![Kind::Map, Kind::MapX], input))
            }
        },

        NonTerminalSymbol(NonTerm::MapXSeq) => match input {
            // Production: mapxseq = %break
            Term::Break => {
                cxt_stack.push_kind(Kind::Break)?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
            // Production: mapxseq = label:value item:value mapxseq
            _ => {
                cxt_stack.push_non_term(NonTerm::MapXSeq)?;
                cxt_stack.push_action("map_push", map_push())?;
                cxt_stack.push_non_term(NonTerm::Value)?; // item
                cxt_stack.push_non_term(NonTerm::Value)?; // label
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
        },

        NonTerminalSymbol(NonTerm::Tag) => match input {
            // Production: tag = %tag value
            Term::Tag(tag) => {
                cxt_stack.push_action("tag_set", tag_set(tag))?;
                cxt_stack.push_non_term(NonTerm::Value)?;
                cxt_stack.push_kind(Kind::Tag)?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
            // Error
            _ => {
                cxt_stack.push_non_term(NonTerm::Tag)?;
                Err(Error::UnexpectedT(vec![Kind::Tag], input))
            }
        },

        NonTerminalSymbol(NonTerm::Tstr) => match input {
            // Production: tstr = %tstr(payload)
            Term::Tstr(_) => {
                cxt_stack.push_kind(Kind::Tstr)?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
            // Production: tstr = %tstrx tstrxseq
            Term::TstrX => {
                cxt_stack.push_non_term(NonTerm::TstrXSeq)?;
                cxt_stack.push_kind(Kind::TstrX)?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
            // Error
            _ => {
                cxt_stack.push_non_term(NonTerm::Tstr)?;
                Err(Error::UnexpectedT(vec![Kind::Tstr, Kind::TstrX], input))
            }
        },

        NonTerminalSymbol(NonTerm::TstrXSeq) => match input {
            // Production: tstrxseq = %break
            Term::Break => {
                cxt_stack.push_kind(Kind::Break)?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
            // Production: tstrxseq = tstr tstrxseq
            Term::Tstr(_) | Term::TstrX => {
                cxt_stack.push_non_term(NonTerm::TstrXSeq)?;
                cxt_stack.push_action("append_tstr", tstr_append())?;
                cxt_stack.push_non_term(NonTerm::Tstr)?;
                do_consume(parse_visitor, val_stack, cxt_stack, input)
            }
            // Error
            _ => {
                cxt_stack.push_non_term(NonTerm::TstrXSeq)?;
                Err(Error::UnexpectedT(
                    vec![Kind::Break, Kind::Tstr, Kind::TstrX],
                    input,
                ))
            }
        },
    }
}

////////////////////////////////////////////////////////////////////////////////
//   Action utilities
////////////////////////////////////////////////////////////////////////////////

fn array_collect(n: u64) -> impl Fn(&mut ContextStack, &mut ValueStack) {
    move |_: &mut ContextStack, val_stack: &mut ValueStack| {
        val_stack.do_array_collect(n)
    }
}

fn array_push() -> impl Fn(&mut ContextStack, &mut ValueStack) {
    move |_: &mut ContextStack, val_stack: &mut ValueStack| {
        val_stack.do_array_push()
    }
}

fn bstr_append() -> impl Fn(&mut ContextStack, &mut ValueStack) {
    move |_: &mut ContextStack, val_stack: &mut ValueStack| {
        val_stack.do_bstr_append()
    }
}

fn map_collect(n: u64) -> impl Fn(&mut ContextStack, &mut ValueStack) {
    move |_: &mut ContextStack, val_stack: &mut ValueStack| {
        val_stack.do_map_collect(n)
    }
}

fn map_push() -> impl Fn(&mut ContextStack, &mut ValueStack) {
    move |_: &mut ContextStack, val_stack: &mut ValueStack| {
        val_stack.do_map_push()
    }
}

fn tag_set(tag: u64) -> impl Fn(&mut ContextStack, &mut ValueStack) {
    move |_: &mut ContextStack, val_stack: &mut ValueStack| {
        val_stack.do_tag_set(tag)
    }
}

fn tstr_append() -> impl Fn(&mut ContextStack, &mut ValueStack) {
    move |_: &mut ContextStack, val_stack: &mut ValueStack| {
        val_stack.do_tstr_append()
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod more_tests;
