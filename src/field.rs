//! The tree form field.
//!
//! The server sends a nested list of native radios or checkboxes. A parent
//! is a `<details>`. This works with JavaScript off. `tree.js` changes the
//! list into a WAI-ARIA APG tree on `autumn:init`. The native inputs stay in
//! the form, so the browser posts them.

use std::collections::{BTreeSet, HashSet};

use autumn_web::form::{Changeset, FieldContext, FieldOptions, FieldRenderer};
use maud::{Markup, PreEscaped, html};

use crate::budget::{Budget, MAX_DEPTH, MAX_NODES, can_descend};
use crate::href::safe_href;
use crate::key::TreeKey;
use crate::node::{Children, TreeNode};

/// How a check on a parent node changes the other nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Cascade {
    /// A check on a parent checks all its children. A parent is checked
    /// when all its children are checked, and mixed when some are.
    /// Disabled nodes do not change and do not count.
    #[default]
    Descendants,
    /// Each node is checked alone.
    Independent,
}

impl Cascade {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Descendants => "descendants",
            Self::Independent => "independent",
        }
    }
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
            loading: "Loading…".to_owned(),
            failed: "Could not load. Open the branch again to try again.".to_owned(),
            show: "Show contents".to_owned(),
            mixed: "Partly selected".to_owned(),
        }
    }
}

