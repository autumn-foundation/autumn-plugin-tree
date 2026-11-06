//! The navigation tree.
//!
//! A navigation tree is a disclosure list, not a `role="tree"`. The APG
//! says: "the disclosure pattern is better suited for most web sites".
//! It needs no JavaScript.

use std::collections::BTreeSet;

use autumn_web::form::FieldOptions;
use autumn_web::widget_pack::element;
use maud::{Markup, html};

use crate::budget::{Budget, can_descend};
use crate::href::safe_href;
use crate::key::TreeKey;
use crate::node::TreeNode;

/// Render navigation links as a disclosure tree.
///
/// - `label` names the `<nav>` landmark.
/// - A parent is a `<details>`. Its `<summary>` holds the label. When the
///   parent has a link, the link is the first item of its list.
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
    let mut open = BTreeSet::new();
    if let Some(current) = current {
        path_to(nodes, current, 1, &mut open);
    }
    let mut budget = Budget::new();
    let list = Nav {
        open: &open,
        current,
    }
    .list(nodes, 1, &mut budget, None);
    element(
        "nav",
        &[("class", "autumn-tree-nav"), ("aria-label", label)],
        options,
        list,
    )
}

/// Add the keys on the path to `target` to `path`. The path includes
/// `target` when it has children. Return `true` when `target` is found.
fn path_to<'a>(
    nodes: &'a [TreeNode],
    target: &TreeKey,
    depth: usize,
    path: &mut BTreeSet<&'a TreeKey>,
) -> bool {
    for node in nodes {
        let found = node.key() == target
            || (can_descend(depth) && path_to(node.loaded(), target, depth + 1, path));
        if found {
            if !node.loaded().is_empty() {
                path.insert(node.key());
            }
            return true;
        }
    }
    false
}

/// The render state of one navigation tree.
struct Nav<'a> {
    open: &'a BTreeSet<&'a TreeKey>,
    current: Option<&'a TreeKey>,
}

impl Nav<'_> {
    /// A `<ul>` of `nodes`. `parent` is the link of the parent node, which
    /// goes first.
    fn list(
        &self,
        nodes: &[TreeNode],
        depth: usize,
        budget: &mut Budget,
        parent: Option<&TreeNode>,
    ) -> Markup {
        let mut items = Vec::with_capacity(nodes.len().saturating_add(1));
        if let Some(parent) = parent {
            items.push(html! { li class="autumn-tree-nav-item" { (self.link(parent)) } });
        }
        for node in nodes {
            if !budget.take() {
                break;
            }
            items.push(self.item(node, depth, budget));
        }
        html! {
            ul class="autumn-tree-nav-list" {
                @for item in &items { (item) }
            }
        }
    }

    fn item(&self, node: &TreeNode, depth: usize, budget: &mut Budget) -> Markup {
        let children = node.loaded();
        if children.is_empty() || !can_descend(depth) {
            return html! { li class="autumn-tree-nav-item" { (self.link(node)) } };
        }
        let has_link = !node.is_disabled() && node.link().and_then(safe_href).is_some();
        let list = self.list(children, depth + 1, budget, has_link.then_some(node));
        html! {
            li class="autumn-tree-nav-item" {
                details class="autumn-tree-nav-details" open[self.open.contains(node.key())] {
                    summary class="autumn-tree-nav-summary" { (node.label()) }
                    (list)
                }
            }
        }
    }

    /// The link of a node, or its text.
    fn link(&self, node: &TreeNode) -> Markup {
        let href = node
            .link()
            .and_then(safe_href)
            .filter(|_| !node.is_disabled());
        let current = self.current == Some(node.key());
        html! {
            @if let Some(href) = href {
                a class="autumn-tree-nav-link" href=(href)
                    aria-current=[current.then_some("page")] { (node.label()) }
            } @else {
                span class="autumn-tree-nav-text" { (node.label()) }
            }
        }
    }
}
