//! The flat view of a tree: one walk inside the render budgets.
//!
//! The renderers and `resolve` read this view, so they see the same nodes.

use std::collections::HashMap;

use crate::budget::{MAX_DEPTH, MAX_NODES};
use crate::href::safe_href;
use crate::node::{Children, TreeNode};

/// One node in tree order.
pub struct FlatNode<'a> {
    pub node: &'a TreeNode,
    pub parent: Option<usize>,
    /// The child nodes in the view, in order.
    pub children: Vec<usize>,
    /// The level of the node. The root level is 1.
    pub level: usize,
    /// The safe URL of a lazy node at a level that can show children.
    pub lazy: Option<&'a str>,
    /// The node budget cut the children of this node or of a node below it.
    pub truncated: bool,
}

/// The nodes in tree order, inside [`MAX_DEPTH`] and [`MAX_NODES`].
pub struct Flat<'a> {
    pub nodes: Vec<FlatNode<'a>>,
    /// The top nodes of the view.
    pub roots: Vec<usize>,
    /// The first node of each key.
    index: HashMap<&'a str, usize>,
}

impl<'a> Flat<'a> {
    /// The view of `roots`, which are at `level`.
    pub fn new(roots: &'a [TreeNode], level: usize) -> Self {
        let mut flat = Self {
            nodes: Vec::new(),
            roots: Vec::new(),
            index: HashMap::new(),
        };
        if level <= MAX_DEPTH {
            flat.roots = flat.add(roots, None, level);
        }
        flat
    }

    /// Add `nodes` under `parent`. Return their indexes.
    fn add(&mut self, nodes: &'a [TreeNode], parent: Option<usize>, level: usize) -> Vec<usize> {
        let mut added = Vec::with_capacity(nodes.len().min(MAX_NODES));
        for node in nodes {
            if self.nodes.len() >= MAX_NODES {
                self.truncate(parent);
                break;
            }
            let index = self.nodes.len();
            let lazy = match node.child_nodes() {
                Children::Lazy(url) if level < MAX_DEPTH => safe_href(url),
                _ => None,
            };
            self.nodes.push(FlatNode {
                node,
                parent,
                children: Vec::new(),
                level,
                lazy,
                truncated: false,
            });
            self.index.entry(node.key().as_str()).or_insert(index);
            added.push(index);
            if level < MAX_DEPTH {
                let children = self.add(node.loaded(), Some(index), level + 1);
                self.nodes[index].children = children;
            }
        }
        added
    }

    /// Mark `parent` and its ancestors as cut.
    fn truncate(&mut self, mut parent: Option<usize>) {
        while let Some(index) = parent {
            if self.nodes[index].truncated {
                return;
            }
            self.nodes[index].truncated = true;
            parent = self.nodes[index].parent;
        }
    }

    /// The index of the first node with `key`.
    pub fn find(&self, key: &str) -> Option<usize> {
        self.index.get(key).copied()
    }
}
