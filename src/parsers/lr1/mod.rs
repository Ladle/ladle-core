/// The parser module, which contains the required logic to perform parsing
pub mod parser;
/// The tables module, which contains representations and logic for
/// the LR(1) transitions and how to create them from a grammar 
pub mod tables;

use super::{ CFG, NonTerm, Term };
use crate::trees::BoxTree;

use parser::LRParser;
use tables::simple::SimpleTransition;

use std::convert::TryFrom;

/// Use a Context Free Grammar to parse a string of Terminals
/// and create a Parse Tree
pub fn parse_simple(cfg: CFG, input: Vec<Term>) -> Option<BoxTree<NonTerm, Term>> {
    let tables: SimpleTransition = SimpleTransition::try_from(cfg).ok()?;
    let mut parser = LRParser::new(&tables, input);
    parser.execute();
    parser.to_output()
}
