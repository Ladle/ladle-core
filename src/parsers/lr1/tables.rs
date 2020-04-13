use crate::parsers::{ CFG, Symbol, NonTerm, Term };

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

/// A simple LRTransition that stores its data in
/// uncompressed sparse tables.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimpleTransition {
    /// The parse actions table.
    /// The first level of indices represents state.
    /// The second level of indices represents input.
    input_actions: Vec<Vec<ParseAction>>,

    /// The parse actions associated with the end of the input.
    /// there is no input to take in.
    /// The indices of this Vec represent state.
    end_actions: Vec<EndParseAction>,

    /// The state transition tables for non-terminals.
    /// The first level of indices represents state.
    /// The second level of indices represents the root non-terminal.
    non_terminal_states: Vec<Vec<Option<SimpleState>>>,

    /// The state transition tables for terminals
    /// The first level of indices represents state.
    /// The second level of indices represents the root terminal.
    terminal_states: Vec<Vec<Option<SimpleState>>>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimpleState(usize);

impl LRTransition for SimpleTransition {
    type State = SimpleState;
    
    fn initial_state() -> SimpleState {
        SimpleState(0)
    }

    fn get_action(&self, state: SimpleState, next: Term) -> ParseAction {
        let index_outer = state.0;
        let index_inner = next.0;
        self.input_actions[index_outer][index_inner]
    }

    fn get_action_end(&self, state: SimpleState) -> EndParseAction {
        let index = state.0;
        self.end_actions[index]
    }

    fn get_state(&self, state: SimpleState, right_most: Symbol) -> Option<SimpleState> {
        let index_outer = state.0;
        match right_most {
            Symbol::Terminal { val } => {
                let index_inner = val.0;
                self.terminal_states[index_outer][index_inner]
            },
            Symbol::NonTerminal { val } => {
                let index_inner = val.0;
                self.terminal_states[index_outer][index_inner]
            }
        }
    }
}

impl From<CFG> for SimpleTransition {
    fn from(_cfg: CFG) -> Self {
        unimplemented!()
    }
}

/// A parse action that an LR(1) parser can take
/// at a given step of the parse algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseAction {
    /// Error Action
    /// The parser fails and emits an error
    Error,

    /// Shift Action
    /// The parser takes the next input in
    /// and creates a tree in the forest for it 
    Shift,

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

/// A parse action that an LR(1) parser can take
/// at a given step of the parse algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndParseAction {
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