/// A tree of radios or checkboxes, bound to one form field.
///
/// Bind it with [`tree_field`], or with `form_for` and
/// `FieldControl::custom(tree)`. Check the posted values with
/// [`resolve`](Self::resolve).
///
/// ```
/// use autumn_plugin_tree::{TreeField, TreeNode, tree_field};
/// use autumn_web::form::{Changeset, FieldOptions};
///
/// #[derive(serde::Serialize)]
/// struct Scope { folders: Vec<String> }
///
/// let tree = TreeField::multiple([
///     TreeNode::new("src", "src").child(TreeNode::new("lib", "lib.rs")),
/// ]);
/// let changeset = Changeset::new(Scope { folders: vec!["lib".into()] });
/// let html = tree_field(&changeset, "folders", "Folders", &tree, &FieldOptions::new())
///     .into_string();
/// assert!(html.contains(r#"data-autumn-tree="multiple""#));
/// assert_eq!(tree.resolve(&["lib"])[0].as_str(), "src");
/// ```
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

    /// Set the cascade of a multiple tree. A single tree ignores it.
    #[must_use]
    pub const fn cascade(mut self, cascade: Cascade) -> Self {
        self.cascade = cascade;
        self
    }

    /// Open these nodes. The ancestors of a checked node are also open.
    #[must_use]
    pub fn expanded<K: Into<TreeKey>>(mut self, keys: impl IntoIterator<Item = K>) -> Self {
        self.expanded.extend(keys.into_iter().map(Into::into));
        self
    }

    /// Show guide lines.
    #[must_use]
    pub const fn lines(mut self) -> Self {
        self.lines = true;
        self
    }

    /// Use the compact density.
    #[must_use]
    pub const fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    /// Set the texts.
    #[must_use]
    pub fn text(mut self, text: TreeText) -> Self {
        self.text = text;
        self
    }

    /// The nodes.
    #[must_use]
    pub fn nodes(&self) -> &[TreeNode] {
        &self.nodes
    }

    const fn cascades(&self) -> bool {
        self.multiple && matches!(self.cascade, Cascade::Descendants)
    }

    /// Check the submitted values against the tree.
    ///
    /// The result has only keys of the tree that are not disabled, in tree
    /// order, with no duplicates. A single tree gives the first valid key,
    /// or none. With [`Cascade::Descendants`], a parent adds its children,
    /// and a parent with all children checked is added. A lazy parent stays
    /// as submitted: the app owns its children.
    ///
    /// The check reads the same nodes as the renderer: [`MAX_DEPTH`] levels
    /// and [`MAX_NODES`] nodes.
    #[must_use]
    pub fn resolve<S: AsRef<str>>(&self, submitted: &[S]) -> Vec<TreeKey> {
        let flat = Flat::new(&self.nodes);
        let enabled = |i: &usize| !flat.nodes[*i].node.is_disabled();
        if !self.multiple {
            return submitted
                .iter()
                .find_map(|s| flat.find(s.as_ref()).filter(enabled))
                .map(|i| flat.nodes[i].node.key().clone())
                .into_iter()
                .collect();
        }
        let wanted: HashSet<&str> = submitted.iter().map(AsRef::as_ref).collect();
        let mut on: Vec<bool> = flat
            .nodes
            .iter()
            .map(|n| !n.node.is_disabled() && wanted.contains(n.node.key().as_str()))
            .collect();
        if self.cascades() {
            // Down: a checked parent checks its enabled children. A parent
            // comes before its children in tree order.
            for i in 0..flat.nodes.len() {
                if let Some(parent) = flat.nodes[i].parent
                    && on[parent]
                    && !flat.nodes[i].node.is_disabled()
                {
                    on[i] = true;
                }
            }
            // Up: a parent with all enabled children checked is checked.
            for i in (0..flat.nodes.len()).rev() {
                let entry = &flat.nodes[i];
                if entry.node.is_disabled() {
                    continue;
                }
                let mut children = entry.children.iter().filter(|c| enabled(c)).peekable();
                if children.peek().is_some() && children.all(|c| on[*c]) {
                    on[i] = true;
                }
            }
        }
        flat.nodes
            .iter()
            .zip(on)
            .filter(|(_, on)| *on)
            .map(|(n, _)| n.node.key().clone())
            .collect()
    }

    /// Render the children of a lazy node, for the htmx response.
    ///
    /// `field` is the form field name. `level` is the level of the children:
    /// the root level is 1. `values` are the checked keys. The result is the
    /// `<li>` items only. The response replaces the content of the group.
    /// The inputs get no app passthrough.
    #[must_use]
    pub fn branch<S: AsRef<str>>(
        &self,
        field: &str,
        level: usize,
        nodes: &[TreeNode],
        values: &[S],
    ) -> Markup {
        let values: HashSet<&str> = values.iter().map(AsRef::as_ref).collect();
        let writer = InputWriter::Plain { field };
        let mut render = Render {
            tree: self,
            field,
            values: &values,
            input: &writer,
            budget: Budget::new(),
        };
        let level = level.max(1);
        render.items(nodes, level, false).markup
    }

    fn render_field(&self, field: &FieldContext<'_>) -> Markup {
        let values: HashSet<&str> = if self.multiple {
            field.values().iter().map(String::as_str).collect()
        } else {
            field.value().into_iter().collect()
        };
        let writer = InputWriter::Field {
            context: field,
            strip_required: self.multiple && field.is_required(),
        };
        let mut render = Render {
            tree: self,
            field: field.name(),
            values: &values,
            input: &writer,
            budget: Budget::new(),
        };
        let items = render.items(&self.nodes, 1, false).markup;
        let mut class = String::from("autumn-tree-root");
        if self.lines {
            class.push_str(" autumn-tree--lines");
        }
        if self.compact {
            class.push_str(" autumn-tree--compact");
        }
        html! {
            ul class=(class)
                id=(format!("{}-tree", field.id()))
                aria-label=(field.label())
                data-autumn-tree=(if self.multiple { "multiple" } else { "single" })
                data-autumn-tree-cascade=[self.multiple.then(|| self.cascade.as_str())]
                data-autumn-tree-required=[field.is_required().then_some("true")]
                data-autumn-tree-loading=(self.text.loading)
                data-autumn-tree-failed=(self.text.failed)
                data-autumn-tree-mixed-text=(self.text.mixed)
            {
                (items)
            }
        }
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
        self.render_field(field)
    }
}

/// Render a [`TreeField`] bound to `changeset[field]`.
///
/// A single tree reads one value. A multiple tree reads a list: bind it to
/// a `Vec<String>` field with `#[serde(default)]`. The framework writes the
/// `<fieldset>`, the `<legend>`, the hint and the errors.
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

