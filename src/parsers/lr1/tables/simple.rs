use std::collections::{ HashSet, HashMap, BTreeSet, VecDeque };

use crate::parsers::{ CFG, CFGProduction, Symbol, Term, NonTerm };
use crate::parsers::lr1::tables::{ LRTransition, ParseAction, EndParseAction };

/// A simple LRTransition that stores its data in
/// uncompressed sparse tables.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimpleTransition {
    /// The parse actions table.
    /// The first level of indices represents state.
    /// The second level of indices represents input terminals.
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

use std::convert::TryFrom;

impl TryFrom<CFG> for SimpleTransition {
    type Error = ();

    fn try_from(cfg: CFG) -> Result<Self, ()> {
        let mut helper = SimpleTransitionHelper::new(cfg);
        helper.push_initial_stage();
        helper.compute_stages();
        helper.export_simple_transition()
    }
}

struct SimpleTransitionHelper {
    /// The set of symbols produced by the grammar
    potential_symbols: HashSet<Symbol>,
    /// The GOTO table for entries (stage, non-terminal) -> stage
    non_terminal_stages: Vec<Vec<Option<usize>>>,
    /// The number of columns in the non_terminal_stages table
    non_terminal_cols: usize,
    /// The GOTO table for entries (stage, terminal) -> stage
    terminal_stages: Vec<Vec<Option<usize>>>,
    /// The number of columns in the terminal_stages table
    terminal_cols: usize,
    /// The extended productions representing the grammar
    /// and the P
    productions: Vec<ExtendedProduction>,
    /// Represents the stages that have been found
    /// Each index in the stages vector represent the stage id
    stages: Vec<Stage>,
    /// Maps a stage to its id,
    /// so that we can check if a newly created stage already exists
    known_stages: HashMap<Stage, usize>,
    /// The stage ids that still need to be expanded
    stage_queue: VecDeque<usize>
}

impl SimpleTransitionHelper {
    fn new(cfg: CFG) -> Self {
        // Compute the set of used symbols
        let potential_symbols: HashSet<Symbol> =
            cfg.rules.iter().flat_map(|rule| rule.right.iter()).map(|a|*a).collect();

        let non_terminal_cols = non_terminal_cols(&potential_symbols);
        let terminal_cols = terminal_cols(&potential_symbols);

        SimpleTransitionHelper {
            potential_symbols,
            non_terminal_stages: Vec::new(),
            non_terminal_cols,
            terminal_stages: Vec::new(),
            terminal_cols,
            productions: convert_productions(cfg.rules, cfg.start_symbol),
            stages: Vec::new(),
            known_stages: HashMap::new(),
            stage_queue: VecDeque::new()
        }
    }

    fn push_initial_stage(&mut self) {
        let initial_item = Item { production: self.productions[0].clone(), position: 0 };
        let mut initial_items = BTreeSet::new();
        initial_items.insert(initial_item);

        let initial_stage = expand_stage(Stage { items: initial_items }, &self.productions);

        self.stages.push(initial_stage.clone());
        self.non_terminal_stages.push(vec![None; self.non_terminal_cols]);
        self.terminal_stages.push(vec![None; self.terminal_cols]);
        self.known_stages.insert(initial_stage, 0);
        self.stage_queue.push_back(0);
    }

    fn compute_stages(&mut self) {
        while let Some(source_idx) = self.stage_queue.pop_front() {
            let source_stage = self.stages[source_idx].clone();

            for next_symbol in self.potential_symbols.iter() {
                let dest_stage = source_stage.clone();
                let dest_stage = apply_symbol(dest_stage, *next_symbol);
                let dest_stage = expand_stage(dest_stage, &self.productions);

                let dest_idx = if let Some(existing_idx) = self.known_stages.get(&dest_stage) {
                    *existing_idx
                } else {
                    let idx = self.stages.len();

                    self.stages.push(dest_stage.clone());
                    self.non_terminal_stages.push(vec![None; self.non_terminal_cols]);
                    self.terminal_stages.push(vec![None; self.terminal_cols]);
                    self.known_stages.insert(dest_stage, idx);
                    self.stage_queue.push_back(idx);

                    idx
                };

                match next_symbol {
                    Symbol::NonTerminal { val } => {
                        self.non_terminal_stages[source_idx][val.0] = Some(dest_idx);
                    },
                    Symbol::Terminal { val } => {
                        self.terminal_stages[source_idx][val.0] = Some(dest_idx);
                    }
                }
            }
        }
    }

    fn export_simple_transition(self) -> Result<SimpleTransition, ()> {
        fn row_map(row: Vec<Option<usize>>) -> Vec<Option<SimpleState>> {
            row.into_iter().map(|stage| stage.map(|s| SimpleState(s))).collect()
        }

        let _non_terminal_states: Vec<Vec<Option<SimpleState>>> =
                self.non_terminal_stages.into_iter().map(row_map).collect();
        let _terminal_states: Vec<Vec<Option<SimpleState>>> =
                self.terminal_stages.into_iter().map(row_map).collect();

        unimplemented!()
    }
}

fn convert_productions(productions: Vec<CFGProduction>, start_symbol: NonTerm) -> Vec<ExtendedProduction> {
    let mut new_productions = vec![ExtendedProduction {
        left: LRLeft::Accept,
        right: vec![start_symbol.into()]
    }];
    new_productions.extend(productions.into_iter().map(|cfg_prod| 
        ExtendedProduction {
            left: LRLeft::NonTerminal { val: cfg_prod.left },
            right: cfg_prod.right
        })
    );
    new_productions
}

fn non_terminal_cols(potential_symbols: &HashSet<Symbol>) -> usize {
    if let Some(max_non_terminal) = 
            potential_symbols.iter().filter_map(|sym| match sym {

        Symbol::NonTerminal { val } => Some(val),
        Symbol::Terminal { val: _ } => None
    }).max() {
        max_non_terminal.0 + 1
    } else {
        0
    }
}

fn terminal_cols(potential_symbols: &HashSet<Symbol>) -> usize {
    if let Some(max_terminal) = 
            potential_symbols.iter().filter_map(|sym| match sym {

        Symbol::NonTerminal { val: _ } => None,
        Symbol::Terminal { val } => Some(val)
    }).max() {
        max_terminal.0 + 1
    } else {
        0
    }
}

fn apply_symbol(_stage: Stage, _symbol: Symbol) -> Stage {
    unimplemented!()
}

fn expand_stage(_stage: Stage, _productions: &[ExtendedProduction]) -> Stage {
    unimplemented!()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Stage {
    items: BTreeSet<Item>
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Item {
    production: ExtendedProduction,
    position: usize
}

/// A extension of CFGProduction to allow a mapping from
/// the ACCEPT pseudo-symbol to the initial symbol.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ExtendedProduction {
    left: LRLeft,
    right: Vec<Symbol>
}

/// An extension of the NonTerm nonterminal symbol type
/// to allow it to include teh ACCEPT pseudo-symbol.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum LRLeft {
    NonTerminal {
        val: NonTerm
    },
    Accept
}
