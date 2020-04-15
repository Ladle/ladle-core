use std::slice::Iter;

/// A Tree that has values stored in both its leaves and branches
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

/// A Heap Allocated Tree.
/// It stores it's children in a vector.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BoxTree<B, L> {
    /// A branch node of an BoxTree.
    /// It has a list of children and possesses a branch value.
    Branch {
        /// The value (non-terminal) associated with this branch of the tree
        val: B,
        /// The children of this node
        children: Vec<BoxTree<B, L>>
    },
    /// A leaf node of an BoxTree.
    /// It has no children and possesses a leaf value
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

/// A Reference Counted Tree,
/// which can be shared and share data with other RcTrees.
/// It stores it's children in a reference counted vector to achieve this.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RcTree<B, L> {
    /// A branch node of an RcTree.
    /// It has a list of children and possesses a branch value.
    Branch {
        /// The value associated with this branch of the tree.
        val: B,
        /// The children of this node.
        children: Rc<Vec<RcTree<B, L>>>
    },
    /// A leaf node of an RcTree.
    /// It has no children and possesses a leaf value
    Leaf {
        /// The value associated with this leaf of the tree.
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
