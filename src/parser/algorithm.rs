use std::hash::Hash;
use std::collections::{ HashMap, VecDeque };
use crate::parser::grammars::{ MidRule };


// Indexes into Vectors that act as marker types
// indexes into the State.rules Vec
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct RuleIdx(usize);
// indexes into the State.nodes Vec
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct NodeIdx(usize);
// indexes into the State.table Vec
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TableIdx(usize);

/// Parser algorithm state
pub struct State<T>
    where
        T: Hash + Eq {

    /// The Mid-Rule grammar being used for parsing
    rules: Vec<MidRule<T>>,
    rule_map: HashMap<T, Vec<RuleIdx>>,
    /// The table of reference-points into the input
    table: Vec<TableEntry>,
    nodes: Vec<Node<T>>,

    node_queue: VecDeque<NodeIdx>,
    check_queue: VecDeque<Check>
}

#[derive(Debug, Clone)]
struct TableEntry {
    started: Vec<NodeIdx>,
    terminated: Vec<NodeIdx>
}

#[derive(Debug)]
struct Node<T> {
    label: T,
    start: TableIdx,
    stop: TableIdx,
    meta: NodeMeta
}

#[derive(Debug)]
enum NodeMeta {
    Terminal {
        token_idx: usize
    },
    NonTerminal {
        rule: RuleIdx,
        children: Vec<NodeIdx>
    }
}

