/// The LR(1) parser module
pub mod lr1;
/// The GLR parser module
pub mod glr;

/// A Context Free Grammar,
/// logically it is a set of productions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CFG {
    pub start_symbol: NonTerm,
    /// The productions which make up the grammar
    pub rules: Vec<CFGProduction>
}

/// A production in a context free grammar.
/// Each production asserts that the right symbols
/// can be produced from the left non-terminal.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CFGProduction {
    /// The non-terminal symbol's index
    pub left: NonTerm,
    /// The symbols produced by this production
    pub right: Vec<Symbol>
}

/// A symbol, either terminal or non-terminal
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Symbol {
    /// A terminal symbol
    Terminal { val: Term },
    /// A non-terminal symbol
    NonTerminal { val: NonTerm }
}

/// A terminal symbol in a grammar
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Term(usize);

/// A non-terminal symbol in a grammar
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonTerm(usize);

impl From<Term> for Symbol {
    fn from(term: Term) -> Self {
        Symbol::Terminal { val: term }
    }
}

impl From<NonTerm> for Symbol {
    fn from(nonterm: NonTerm) -> Self {
        Symbol::NonTerminal { val: nonterm }
    }
}
