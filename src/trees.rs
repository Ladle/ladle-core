use std::slice::Iter;

/// A Tree that has values stored at the 
pub trait Tree<B, L>: Sized {
    /// Create a new leaf node.
    fn new_leaf(val: L) -> Self;
    /// Create a new branch node.
    fn new_branch(val: B, children: Vec<Self>) -> Self;
    /// Iterate through the children of this node.
    fn iter_children(&self) -> Iter<'_, Self>;
    /// Retrieve the branch value if this node is a branch, none otherwise.
    fn branch_val(&self) -> Option<&B>;
    /// Retrieve the leaf value if this node is a leaf, none otherwise. 
    fn leaf_val(&self) -> Option<&L>;
}

/// A heap allocated tree
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BoxTree<B, L> {
    Branch {
        /// The value (non-terminal) associated with this branch of the tree
        val: B,
        /// The children of this node
        children: Vec<BoxTree<B, L>>
    },
    Leaf {
        /// The value (terminal) associated with this leaf of the tree
        val: L
    }
}

impl<B, L> Tree<B, L> for BoxTree<B, L> {
    fn new_leaf(val: L) -> Self {
        BoxTree::Leaf { val }
    }

    fn new_branch(val: B, children: Vec<Self>) -> Self {
        BoxTree::Branch { val, children }
    }

    fn iter_children(&self) -> Iter<'_, BoxTree<B, L>> {
        match self {
            BoxTree::Branch { children, .. } => children.iter(),
            BoxTree::Leaf { .. } => [].iter()
        }
    }

    fn branch_val(&self) -> Option<&B> {
        match self {
            BoxTree::Branch { val, .. } => Some(val),
            BoxTree::Leaf { .. } => None
        }
    }

    fn leaf_val(&self) -> Option<&L> {
        match self {
            BoxTree::Branch { .. } => None,
            BoxTree::Leaf { val } => Some(val)
        }
    }
}

use std::rc::Rc;

/// A Reference Counted Tree, which can be shared and share
/// data with other RcTrees
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RcTree<B, L> {
    Branch {
        /// The value (non-terminal) associated with this branch of the tree
        val: B,
        /// The children of this node
        children: Rc<Vec<RcTree<B, L>>>
    },
    Leaf {
        /// The value (terminal) associated with this leaf of the tree
        val: L
    }
}

impl<B, L> Tree<B, L> for RcTree<B, L> {
    fn new_leaf(val: L) -> Self {
        RcTree::Leaf { val }
    }

    fn new_branch(val: B, children: Vec<Self>) -> Self {
        RcTree::Branch { val, children: Rc::new(children) }
    }

    fn iter_children(&self) -> Iter<'_, RcTree<B, L>> {
        match self {
            RcTree::Branch { children, .. } => {
                let borrowed: &Vec<Self> = children.borrow();
                borrowed.iter()
            },
            RcTree::Leaf { .. } => [].iter()
        }
    }

    fn branch_val(&self) -> Option<&B> {
        match self {
            RcTree::Branch { val, .. } => Some(val),
            RcTree::Leaf { .. } => None
        }
    }

    fn leaf_val(&self) -> Option<&L> {
        match self {
            RcTree::Branch { .. } => None,
            RcTree::Leaf { val } => Some(val)
        }
    }
}

use std::borrow::Borrow;

impl<B, L> From<RcTree<B, L>> for BoxTree<B, L>
    where
        B: Clone,
        L: Clone {

    fn from(rc_tree: RcTree<B, L>) -> Self {
        match rc_tree {
            RcTree::Branch { val, children } => {
                let children: &Vec<RcTree<B, L>> = children.borrow();
                let children: Vec<BoxTree<B, L>> = children.clone().into_iter()
                    .map(BoxTree::from).collect();

                BoxTree::Branch { val, children }
            },
            RcTree::Leaf { val } => BoxTree::Leaf { val }
        }
    }
}

impl<B, L> From<BoxTree<B, L>> for RcTree<B, L> {
    fn from(box_tree: BoxTree<B, L>) -> Self {
        match box_tree {
            BoxTree::Branch { val, children } => {
                let children: Vec<RcTree<B, L>> = children
                    .into_iter().map(RcTree::from).collect();
                let children = Rc::new(children);
                
                RcTree::Branch { val, children }
            },
            BoxTree::Leaf { val } => RcTree::Leaf { val }
        }
    }
}
