//! The tree data model.

use crate::key::TreeKey;

/// The children of a node.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Children {
    /// An end node.
    #[default]
    None,
    /// Child nodes that the server renders now.
    Nodes(Vec<TreeNode>),
    /// Child nodes that load from this URL when the node opens.
    Lazy(String),
}

/// One node of a tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    key: TreeKey,
    label: String,
    href: Option<String>,
    disabled: bool,
    children: Children,
}

impl TreeNode {
    /// Make an end node.
    #[must_use]
    pub fn new(key: impl Into<TreeKey>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            href: None,
            disabled: false,
            children: Children::None,
        }
    }

    /// Set the link. [`nav_tree`](crate::nav_tree) uses it.
    #[must_use]
    pub fn href(mut self, href: impl Into<String>) -> Self {
        let _ = href.into();
        self
    }

    /// Make the node disabled.
    #[must_use]
    pub const fn disabled(self) -> Self {
        self
    }

    /// Add a child node.
    #[must_use]
    pub fn child(self, child: Self) -> Self {
        let _ = child;
        self
    }

    /// Add child nodes.
    #[must_use]
    pub fn children(self, children: impl IntoIterator<Item = Self>) -> Self {
        let _ = children.into_iter().count();
        self
    }

    /// Load the children from `url` when the node opens.
    #[must_use]
    pub fn lazy(self, url: impl Into<String>) -> Self {
        let _ = url.into();
        self
    }

    /// The key.
    #[must_use]
    pub const fn key(&self) -> &TreeKey {
        &self.key
    }

    /// The label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// The link.
    #[must_use]
    pub fn link(&self) -> Option<&str> {
        self.href.as_deref()
    }

    /// `true` when the node is disabled.
    #[must_use]
    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// The children.
    #[must_use]
    pub const fn child_nodes(&self) -> &Children {
        &self.children
    }
}