// ── Flat view for resolve ───────────────────────────────────────────────

/// One node in tree order, with its parent and children indexes.
struct FlatNode<'a> {
    node: &'a TreeNode,
    parent: Option<usize>,
    children: Vec<usize>,
}

/// The nodes in tree order, inside the render budgets.
struct Flat<'a> {
    nodes: Vec<FlatNode<'a>>,
}

impl<'a> Flat<'a> {
    fn new(roots: &'a [TreeNode]) -> Self {
        let mut flat = Self { nodes: Vec::new() };
        flat.add(roots, None, 1);
        flat
    }

    fn add(&mut self, nodes: &'a [TreeNode], parent: Option<usize>, depth: usize) {
        for node in nodes {
            if self.nodes.len() >= MAX_NODES {
                return;
            }
            let index = self.nodes.len();
            self.nodes.push(FlatNode {
                node,
                parent,
                children: Vec::new(),
            });
            if let Some(parent) = parent {
                self.nodes[parent].children.push(index);
            }
            if can_descend(depth) {
                self.add(node.loaded(), Some(index), depth + 1);
            }
        }
    }

    fn find(&self, key: &str) -> Option<usize> {
        self.nodes.iter().position(|n| n.node.key().as_str() == key)
    }
}

// ── Rendering ───────────────────────────────────────────────────────────

/// The check state of a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Check {
    Off,
    On,
    Mixed,
}

/// How the renderer writes one input.
enum InputWriter<'a> {
    /// Through the framework: the app passthrough, `required` and
    /// `aria-invalid` come with it.
    Field {
        context: &'a FieldContext<'a>,
        /// `required` on a checkbox means "check this box". A multiple tree
        /// removes it.
        strip_required: bool,
    },
    /// Plain attributes, for an htmx fragment.
    Plain { field: &'a str },
}

impl InputWriter<'_> {
    fn write(&self, suffix: &str, attrs: &[(&str, &str)]) -> Markup {
        match self {
            Self::Field {
                context,
                strip_required,
            } => {
                let input = context.group_item("input", suffix, attrs, html! {});
                if !*strip_required {
                    return input;
                }
                // `group_item` writes `required` just after `name`.
                let name = html! { (context.name()) }.into_string();
                let with = format!(" name=\"{name}\" required");
                let without = format!(" name=\"{name}\"");
                PreEscaped(input.into_string().replacen(&with, &without, 1))
            }
            Self::Plain { field } => html! {
                input id=(format!("{field}-{suffix}")) name=(field)
                    type=[attr(attrs, "type")]
                    value=[attr(attrs, "value")]
                    class=[attr(attrs, "class")]
                    aria-labelledby=[attr(attrs, "aria-labelledby")]
                    checked[attr(attrs, "checked").is_some()]
                    disabled[attr(attrs, "disabled").is_some()];
            },
        }
    }
}

fn attr<'a>(attrs: &[(&str, &'a str)], name: &str) -> Option<&'a str> {
    attrs.iter().find(|(n, _)| *n == name).map(|(_, v)| *v)
}

/// The result of rendering some items.
struct Rendered {
    markup: Markup,
    /// The check state of the items, for their parent. `None` when no
    /// enabled item counts.
    check: Option<Check>,
    /// An item, or a node under it, is checked.
    any_checked: bool,
}

struct Render<'a> {
    tree: &'a TreeField,
    field: &'a str,
    values: &'a HashSet<&'a str>,
    input: &'a InputWriter<'a>,
    budget: Budget,
}

