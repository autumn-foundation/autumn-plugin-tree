//! The tree form field.

use std::collections::BTreeSet;

use autumn_web::form::{Changeset, FieldContext, FieldOptions, FieldRenderer};
use maud::Markup;

use crate::key::TreeKey;
use crate::node::TreeNode;

/// How a check on a parent node changes the other nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Cascade {
    /// A check on a parent checks all its children. A parent is checked
    /// when all its children are checked, and mixed when some are.
    #[default]
    Descendants,
    /// Each node is checked alone.
    Independent,
}

/// The texts of the tree. The defaults are English.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeText {
    /// The text of a branch that loads.
    pub loading: String,
    /// The text of a branch that did not load.
    pub failed: String,
    /// The link to the children of a lazy branch, with JavaScript off.
    pub show: String,
    /// The hidden text of a parent with some children checked.
    pub mixed: String,
}

impl Default for TreeText {
    fn default() -> Self {
        Self {
            loading: String::new(),
            failed: String::new(),
            show: String::new(),
            mixed: String::new(),
        }
    }
}

/// A tree of radios or checkboxes, bound to one form field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeField {
    nodes: Vec<TreeNode>,
    multiple: bool,
    cascade: Cascade,
    expanded: BTreeSet<TreeKey>,
    lines: bool,
    compact: bool,
    text: TreeText,
}

impl TreeField {
    /// A tree with one choice (radios).
    #[must_use]
    pub fn single(nodes: impl IntoIterator<Item = TreeNode>) -> Self {
        Self::make(nodes, false)
    }

    /// A tree with many choices (checkboxes).
    #[must_use]
    pub fn multiple(nodes: impl IntoIterator<Item = TreeNode>) -> Self {
        Self::make(nodes, true)
    }

    fn make(nodes: impl IntoIterator<Item = TreeNode>, multiple: bool) -> Self {
        Self {
            nodes: nodes.into_iter().collect(),
            multiple,
            cascade: Cascade::default(),
            expanded: BTreeSet::new(),
            lines: false,
            compact: false,
            text: TreeText::default(),
        }
    }

    /// Set the cascade of a multiple tree.
    #[must_use]
    pub const fn cascade(self, cascade: Cascade) -> Self {
        let _ = cascade;
        self
    }

    /// Open these nodes. The ancestors of a checked node are also open.
    #[must_use]
    pub fn expanded<K: Into<TreeKey>>(self, keys: impl IntoIterator<Item = K>) -> Self {
        let _ = keys.into_iter().count();
        self
    }

    /// Show guide lines.
    #[must_use]
    pub const fn lines(self) -> Self {
        self
    }

    /// Use the compact density.
    #[must_use]
    pub const fn compact(self) -> Self {
        self
    }

    /// Set the texts.
    #[must_use]
    pub fn text(self, text: TreeText) -> Self {
        let _ = text;
        self
    }

    /// The nodes.
    #[must_use]
    pub fn nodes(&self) -> &[TreeNode] {
        &self.nodes
    }

    /// Check the submitted values against the tree.
    ///
    /// The result has only keys of the tree that are not disabled, in tree
    /// order. A single tree gives one key or none. With
    /// [`Cascade::Descendants`], a parent adds its children, and a parent
    /// with all children checked is added.
    #[must_use]
    pub fn resolve<S: AsRef<str>>(&self, submitted: &[S]) -> Vec<TreeKey> {
        let _ = submitted;
        Vec::new()
    }

    /// Render the children of a lazy node, for the htmx response.
    ///
    /// `field` is the form field name, `level` is the level of the
    /// children (the root level is 1), and `values` are the checked keys.
    #[must_use]
    pub fn branch<S: AsRef<str>>(
        &self,
        field: &str,
        level: usize,
        nodes: &[TreeNode],
        values: &[S],
    ) -> Markup {
        let _ = (field, level, nodes, values);
        maud::html! {}
    }
}

impl FieldRenderer for TreeField {
    fn name(&self) -> &'static str {
        "tree"
    }

    fn grouped(&self) -> bool {
        true
    }

    fn render(&self, field: &FieldContext<'_>) -> Markup {
        let _ = field;
        maud::html! {}
    }
}

/// Render a [`TreeField`] bound to `changeset[field]`.
#[must_use]
pub fn tree_field<T: serde::Serialize>(
    changeset: &Changeset<T>,
    field: &str,
    label: &str,
    tree: &TreeField,
    options: &FieldOptions<'_>,
) -> Markup {
    autumn_web::form::custom_input_with(changeset, field, label, tree, options)
}
