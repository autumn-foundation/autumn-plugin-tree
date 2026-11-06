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
///
/// Keys must be unique in one tree. The widgets render [`MAX_DEPTH`](crate::MAX_DEPTH)
/// levels. `Clone`, `PartialEq` and `Debug` go down each level, so keep
/// app trees inside that depth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    key: TreeKey,
    label: String,
    href: Option<String>,
    link_text: Option<String>,
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
            link_text: None,
            disabled: false,
            children: Children::None,
        }
    }

    /// Set the link. [`nav_tree`](crate::nav_tree) uses it.
    #[must_use]
    pub fn href(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }

    /// Set the text of the own link of a parent in
    /// [`nav_tree`](crate::nav_tree). The default is the label.
    #[must_use]
    pub fn link_text(mut self, text: impl Into<String>) -> Self {
        self.link_text = Some(text.into());
        self
    }

    /// Make the node disabled.
    #[must_use]
    pub const fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    /// Add a child node. This replaces a lazy URL.
    #[must_use]
    pub fn child(self, child: Self) -> Self {
        self.children([child])
    }

    /// Add child nodes. This replaces a lazy URL.
    #[must_use]
    pub fn children(mut self, children: impl IntoIterator<Item = Self>) -> Self {
        match &mut self.children {
            Children::Nodes(nodes) => nodes.extend(children),
            other => *other = Children::Nodes(children.into_iter().collect()),
        }
        self
    }

    /// Load the children from `url` when the node opens. This replaces
    /// the children.
    #[must_use]
    pub fn lazy(mut self, url: impl Into<String>) -> Self {
        self.children = Children::Lazy(url.into());
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

    /// The text of the own link of a parent: the link text, else the label.
    #[must_use]
    pub fn link_label(&self) -> &str {
        self.link_text.as_deref().unwrap_or(&self.label)
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

impl TreeNode {
    /// The child nodes that the server renders now. Empty for an end node
    /// and for a lazy node.
    pub(crate) fn loaded(&self) -> &[Self] {
        match &self.children {
            Children::Nodes(nodes) => nodes,
            Children::None | Children::Lazy(_) => &[],
        }
    }
}

/// Drop the children without recursion, so a very deep tree cannot
/// overflow the stack.
impl Drop for TreeNode {
    fn drop(&mut self) {
        let mut stack = match std::mem::take(&mut self.children) {
            Children::Nodes(nodes) => nodes,
            Children::None | Children::Lazy(_) => return,
        };
        while let Some(mut node) = stack.pop() {
            if let Children::Nodes(nodes) = std::mem::take(&mut node.children) {
                stack.extend(nodes);
            }
        }
    }
}
