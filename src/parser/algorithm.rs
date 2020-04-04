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
    tasks: VecDeque<Task>
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
        let queue = VecDeque::with_capacity(tokens.len() * 2);

        let mut state = State { rules, rule_map, table, nodes, queue };

        for (i, token) in tokens.iter().enumerate() {
            let node_idx = state.add_terminal(*token, i);
            // TODO: Add optimization
            // Only queue nodes which are part of rules containing only terminals
            // Initially these are the only nodes that can produce yield results anyway
            // All nodes that become usable can be reached by enqueuing produced nodes as normal
            state.queue.push_back(node_idx);
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

    pub fn run(&mut self) {
        while let Some(next_task) = self.tasks.pop_front() {
            match next_task {
                Task::NodeCheck { node } => {
                    self.node_check(node);
                },
                Task::RightCheck { rule_idx, right_pos, leftmost, rightmost } => {
                    self.right_check(rule_idx, right_pos, leftmost, rightmost);
                },
                Task::LeftCheck { rule_idx, left_pos, leftmost, rightmost } => {
                    self.left_check(rule_idx, left_pos, leftmost, rightmost);
                }
            }
        }
    }

    fn node_check(&mut self, node_idx: NodeIdx) {
        let base_node = self.get_node(node_idx);

        if let Some(rule_indices) = self.rule_map.get(&base_node.label) {
            for rule_idx in rule_indices {
                let rule = self.get_rule(*rule_idx);
                let has_next = rule.successors.len() != 0;

                self.tasks.push_back(Task::RightCheck {
                    rule_idx: *rule_idx,
                    right_pos: 0,
                    leftmost: base_node.start,
                    rightmost: base_node.stop
                });
            }
        }

    }

    fn right_check(&mut self,
            rule_idx: RuleIdx, right_pos: usize,
            leftmost: TableIdx, rightmost: TableIdx) {

        let rule = self.get_rule(rule_idx);

        if right_pos == 0 {
            if rule.successors.is_empty() {
                self.tasks.push_back(Task::LeftCheck {
                    rule_idx,
                    left_pos: 0,
                    leftmost, rightmost 
                });
            } else {
                self.tasks.push_back(Task::RightCheck {
                    rule_idx,
                    left_pos: 1,
                    leftmost, rightmost 
                });
            }
        } else {

        }
    }

    fn left_check(&mut self,
            rule_idx: RuleIdx, left_pos: usize,
            leftmost: TableIdx, rightmost: TableIdx) {

        if left_pos == 0 {

        } else {
            
        }
    }

    pub fn check_node(&mut self, base_idx: NodeIdx) {
        let base_node = self.get_node(base_idx);

        let (mut right_checks, mut left_checks) = self.build_checks(base_node);
        self.perform_right_checks(base_node, &mut right_checks, &mut left_checks);
        self.perform_left_checks(base_node, &mut left_checks);
    }

    // For a given label
    fn build_checks(&self, base_node: &Node<T>) -> (VecDeque<RightCheck>, VecDeque<LeftCheck>) {
        let table_next = self.next_table_idx(base_node.stop);
        let table_prev = self.prev_table_idx(base_node.start);

        if let Some(rule_indices) = self.rule_map.get(&base_node.label) {
            for rule_idx in rule_indices {
                let rule = self.get_rule(*rule_idx);
                let has_next = rule.successors.len() != 0;
                let has_prev = rule.predecessors.len() != 0;

                match (has_next, table_next, has_prev, table_prev) {
                    (true, Some(table_n), _, _) => {
                        right_checks.push_back(RightCheck {
                            rule_idx: *rule_idx,
                            rule_pos: 0,
                            table_pos: table_n
                        })
                    },
                    (_, _, true, Some(table_p)) => {
                        left_checks.push_back(LeftCheck {
                            rule_idx: *rule_idx,
                            rightmost_extent: base_node.stop,
                            rule_pos: 0,
                            table_pos: table_p
                        })
                    },
                    _ => {}
                }
            }
        }

        return (right_checks, left_checks);
    }

    fn perform_right_checks(&self, base_node: &Node<T>,
                right_checks: &mut VecDeque<RightCheck>,
                left_checks: &mut VecDeque<LeftCheck>) {

        while let Some(next_check) = right_checks.pop_front() {
            let current_rule = self.get_rule(next_check.rule_idx);
            let expected = current_rule.successors[next_check.rule_pos];
            
            for suc_idx in self.get_table_entry(next_check.table_pos).started.iter() {
                // Check whether the found label matches the expected one
                if self.get_node(*suc_idx).label == expected {
                    // Check whether the right-check is done
                    if next_check.rule_pos + 1 == current_rule.successors.len() {
                        // If the right-check is done, create a left-check
                        left_checks.push_back(LeftCheck {
                            rule_idx: next_check.rule_idx,
                            rightmost_extent: next_check.table_pos,
                            rule_pos: 0,
                            table_pos: 1,
                        });
                    } else {
                        // Increment idx and fail if we would overrun the table
                        if let Some(next_table_idx) = self.next_table_idx(next_check.table_pos) {
                            right_checks.push_back(RightCheck {
                                rule_idx: next_check.rule_idx,
                                rule_pos: next_check.rule_pos + 1,
                                table_pos: next_table_idx,
                            });
                        }
                    }
                }
            }
        }
    }

    fn perform_left_checks(&mut self, left_checks: &mut VecDeque<LeftCheck>) {
        while let Some(next_check) = left_checks.pop_front() {
            let current_rule = self.get_rule(next_check.rule_idx);

            let expected = current_rule.successors[next_check.rule_pos];
            
            for suc_idx in self.get_table_entry(next_check.table_pos).terminated.iter() {
                if self.get_node(*suc_idx).label == expected {
                    if next_check.rule_pos + 1 == current_rule.predecessors.len() {
                        // matched
                    } else {
                        left_checks.push_back(LeftCheck {
                            rule_idx: next_check.rule_idx,
                            rightmost_extent: next_check.rightmost_extent,
                            rule_pos: next_check.rule_pos + 1,
                            table_pos: TableIdx(next_check.table_pos.0 - 1),
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

    pub fn run_till_done(&mut self) {
        while let Some(next_check) = self.queue.pop_front() {
            self.check_node(next_check);
        }
    }

    pub fn get_parsed_trees() {

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

enum Task {
    NodeCheck {
        node: NodeIdx
    },

    RightCheck {
        /// The current rule being examined
        rule_idx: RuleIdx,
        ///
        right_pos: usize,
        
        leftmost: TableIdx,
        rightmost: TableIdx
    },

    LeftCheck {
        /// The current rule being examined
        rule_idx: RuleIdx,
        /// The index of the next expected token in the predecessor list
        left_pos: usize,

        leftmost: TableIdx,
        rightmost: TableIdx
    }
}

enum CheckStage {
    Init, Right, Left, Done
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
