//! The tree form field.
//!
//! The server sends a nested list of native radios or checkboxes. A parent
//! is a `<details>`. This works with JavaScript off. `tree.js` changes the
//! list into a WAI-ARIA APG tree on `autumn:init`. The native inputs stay in
//! the form, so the browser posts them.

use std::collections::{BTreeSet, HashSet};

use autumn_web::form::{Changeset, FieldContext, FieldOptions, FieldRenderer};
use maud::{Markup, PreEscaped, html};

use crate::flat::Flat;
use crate::key::TreeKey;
use crate::node::TreeNode;

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
    /// The check reads the same nodes as the renderer: [`MAX_DEPTH`](crate::MAX_DEPTH)
    /// levels and [`MAX_NODES`](crate::MAX_NODES) nodes. A disabled node
    /// keeps its own state, and a cascade does not go through it.
    #[must_use]
    pub fn resolve<S: AsRef<str>>(&self, submitted: &[S]) -> Vec<TreeKey> {
        let flat = Flat::new(&self.nodes, 1);
        let enabled = |i: &usize| !flat.nodes[*i].node.is_disabled();
        if !self.multiple {
            return submitted
                .iter()
                .find_map(|s| flat.find(s.as_ref()).filter(enabled))
                .map(|i| flat.nodes[i].node.key().clone())
                .into_iter()
                .collect();
        }
        let mut on = vec![false; flat.nodes.len()];
        for key in submitted {
            if let Some(i) = flat.find(key.as_ref()).filter(enabled) {
                on[i] = true;
            }
        }
        if self.cascades() {
            // Down: a checked parent checks its enabled children. A parent
            // comes before its children in tree order.
            for i in 0..flat.nodes.len() {
                if let Some(parent) = flat.nodes[i].parent
                    && on[parent]
                    && enabled(&i)
                {
                    on[i] = true;
                }
            }
            // Up: a parent with all enabled children checked is checked. A
            // cut parent is not: the cut children are unknown.
            for i in (0..flat.nodes.len()).rev() {
                let entry = &flat.nodes[i];
                if !enabled(&i) || entry.truncated {
                    continue;
                }
                let mut children = entry.children.iter().filter(|c| enabled(c)).peekable();
                if children.peek().is_some() && children.all(|c| on[*c]) {
                    on[i] = true;
                }
            }
        }
        let mut seen = HashSet::new();
        flat.nodes
            .iter()
            .zip(on)
            .filter(|(n, on)| *on && seen.insert(n.node.key().as_str()))
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
        let flat = Flat::new(nodes, level.max(1));
        let render = Render {
            tree: self,
            flat: &flat,
            field,
            values: &values,
            input: &writer,
        };
        render.items(&flat.roots, false).markup
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
        let flat = Flat::new(&self.nodes, 1);
        let render = Render {
            tree: self,
            flat: &flat,
            field: field.name(),
            values: &values,
            input: &writer,
        };
        let items = render.items(&flat.roots, false).markup;
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

// ── Rendering ───────────────────────────────────────────────────────────

/// The check state of a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Check {
    Off,
    On,
    Mixed,
}

impl From<bool> for Check {
    fn from(on: bool) -> Self {
        if on { Self::On } else { Self::Off }
    }
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
                // autumn-web 0.10 writes `required` on each box, just after
                // `name`. Later releases do not, so this does nothing there.
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
    flat: &'a Flat<'a>,
    field: &'a str,
    values: &'a HashSet<&'a str>,
    input: &'a InputWriter<'a>,
}

