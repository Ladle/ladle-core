mod lr1;
mod glr;
mod trees;

pub struct CFG {
    /// Each index in the outer table represents a non-terminal
    /// The value of that cell is the list of productions
    /// which can be made from that non-terminal
    indexed_rules: Vec<Vec<CFGProduction>>
}

#[derive(Clone)]
pub struct CFGProduction {
    /// The non-terminal symbol's index
    pub left: NonTerm,
    /// The symbols produced by this production
    pub right: Vec<Symbol>
}

#[derive(Clone)]
pub enum Symbol {
    Terminal { val: Term },
    NonTerminal { val: NonTerm }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Term(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
