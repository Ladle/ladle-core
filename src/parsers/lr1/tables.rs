pub mod simple;

use crate::parsers::{ Symbol, NonTerm, Term };

/// An LRTransition contains the grammatical information necessary
/// to perform LR(1) Parsing.
/// It contains both the parse action table and state table.
pub trait LRTransition {
    /// The type for representing a state of the parser.
    type State: Copy;

    /// The initial state that an LRParser using this transition system
    /// will have.
    fn initial_state() -> Self::State;

    /// Compute the next action based on the top state of the state stack
    /// and also the next terminal symbol in the input string.
    fn get_action(&self, top_state: Self::State, next: Term) -> ParseAction;

    /// Compute the next action based on the top state of the state stack
    /// when there is no more input left to be parsed.
    fn get_action_end(&self, state: Self::State) -> EndParseAction;
    
    /// Get the next state based on the current state and the symbol
    /// at the root of the right most tree
    fn get_state(&self, state: Self::State, right_most: Symbol) -> Option<Self::State>;
}

/// A parse action that an LR(1) parser can take
/// at a given step of the parse algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseAction {
    /// Accept Action.
    /// The parser accepts the input,
    /// and indicates that the tree in the forest
    /// represents the entire input.
    Accept,

    /// Error Action.
    /// The parser fails and emits an error.
    Error,

    /// Shift Action.
    /// The parser takes the next input in
    /// and creates a tree in the forest for it.
    Shift,

    /// Reduce Action.
    /// Combine the last nodes trees in the tree table
    /// into a single tree labeled `nonterm`.
    Reduce {
        /// The non-terminal to label the new tree with
        nonterm: NonTerm,
        /// The number of nodes to make the tree out of
        nodes: usize
    }
}

/// A parse action that an LR(1) parser can take
/// at a given step of the parse algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndParseAction {
    /// Accept Action.
    /// The parser accepts the input,
    /// and indicates that the tree in the forest
    /// represents the entire input.
    Accept,

    /// Error Action
    /// The parser fails and emits an error
    Error,

    /// Reduce Action
    /// Combine the last nodes trees in the tree table
    /// into a single tree labeled `nonterm`
    Reduce {
        /// The non-terminal to label the new tree with
        nonterm: NonTerm,
        /// The number of nodes to make the tree out of
        nodes: usize
    }
}