impl Render<'_> {
    /// Render the nodes at `indexes`: one level. `forced` is `true` when a
    /// checked parent cascades to them.
    fn items(&self, indexes: &[usize], forced: bool) -> Rendered {
        let mut markup = String::new();
        let mut states = Vec::with_capacity(indexes.len());
        let mut any_checked = false;
        let size = indexes.len().to_string();
        for (position, index) in indexes.iter().enumerate() {
            let item = self.item(*index, position + 1, &size, forced);
            markup.push_str(&item.markup.into_string());
            any_checked |= item.checked_below || item.check != Check::Off;
            if !self.flat.nodes[*index].node.is_disabled() {
                states.push(item.check);
            }
        }
        Rendered {
            markup: PreEscaped(markup),
            check: combine(&states),
            any_checked,
        }
    }

    /// Render one node.
    fn item(&self, index: usize, position: usize, size: &str, forced: bool) -> Item {
        let entry = &self.flat.nodes[index];
        let node = entry.node;
        let place = Place {
            level: entry.level,
            position,
            size,
        };
        let cascades = self.tree.cascades();
        // A disabled node is frozen: it keeps its own state, and a cascade
        // does not go through it.
        let frozen = node.is_disabled();
        let on = self.values.contains(node.key().as_str()) || (forced && cascades && !frozen);
        let (body, check, checked_below) = match entry.lazy {
            Some(url) => (Body::Lazy(url), Check::from(on), false),
            None if entry.children.is_empty() => (Body::Leaf, Check::from(on), false),
            None => {
                let below = self.items(&entry.children, cascades && on && !frozen);
                // A cut parent keeps its own state: the cut children are
                // unknown.
                let check = if on {
                    Check::On
                } else if cascades && !frozen && !entry.truncated {
                    below.check.unwrap_or(Check::Off)
                } else {
                    Check::Off
                };
                (Body::Group(below.markup), check, below.any_checked)
            }
        };
        let markup = self.node(node, &place, check, body, checked_below, entry.truncated);
        Item {
            markup,
            check,
            checked_below,
        }
    }

    /// Write the `<li>` of one node: the input, then a label or a
    /// `<details>` with the children. `partial` marks a parent whose
    /// children the node budget cut.
    #[allow(
        clippy::too_many_arguments,
        reason = "each argument is one part of the node state"
    )]
    fn node(
        &self,
        node: &TreeNode,
        place: &Place<'_>,
        check: Check,
        body: Body<'_>,
        open: bool,
        partial: bool,
    ) -> Markup {
        let tree = self.tree;
        let suffix = format!("n-{}", node.key().token());
        let base = format!("{}-{suffix}", self.field);
        let label_id = format!("{base}-label");
        let mixed_id = format!("{base}-mixed");
        let mixed = check == Check::Mixed;
        // With JavaScript off, the "partly selected" text is part of the
        // input name.
        let labelled_by = if mixed {
            format!("{label_id} {mixed_id}")
        } else {
            label_id.clone()
        };
        let mut attrs: Vec<(&str, &str)> = vec![
            ("type", if tree.multiple { "checkbox" } else { "radio" }),
            ("value", node.key().as_str()),
            ("class", "autumn-tree-input"),
            ("aria-labelledby", &labelled_by),
        ];
        if check == Check::On {
            attrs.push(("checked", ""));
        }
        if node.is_disabled() {
            attrs.push(("disabled", ""));
        }
        let input = self.input.write(&suffix, &attrs);
        let open = open || tree.expanded.contains(node.key());
        let label = html! { span class="autumn-tree-label" id=(label_id) { (node.label()) } };
        let group_id = format!("{base}-group");
        html! {
            li class="autumn-tree-node" id=(format!("{base}-node"))
                aria-level=(place.level) aria-setsize=(place.size) aria-posinset=(place.position)
                data-autumn-tree-mixed=[mixed.then_some("true")]
                data-autumn-tree-partial=[partial.then_some("true")]
            {
                (input)
                @match body {
                    Body::Leaf => {
                        label class="autumn-tree-row" for=(base) { (label) }
                    }
                    Body::Lazy(url) => {
                        details class="autumn-tree-details" open[open] {
                            summary class="autumn-tree-row" { (label) }
                            // The group does not take the htmx attributes of
                            // the form around it.
                            ul class="autumn-tree-group" id=(group_id)
                                data-autumn-tree-src=(url) hx-get=(url)
                                hx-trigger="autumn-tree:load" hx-swap="innerHTML"
                                hx-target="this" hx-select="unset" hx-include="unset"
                                hx-vals="unset" hx-confirm="unset" hx-push-url="false"
                            {
                                li class="autumn-tree-placeholder" {
                                    a href=(url) { (tree.text.show) }
                                }
                            }
                        }
                    }
                    Body::Group(children) => {
                        details class="autumn-tree-details" open[open] {
                            summary class="autumn-tree-row" {
                                (label)
                                @if mixed {
                                    span class="autumn-tree-mixed" id=(mixed_id) { (tree.text.mixed) }
                                }
                            }
                            ul class="autumn-tree-group" id=(group_id) { (children) }
                        }
                    }
                }
            }
        }
    }
}

/// One rendered node.
struct Item {
    markup: Markup,
    check: Check,
    /// A node under this one is checked or mixed.
    checked_below: bool,
}

/// The position of a node in its level.
struct Place<'a> {
    level: usize,
    position: usize,
    size: &'a str,
}

/// What a node holds after its input.
enum Body<'a> {
    /// A label: an end node.
    Leaf,
    /// An empty group that loads from this URL.
    Lazy(&'a str),
    /// The rendered children.
    Group(Markup),
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
