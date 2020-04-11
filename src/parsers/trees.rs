use super::{ NonTerm, Term };

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoxTree {
    Branch {
        /// The value (non-terminal) associated with this branch of the tree
        val: NonTerm,
        /// The children of this node
        children: Vec<BoxTree>
    },
    Leaf {
        /// The value (terminal) associated with this leaf of the tree
        val: Term
    }
}

use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RcTree {
    Branch {
        /// The value (non-terminal) associated with this branch of the tree
        val: NonTerm,
        /// The children of this node
        children: Rc<Vec<RcTree>>
    },
    Leaf {
        /// The value (terminal) associated with this leaf of the tree
        val: Term
    }
}

use std::borrow::Borrow;

impl From<RcTree> for BoxTree {
    fn from(rc_tree: RcTree) -> Self {
        match rc_tree {
            RcTree::Branch { val, children } => {
                let children: &Vec<RcTree> = children.borrow();
                let children: Vec<BoxTree> = children.clone().into_iter()
                    .map(BoxTree::from).collect();

                BoxTree::Branch { val, children }
            },
            RcTree::Leaf { val } => BoxTree::Leaf { val }
        }
    }
}


impl From<BoxTree> for RcTree {
    fn from(box_tree: BoxTree) -> Self {
        match box_tree {
            BoxTree::Branch { val, children } => {
                let children: &Vec<BoxTree> = children.borrow();
                let children: Vec<RcTree> = children.clone().into_iter()
                    .map(RcTree::from).collect();
                let children = Rc::new(children);
                
                RcTree::Branch { val, children }
            },
            BoxTree::Leaf { val } => RcTree::Leaf { val }
        }
    }
}
