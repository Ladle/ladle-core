use crate::parsers::{ Term, NonTerm, Symbol };
use crate::trees::{ BoxTree, Tree };

use super::tables::{ LRTransition, ParseAction, EndParseAction };

/// An LR(1) parser for a singular input.
/// It contains a reference to an LRTransition,
/// that it uses to perform the parsing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LRParser<'a, T> 
    where
        T: LRTransition {

    /// The Parse action and state transition tables
    transition: &'a T,

    /// The terminal indexes for the input string
    input: Vec<Term>,
    /// The index of the next terminal to read
    input_index: usize,

    /// The stack of tree states
    state_stack: Vec<T::State>,
    /// The list of current trees
    forest: Vec<BoxTree<NonTerm, Term>>,
    /// Whether the parser has failed
    failed: bool,
    /// Whether the parser has finished
    finished: bool
}

impl<'a, T> LRParser<'a, T>
    where
        T: LRTransition {

    /// Create an LRParser
    /// with a set of LRTransition that represent the grammar logic
    /// and a list of input terminals to parse
    pub fn new(transition: &'a T, input: Vec<Term>) -> Self {
        LRParser {
            transition,
            input,
            input_index: 0,
            state_stack: vec![T::initial_state()],
            forest: Vec::new(),
            failed: false,
            finished: false
        }
    }

    /// Execute the parser until completion
    pub fn execute(&mut self) {
        while !self.finished && !self.failed {
            self.execute_step();
        }
    }

    /// Execute one step of the parser
    pub fn execute_step(&mut self) {
        if self.failed || self.finished {
            return;
        }

        if let Some(top_state) = self.state_stack.last() {
            let top_state = *top_state;

            if self.input_index == self.input.len() {
                self.execute_end_action(top_state);
            } else {
                self.execute_action(top_state);
            }

            let right_most = root_as_symbol(self.forest.last().unwrap());

            if let Some(next_state) = self.transition.get_state(top_state, right_most) {
                self.state_stack.push(next_state);
            } else {
                self.failed = true;
                return;
            }
        } else {
            self.failed = true;
            return;
        }        
    }

    fn execute_end_action(&mut self, top_state: T::State) {
        let end_action = self.transition.get_action_end(top_state);

        match end_action {
            EndParseAction::Accept => {
                self.finished = true;
                return;
            }
            EndParseAction::Error => {
                self.failed = true;
                return
            },
            EndParseAction::Reduce { nonterm, nodes } => {
                self.reduce(nonterm, nodes);
            }
        }
    }

    fn execute_action(&mut self, top_state: T::State) {
        let next_input = self.input[self.input_index];
        self.input_index += 1;

        let action = self.transition.get_action(top_state, next_input);

        match action {
            ParseAction::Accept => {
                self.finished = true;
                return;
            },
            ParseAction::Error => {
                self.failed = true;
                return
            },
            ParseAction::Shift => {
                self.forest.push(BoxTree::new_leaf(next_input));
            },
            ParseAction::Reduce { nonterm, nodes } => {
                self.reduce(nonterm, nodes);
            }
        }
    }

    fn reduce(&mut self, nonterm: NonTerm, nodes: usize) {
        let mut children = Vec::new();

        for _i in 0..nodes {
            let tree = self.forest.pop().unwrap();
            let _state = self.state_stack.pop();
            children.push(tree);
        }

        children.reverse();
        let new_tree = BoxTree::new_branch(nonterm, children);

        self.forest.push(new_tree);
    }

    /// Is the LRParser finished parsing
    pub fn finished(&self) -> bool {
        self.finished
    }

    pub fn failed(&self) -> bool {
        self.failed
    }

    /// Extract the output from the parser
    pub fn to_output(mut self) -> Option<BoxTree<NonTerm, Term>> {
        if self.finished && !self.failed {
            Some(self.forest.remove(0))
        } else {
            None
        }
    }
}

fn root_as_symbol(box_tree: &BoxTree<NonTerm, Term>) -> Symbol {
    match box_tree {
        BoxTree::Branch { val, .. } => Symbol::NonTerminal { val: *val },
        BoxTree::Leaf { val } => Symbol::Terminal { val: *val }
    }
}
