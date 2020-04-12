pub mod lr1;
pub mod glr;

/// A Context Free Grammar,
/// logically it is a set of productions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CFG {
    /// Each index in the outer table represents a non-terminal
    /// The value of that cell is the list of productions
    /// which can be made from that non-terminal
    indexed_rules: Vec<Vec<CFGProduction>>
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Symbol {
    Terminal { val: Term },
    NonTerminal { val: NonTerm }
}

/// A terminal symbol in a grammar
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Term(usize);

/// A non-terminal symbol in a grammar
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NonTerm(usize);

use std::cmp::max;

impl From<Vec<CFGProduction>> for CFG {
    fn from(productions: Vec<CFGProduction>) -> Self {
        let mut max_index = 0;
        for prod in productions.iter() {
            max_index = max(prod.left.0, max_index);
        }
        
        let mut indexed_rules = Vec::new();
        for _i in 0..max_index+1 {
            indexed_rules.push(Vec::new());
        }

        for prod in productions.iter() {
            indexed_rules[prod.left.0].push(prod.clone());
        }

        CFG { indexed_rules }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn productions1() -> Vec<CFGProduction> {
        vec![
            CFGProduction { left: NonTerm(0), right: vec![
                Symbol::NonTerminal { val: NonTerm(1) },
                Symbol::NonTerminal { val: NonTerm(1) }
            ]},
            CFGProduction { left: NonTerm(0), right: vec![
                Symbol::NonTerminal { val: NonTerm(1) }
            ]},

            CFGProduction { left: NonTerm(1), right: vec![
                Symbol::NonTerminal { val: NonTerm(2) },
                Symbol::NonTerminal { val: NonTerm(2) }
            ]},
            CFGProduction { left: NonTerm(1), right: vec![
                Symbol::NonTerminal { val: NonTerm(2) }
            ]},

            CFGProduction { left: NonTerm(2), right: vec![
                Symbol::Terminal { val: Term(0) },
                Symbol::Terminal { val: Term(0) }
            ]},
            CFGProduction { left: NonTerm(2), right: vec![
                Symbol::Terminal { val: Term(0) }
            ]},
        ]
    }

    fn grammar1() -> CFG {
        let productions = productions1();

        CFG {
            indexed_rules: vec![
                vec![ productions[0].clone(), productions[1].clone() ],
                vec![ productions[2].clone(), productions[3].clone() ],
                vec![ productions[4].clone(), productions[5].clone() ]
            ]
        }
    }

    #[test]
    fn cfg_from_productions() {
        let productions = productions1();
        let cfg = CFG::from(productions.clone());
        assert_eq!(cfg, grammar1());
    }
}
