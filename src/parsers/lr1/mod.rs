pub mod parser;
pub mod tables;

use super::{ CFG, Term };
use super::trees::BoxTree;

use parser::LRParser;
use tables::LRTables;

pub fn parse(cfg: CFG, input: Vec<Term>) -> Option<BoxTree> {
    let tables: LRTables = cfg.into();
    let mut parser = LRParser::new(tables, input);
    parser.execute();
    parser.to_output()
}
