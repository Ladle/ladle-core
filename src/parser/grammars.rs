// use std::collections::HashMap;

/// A Context Free Grammar
#[derive(Debug, Clone)]
pub struct CFG<T> {
    /// The rules that make up this grammar
    rules: Vec<CFGRule<T>>
}

/// A rule from a Context Free Grammar
#[derive(Debug, Clone)]
pub struct CFGRule<T> {
    /// The node type produced by this rules successsful application
    result: T,
    /// Uniquely identifies this rule among rules that produce the same result 
    variant: usize,
    /// The nodes that must appear in sequence for this rule to be applied
    nodes: Vec<T>
}

impl<T> CFG<T> {
    pub fn as_mid_grammar(&self) -> Vec<MidRule<T>> {
        let mut mid_rules = Vec::new();

        for cfg_rule in self.rules.iter() {
            cfg_rule.append_mid_rules(&mut mid_rules);
        }

        mid_rules
    }
}

impl<T> CFGRule<T> {
    pub fn append_mid_rules(&self, mid_rules: &mut Vec<MidRule<T>>) {

    }
}
/// A rule for a Middle-Node Grammar
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidRule<T> {
    /// The node type produced by this rules successsful application
    pub result: T,
    /// Uniquely identifies this rule among rules that produce the same result 
    pub variant: usize,
    /// The node type that this rule begins from
    pub base: T,
    /// The nodes that have to occur before the base for the rule to be applied
    pub predecessors: Vec<T>,
    /// The nodes that have to occur after the base for the rule to be applied
    pub successors: Vec<T>
}

impl<T> MidRule<T> {
    pub fn base(&self) -> &T {
        &self.base
    }
}
