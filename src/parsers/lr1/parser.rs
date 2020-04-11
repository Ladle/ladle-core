use crate::parsers::{ Term, NonTerm };
use crate::parsers::trees::BoxTree;

use super::tables::{ LRTables };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct State(usize);

struct LRParser {
    // The Parse action and state transition tables
    tables: LRTables,

    /// The terminal indexes for the input string
    input: Vec<Term>,
    /// The index of the next terminal to read
    input_index: usize,

    /// The stack of tree states
    state_stack: Vec<State>,
    /// The list of current trees
    forest: Vec<BoxTree>
}

impl LRParser {

}
