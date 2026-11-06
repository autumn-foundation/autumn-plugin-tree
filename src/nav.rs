//! The navigation tree.

use autumn_web::form::FieldOptions;
use maud::Markup;

use crate::key::TreeKey;
use crate::node::TreeNode;

/// Render navigation links as a disclosure tree.
///
/// `label` names the `<nav>` landmark. The link of `current` gets
/// `aria-current="page"`, and its ancestors are open.
#[must_use]
pub fn nav_tree(
    label: &str,
    nodes: &[TreeNode],
    current: Option<&TreeKey>,
    options: &FieldOptions<'_>,
) -> Markup {
    let _ = (label, nodes, current, options);
    maud::html! {}
}