impl<T> State<T>
    where
        T: Hash + Eq + Clone + Copy {

    fn new(rules: Vec<MidRule<T>>, tokens: Vec<T>) -> Self {
        let rule_map = make_rule_map(&rules[..]);
        // allocate table with exact size that will be needed
        let table = vec![TableEntry::new(); tokens.len() + 1];
        // allocate nodes with twice as much room as minimally needed
        let nodes = Vec::with_capacity(tokens.len() * 2);
        // allocate queue with twice as much room as minimally needed
        let node_queue = VecDeque::with_capacity(tokens.len() * 2);
        // allocate queue for the checks
        let check_queue = VecDeque::with_capacity(tokens.len());

        let mut state = State { rules, rule_map, table, nodes, node_queue, check_queue };

        for (i, token) in tokens.iter().enumerate() {
            let node_idx = state.add_terminal(*token, i);
            // TODO: Add optimization
            // Only queue nodes which are part of rules containing only terminals
            // Initially these are the only nodes that can produce yield results anyway
            // All nodes that become usable can be reached by enqueuing produced nodes as normal
            state.node_queue.push_back(node_idx);
        }

        state
    }

    pub fn add_terminal(&mut self, label: T, token_idx: usize) -> NodeIdx {
        let node_idx = NodeIdx(self.nodes.len());
        let start = TableIdx(token_idx);
        let stop = TableIdx(token_idx + 1);

        let node = Node {
            label, start, stop,
            meta: NodeMeta::Terminal {
                token_idx: token_idx
            }
        };

        self.nodes.push(node);

        self.table[start.0].started.push(node_idx);
        self.table[stop.0].terminated.push(node_idx);

        return node_idx;
    }
    
    pub fn done(&self) -> bool {
        self.node_queue.is_empty()
    }

    pub fn run_till_done(&mut self) {
        while !self.done() {
            self.run_cycle();
        }
    }

    pub fn run_cycle(&mut self) {
        while let Some(node_idx) = self.node_queue.pop_front() {
            self.check_node(node_idx)
        }

        while let Some(check) = self.check_queue.pop_front() {
            match check.stage {
                CheckStage::StartRight => self.check_start_right(check),
                CheckStage::Right => self.check_right(check),
                CheckStage::StartLeft => self.check_start_left(check),
                CheckStage::Left => self.check_left(check)
            }
        }
    }

    fn check_node(&mut self, node_idx: NodeIdx) {
        let base_node = self.get_node(node_idx);
        let leftmost = base_node.start;
        let rightmost = base_node.stop;

        if let Some(rule_indices) = self.rule_map.get(&base_node.label) {
            for rule_idx in rule_indices {
                self.check_queue.push_back(Check {
                    rule_idx: *rule_idx,
                    stage: CheckStage::StartRight,

                    pos: 0,
                    leftmost, rightmost,

                    base: node_idx,
                    right_nodes: Vec::new(),
                    left_nodes: Vec::new()
                });
            }
        }

    }

    fn check_start_right(&mut self, mut check: Check) {
        let rule = self.get_rule(check.rule_idx);
        
        if rule.successors.is_empty() {
            check.stage = CheckStage::StartLeft;
            self.check_queue.push_back(check);
        } else {
            if let Some(next_idx) = self.next_table_idx(check.rightmost) {
                check.stage = CheckStage::Right;
                check.rightmost = next_idx;
                self.check_queue.push_back(check);
            }
        }
    }


    fn check_right(&mut self, check: Check) {
        let rule = self.get_rule(check.rule_idx);
        let rule_suc_len = rule.successors.len();
        let expected = rule.successors[check.pos];
        
        for suc_idx in self.get_table_entry(check.rightmost).started.clone().iter() {
            // Check whether the found label matches the expected one
            if self.get_node(*suc_idx).label == expected {
                let new_left_nodes = check.left_nodes.clone();
                let mut new_right_nodes = check.right_nodes.clone();
                new_right_nodes.push(*suc_idx);

                // Check whether the right-check is done
                if check.pos + 1 == rule_suc_len {
                    // If the right-check is done, create a left-check
                    self.check_queue.push_back(Check {
                        rule_idx: check.rule_idx,
                        stage: CheckStage::StartLeft,

                        pos: 0,
                        leftmost: check.leftmost,
                        rightmost: check.rightmost,

                        base: check.base,
                        left_nodes: new_left_nodes,
                        right_nodes: new_right_nodes
                    });
                } else {
                    // Increment idx, only proceeding if we don't overrun the table
                    if let Some(next_rightmost) = self.next_table_idx(check.rightmost) {
                        self.check_queue.push_back(Check {
                            rule_idx: check.rule_idx,
                            stage: CheckStage::Right,

                            pos: check.pos + 1,
                            leftmost: check.leftmost,
                            rightmost: next_rightmost,

                            base: check.base,
                            left_nodes: new_left_nodes,
                            right_nodes: new_right_nodes
                        });
                    }
                }
            }
        }
    }


    fn check_start_left(&mut self, mut check: Check) {
        let rule = self.get_rule(check.rule_idx);

        if rule.predecessors.is_empty() {
            // reverse iterate the left_nodes, then iterate the right_nodes, then deref, then make a vector
            self.add_non_terminal(rule.result, check.leftmost, check.rightmost, check.rule_idx, check.right_nodes);
        } else {
            if let Some(next_idx) = self.next_table_idx(check.rightmost) {
                check.stage = CheckStage::Right;
                check.rightmost = next_idx;
                self.check_queue.push_back(check);
            }
        }
    }


    fn check_left(&mut self, check: Check) {
        let rule = self.get_rule(check.rule_idx);

        let rule_pred_len = rule.predecessors.len();
        let result = rule.result;
        let expected = rule.predecessors[check.pos];
        
        for suc_idx in self.get_table_entry(check.rightmost).terminated.clone().iter() {
            if self.get_node(*suc_idx).label == expected {
                let mut new_left_nodes = check.left_nodes.clone();
                let new_right_nodes = check.right_nodes.clone();
                new_left_nodes.push(*suc_idx);

                if check.pos + 1 == rule_pred_len {
                    // reverse iterate the left_nodes, then iterate the right_nodes, then deref, then make a vector
                    let children = check.left_nodes.iter().rev().chain(check.right_nodes.iter()).map(|a| *a).collect();
                    self.add_non_terminal(result, check.leftmost, check.rightmost, check.rule_idx, children);
                } else {
                    if let Some(next_leftmost) = self.prev_table_idx(check.leftmost) {
                        self.check_queue.push_back(Check {
                            rule_idx: check.rule_idx,
                            stage: CheckStage::Left,

                            pos: check.pos + 1,
                            leftmost: next_leftmost,
                            rightmost: check.rightmost,

                            base: check.base,
                            left_nodes: new_left_nodes,
                            right_nodes: new_right_nodes
                        });
                    }
                }
            }
        }
    }
    
    pub fn add_non_terminal(&mut self, label: T,
            start: TableIdx, stop: TableIdx,
            rule: RuleIdx, children: Vec<NodeIdx>) -> NodeIdx {

        let node_idx = NodeIdx(self.nodes.len());

        let node = Node {
            label, start, stop,
            meta: NodeMeta::NonTerminal {
                rule, children
            }
        };
        
        self.nodes.push(node);
        
        self.table[start.0].started.push(node_idx);
        self.table[stop.0].terminated.push(node_idx);

        return node_idx;
    }

    pub fn get_parsed_trees() {
        // TODO
    }

    #[inline]
    fn get_node(&self, idx: NodeIdx) -> &Node<T> {
        &self.nodes[idx.0]
    }

    #[inline]
    fn get_rule(&self, idx: RuleIdx) -> &MidRule<T> {
        &self.rules[idx.0]
    }

    #[inline]
    fn get_table_entry(&self, idx: TableIdx) -> &TableEntry {
        &self.table[idx.0]
    }

    #[inline]
    fn next_table_idx(&self, idx: TableIdx) -> Option<TableIdx> {
        let next = idx.0 + 1;
        if next < self.table.len() {
            Some(TableIdx(next))
        } else {
            None
        }
    }

    #[inline]
    fn prev_table_idx(&self, idx: TableIdx) -> Option<TableIdx> {
        if idx.0 > 0 {
            Some(TableIdx(idx.0 - 1))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
struct Check {
    /// The current rule being examined
    rule_idx: RuleIdx,

    stage: CheckStage,
    ///
    pos: usize,
    
    leftmost: TableIdx,
    rightmost: TableIdx,

    base: NodeIdx,
    right_nodes: Vec<NodeIdx>,
    left_nodes: Vec<NodeIdx>
}

#[derive(Debug, Clone)]
enum CheckStage {
    StartRight, Right, StartLeft, Left
}

impl TableEntry {
    fn new() -> Self {
        TableEntry {
            started: Vec::new(),
            terminated: Vec::new()
        }
    }
}

fn make_rule_map<T>(source: &[MidRule<T>]) -> HashMap<T, Vec<RuleIdx>>
    where
        T: Hash + Eq + Copy {

    let mut rules: HashMap<T, Vec<RuleIdx>> = HashMap::new();

    for (i, rule) in source.iter().enumerate() {
        if let Some(vec) = rules.get_mut(rule.base()) {
            vec.push(RuleIdx(i));
        } else {
            rules.insert(*rule.base(), vec![RuleIdx(i)]);
        }
    }

    rules
}

use std::rc::Rc;

#[derive(Debug)]
pub enum TreeNode<T> {
    Terminal {
        index: usize
    },
    NonTerminal {
        rule: T,
        variant: usize,
        children: Vec<Rc<TreeNode<T>>>
    }
}
