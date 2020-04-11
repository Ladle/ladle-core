use std::hash::Hash;
use std::collections::{ HashMap, VecDeque };

use super::grammars::{ MidRule };


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

impl<T> Node<T> {
    fn new_non_terminal(label: T, start: TableIdx, stop: TableIdx, rule: RuleIdx, children: Vec<NodeIdx>) -> Self {
        Node {
            label, start, stop,
            meta: NodeMeta::NonTerminal {
                rule, children
            }
        }
    }
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
            let node_idx = state.add_node(Node {
                label: *token,
                start: TableIdx(i),
                stop:  TableIdx(i + 1),
                meta:  NodeMeta::Terminal {
                    token_idx: i
                }
            });
            // TODO: Add optimization
            // Only queue nodes which are part of rules containing only terminals
            // Initially these are the only nodes that can produce yield results anyway
            // All nodes that become usable can be reached by enqueuing produced nodes as normal
            state.node_queue.push_back(node_idx);
        }

        state
    }

    fn add_node(&mut self, node: Node<T>) -> NodeIdx {
        let node_idx = NodeIdx(self.nodes.len());

        self.table[node.start.0].started.push(node_idx);
        self.table[node.stop.0].terminated.push(node_idx);

        self.nodes.push(node);

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
        let mut new_nodes = Vec::new();
        while let Some(node_idx) = self.node_queue.pop_front() {
            self.check_node(node_idx, &mut new_nodes)
        }

        while let Some(check) = self.check_queue.pop_front() {
            match check.stage {
                CheckStage::Right => self.check_right(check, &mut new_nodes),
                CheckStage::Left => self.check_left(check, &mut new_nodes)
            }
        }

        for new_node in new_nodes.into_iter() {
            let node_idx = self.add_node(new_node);
            self.node_queue.push_back(node_idx);
        }
    }

    fn check_node(&mut self, node_idx: NodeIdx, new_nodes: &mut Vec<Node<T>>) {
        let base_node = self.get_node(node_idx);
        let leftmost = base_node.start;
        let rightmost = base_node.stop;

        if let Some(rule_indices) = self.rule_map.get(&base_node.label) {
            for rule_idx in rule_indices {
                let rule = self.get_rule(*rule_idx);

                if rule.successors.is_empty() && rule.predecessors.is_empty() {
                    let result = rule.result;

                    new_nodes.push(Node::new_non_terminal(
                        result, leftmost, rightmost,
                        *rule_idx, vec![node_idx]));
                } else {
                    let stage = if rule.successors.is_empty() {
                        CheckStage::Left
                    } else {
                        CheckStage::Right
                    };

                    self.check_queue.push_back(Check {
                        rule_idx: *rule_idx,
                        stage,
    
                        pos: 0,
                        leftmost, rightmost,
    
                        base: node_idx,
                        right_nodes: Vec::new(),
                        left_nodes: Vec::new()
                    });
                }
            }
        }
    }

    fn check_right(&mut self, check: Check, new_nodes: &mut Vec<Node<T>>) {
        let rule = self.get_rule(check.rule_idx);
        let rule_suc_len = rule.successors.len();
        let rule_pred_len = rule.predecessors.len();
        let result = rule.result;
        let expected = rule.successors[check.pos];
        
        for suc_idx in self.get_table_entry(check.rightmost).started.clone().iter() {
            let current_node = self.get_node(*suc_idx);
            // Check whether the found label matches the expected one
            if current_node.label == expected {
                let mut new_check = check.clone();
                new_check.rightmost = current_node.stop;
                new_check.right_nodes.push(*suc_idx);

                // Check whether the right-check is done
                if check.pos + 1 == rule_suc_len {
                    if rule_pred_len == 0 {
                        new_nodes.push(Node::new_non_terminal(
                            result, check.leftmost, check.rightmost,
                            check.rule_idx, check.right_nodes.clone()));
                    } else {
                        new_check.stage = CheckStage::Left;
                        new_check.pos = 0;
                        self.check_queue.push_back(new_check);
                    }
                } else {
                    new_check.pos += 1;
                    new_check.rightmost = current_node.stop;
                    self.check_queue.push_back(new_check);
                }
            }
        }
    }

    fn check_left(&mut self, check: Check, new_nodes: &mut Vec<Node<T>>) {
        let rule = self.get_rule(check.rule_idx);

        let rule_pred_len = rule.predecessors.len();
        let result = rule.result;
        let expected = rule.predecessors[check.pos];
        
        for suc_idx in self.get_table_entry(check.rightmost).terminated.clone().iter() {
            let current_node = self.get_node(*suc_idx);
            if current_node.label == expected {
                if check.pos + 1 == rule_pred_len {
                    // reverse iterate the left_nodes, then iterate the right_nodes, then deref, then make a vector
                    let children = check.left_nodes.iter().rev().chain(check.right_nodes.iter()).map(|a| *a).collect();
                    let node = Node {
                        label: result,
                        start: check.leftmost,
                        stop: check.rightmost,
                        meta: NodeMeta::NonTerminal {
                            rule: check.rule_idx,
                            children
                        }
                    };
                    new_nodes.push(node);
                } else {
                    let mut new_check = check.clone();
                    new_check.pos += 1;
                    new_check.leftmost = current_node.start;
                    new_check.left_nodes.push(*suc_idx);

                    self.check_queue.push_back(new_check);
                }
            }
        }
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
    Right, Left
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
