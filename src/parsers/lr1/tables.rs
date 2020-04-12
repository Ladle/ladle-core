use crate::parsers::{ CFG, NonTerm };

/// A type that holds the state and parse action tables
/// for an LR(1) parser
pub struct LRTables {
    
}

impl LRTables {
    
}

impl From<CFG> for LRTables {
    fn from(_cfg: CFG) -> Self {
        unimplemented!()
    }
}

/// The Parse Actions for an LR(1) parser
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

/// 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State(usize);
