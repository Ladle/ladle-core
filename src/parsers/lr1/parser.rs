use crate::parsers::{ Term, NonTerm };
use crate::parsers::trees::BoxTree;

use super::tables::{ LRTables };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct State(usize);

pub struct LRParser {
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
    pub fn new(tables: LRTables, input: Vec<Term>) -> Self {
        LRParser {
            tables,
            input,
            input_index: 0,
            state_stack: Vec::new(),
            forest: Vec::new()
        }
    }

    pub fn done(&self) -> bool {
        self.input_index == self.input.len()
    }

    pub fn execute(&mut self) {

    }

    pub fn execute_step(&mut self) {

    }

    pub fn to_output(mut self) -> Option<BoxTree> {
        if self.done() && self.forest.len() == 1 {
            Some(self.forest.remove(0))
        } else {
            None
        }
    }
}