impl Render<'_> {
    /// Render `nodes` at `level`. `forced` is `true` when a checked parent
    /// cascades to them.
    fn items(&mut self, nodes: &[TreeNode], level: usize, forced: bool) -> Rendered {
        let mut markup = String::new();
        let mut states = Vec::with_capacity(nodes.len());
        let mut any_checked = false;
        if level > MAX_DEPTH {
            return Rendered {
                markup: PreEscaped(markup),
                check: None,
                any_checked,
            };
        }
        let size = nodes.len().to_string();
        for (index, node) in nodes.iter().enumerate() {
            if !self.budget.take() {
                break;
            }
            let (item, check, checked_below) = self.item(node, level, index + 1, &size, forced);
            markup.push_str(&item.into_string());
            any_checked |= checked_below || check != Check::Off;
            if !node.is_disabled() {
                states.push(check);
            }
        }
        Rendered {
            markup: PreEscaped(markup),
            check: combine(&states),
            any_checked,
        }
    }

    /// Render one node. Return its markup, its check state, and whether a
    /// node under it is checked.
    fn item(
        &mut self,
        node: &TreeNode,
        level: usize,
        position: usize,
        size: &str,
        forced: bool,
    ) -> (Markup, Check, bool) {
        let tree = self.tree;
        let token = node.key().token();
        let suffix = format!("n-{token}");
        let base = format!("{}-{suffix}", self.field);
        let label_id = format!("{base}-label");
        let own = self.values.contains(node.key().as_str());
        let cascades = tree.cascades();
        let pushed = forced && cascades && !node.is_disabled();
        let lazy = match node.child_nodes() {
            Children::Lazy(url) => safe_href(url),
            _ => None,
        };
        let children = node.loaded();
        let parent = can_descend(level) && (!children.is_empty() || lazy.is_some());

        let (group, check, checked_below) = if parent && lazy.is_none() {
            let below = self.items(children, level + 1, cascades && (own || pushed));
            let check = if own || pushed {
                Check::On
            } else if cascades {
                below.check.unwrap_or(Check::Off)
            } else {
                Check::Off
            };
            (Some(below.markup), check, below.any_checked)
        } else {
            let check = if own || pushed { Check::On } else { Check::Off };
            (None, check, false)
        };

        let mut attrs: Vec<(&str, &str)> = vec![
            ("type", if tree.multiple { "checkbox" } else { "radio" }),
            ("value", node.key().as_str()),
            ("class", "autumn-tree-input"),
            ("aria-labelledby", &label_id),
        ];
        if check == Check::On {
            attrs.push(("checked", ""));
        }
        if node.is_disabled() {
            attrs.push(("disabled", ""));
        }
        let input = self.input.write(&suffix, &attrs);
        let mixed = check == Check::Mixed;
        let open = tree.expanded.contains(node.key()) || checked_below;
        let level_text = level.to_string();
        let position_text = position.to_string();

        let markup = html! {
            li class="autumn-tree-node" id=(format!("{base}-node"))
                aria-level=(level_text) aria-setsize=(size) aria-posinset=(position_text)
                data-autumn-tree-mixed=[mixed.then_some("true")]
            {
                (input)
                @if parent {
                    details class="autumn-tree-details" open[open && lazy.is_none()] {
                        summary class="autumn-tree-row" {
                            span class="autumn-tree-label" id=(label_id) { (node.label()) }
                            @if mixed {
                                span class="autumn-tree-mixed" { (tree.text.mixed) }
                            }
                        }
                        @if let Some(url) = lazy {
                            ul class="autumn-tree-group" id=(format!("{base}-group"))
                                data-autumn-tree-src=(url) hx-get=(url)
                                hx-trigger="autumn-tree:load" hx-swap="innerHTML"
                            {
                                li class="autumn-tree-placeholder" {
                                    a href=(url) { (tree.text.show) }
                                }
                            }
                        } @else {
                            ul class="autumn-tree-group" id=(format!("{base}-group")) {
                                @if let Some(group) = group { (group) }
                            }
                        }
                    }
                } @else {
                    label class="autumn-tree-row" for=(base) {
                        span class="autumn-tree-label" id=(label_id) { (node.label()) }
                    }
                }
            }
        };
        (markup, check, checked_below)
    }
}

/// The state of a parent from the states of its enabled children.
fn combine(states: &[Check]) -> Option<Check> {
    if states.is_empty() {
        return None;
    }
    if states.iter().all(|s| *s == Check::On) {
        Some(Check::On)
    } else if states.iter().all(|s| *s == Check::Off) {
        Some(Check::Off)
    } else {
        Some(Check::Mixed)
    }
}
