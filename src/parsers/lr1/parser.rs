use crate::parsers::{ Term, NonTerm };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct State(usize);

struct LRParser {
    /// The terminal indexes for the input string
    input: Vec<Term>,
    /// The index of the next terminal to read
    input_index: usize,

    state_stack: Vec<State>,
    forest: Vec<Tree>
}

enum Tree {
    Branch {
        /// The value (non-terminal) associated with this branch of the tree
        val: NonTerm,
        /// The children of this node
        children: Vec<Tree>
    },
    Leaf {
        /// The value (terminal) associated with this leaf of the tree
        val: Term
    }
}
