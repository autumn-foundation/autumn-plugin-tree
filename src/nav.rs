//! The navigation tree.
//!
//! A navigation tree is a disclosure list, not a `role="tree"`. The APG
//! says: "the disclosure pattern is better suited for most web sites".
//! It needs no JavaScript.

use autumn_web::form::FieldOptions;
use autumn_web::widget_pack::element;
use maud::{Markup, html};

use crate::flat::Flat;
use crate::href::safe_href;
use crate::key::TreeKey;
use crate::node::TreeNode;

/// Render navigation links as a disclosure tree.
///
/// - `label` names the `<nav>` landmark.
/// - A parent is a `<details>`. Its `<summary>` holds the label. When the
///   parent has a link, the link is the first item of its list. Set
///   [`TreeNode::link_text`] to give that link its own text, for example
///   "Guide overview".
/// - The link of `current` gets `aria-current="page"`. Its ancestors are
///   open. When `current` is a parent, it is open too.
/// - A disabled node, or a node with an unsafe link, is text.
/// - A lazy node is an end node.
/// - The root gets the app `data-*` and `hx-*` passthrough from `options`.
///
/// ```
/// use autumn_plugin_tree::{TreeKey, TreeNode, nav_tree};
/// use autumn_web::form::FieldOptions;
///
/// let nodes = [TreeNode::new("guide", "Guide")
///     .child(TreeNode::new("start", "Start").href("/guide/start"))];
/// let html = nav_tree("Docs", &nodes, Some(&TreeKey::new("start")), &FieldOptions::new())
///     .into_string();
/// assert!(html.contains(r#"aria-current="page""#));
/// assert!(html.contains("<details class=\"autumn-tree-nav-details\" open>"));
/// ```
#[must_use]
pub fn nav_tree(
    label: &str,
    nodes: &[TreeNode],
    current: Option<&TreeKey>,
    options: &FieldOptions<'_>,
) -> Markup {
    let flat = Flat::new(nodes, 1);
    let current = current.and_then(|key| flat.find(key.as_str()));
    let mut open = vec![false; flat.nodes.len()];
    let mut walk = current;
    while let Some(index) = walk {
        open[index] = !flat.nodes[index].children.is_empty();
        walk = flat.nodes[index].parent;
    }
    let nav = Nav {
        flat: &flat,
        open: &open,
        current,
    };
    element(
        "nav",
        &[("class", "autumn-tree-nav"), ("aria-label", label)],
        options,
        nav.list(&flat.roots, None),
    )
}

/// The render state of one navigation tree.
struct Nav<'a> {
    flat: &'a Flat<'a>,
    open: &'a [bool],
    current: Option<usize>,
}

impl Nav<'_> {
    /// A `<ul>` of the nodes at `indexes`. `parent` is the parent node,
    /// whose link goes first.
    fn list(&self, indexes: &[usize], parent: Option<usize>) -> Markup {
        html! {
            ul class="autumn-tree-nav-list" {
                @if let Some(parent) = parent {
                    li class="autumn-tree-nav-item" { (self.link(parent, true)) }
                }
                @for index in indexes { (self.item(*index)) }
            }
        }
    }

    fn item(&self, index: usize) -> Markup {
        let entry = &self.flat.nodes[index];
        if entry.children.is_empty() {
            return html! { li class="autumn-tree-nav-item" { (self.link(index, false)) } };
        }
        let node = entry.node;
        let has_link = !node.is_disabled() && node.link().and_then(safe_href).is_some();
        html! {
            li class="autumn-tree-nav-item" {
                details class="autumn-tree-nav-details" open[self.open[index]] {
                    summary class="autumn-tree-nav-summary" { (node.label()) }
                    (self.list(&entry.children, has_link.then_some(index)))
                }
            }
        }
    }

    /// The link of a node, or its text. The own link of a parent uses the
    /// link text of the node.
    fn link(&self, index: usize, own: bool) -> Markup {
        let node = self.flat.nodes[index].node;
        let href = node
            .link()
            .and_then(safe_href)
            .filter(|_| !node.is_disabled());
        let current = self.current == Some(index);
        let text = if own { node.link_label() } else { node.label() };
        html! {
            @if let Some(href) = href {
                a class="autumn-tree-nav-link" href=(href)
                    aria-current=[current.then_some("page")] { (text) }
            } @else {
                span class="autumn-tree-nav-text" { (text) }
            }
        }
    }
}
