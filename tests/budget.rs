//! Render budgets: a deep or large tree does not stop the server.

mod common;

use autumn_plugin_tree::{MAX_DEPTH, MAX_NODES, TreeField, TreeNode, nav_tree, tree_field};
use autumn_web::form::{Changeset, FieldOptions};
use common::{parse, select};

#[derive(serde::Serialize, Default)]
struct Pick {
    pick: Vec<String>,
}

/// A chain of `depth` nodes. It is built from the leaf up, without recursion.
fn chain(depth: usize) -> TreeNode {
    let mut node = TreeNode::new(format!("n{depth}"), "leaf");
    for i in (1..depth).rev() {
        node = TreeNode::new(format!("n{i}"), format!("level {i}")).child(node);
    }
    node
}

fn field_html(nodes: Vec<TreeNode>) -> String {
    let changeset = Changeset::new(Pick::default());
    tree_field(&changeset, "pick", "Pick", &TreeField::multiple(nodes), &FieldOptions::new())
        .into_string()
}

#[test]
fn the_budgets_are_sane() {
    assert_eq!(MAX_DEPTH, 64);
    assert_eq!(MAX_NODES, 10_000);
}

#[test]
fn a_deep_tree_stops_at_max_depth() {
    let html = field_html(vec![chain(MAX_DEPTH + 40)]);
    let doc = parse(&html);
    assert_eq!(select(&doc, "li.autumn-tree-node").len(), MAX_DEPTH);
    assert!(html.contains(&format!("aria-level=\"{MAX_DEPTH}\"")));
    assert!(!html.contains(&format!("aria-level=\"{}\"", MAX_DEPTH + 1)));
}

#[test]
fn a_large_tree_stops_at_max_nodes() {
    let nodes: Vec<TreeNode> =
        (0..MAX_NODES + 500).map(|i| TreeNode::new(format!("k{i}"), "x")).collect();
    let doc = parse(&field_html(nodes));
    assert_eq!(select(&doc, "li.autumn-tree-node").len(), MAX_NODES);
}

#[test]
fn the_nav_tree_has_the_same_budgets() {
    let html = nav_tree("Deep", &[chain(MAX_DEPTH + 10)], None, &FieldOptions::new()).into_string();
    let doc = parse(&html);
    assert_eq!(select(&doc, "li").len(), MAX_DEPTH);
}

#[test]
fn resolve_has_the_same_depth_budget() {
    let tree = TreeField::multiple(vec![chain(MAX_DEPTH + 5)]);
    let deep = format!("n{}", MAX_DEPTH + 3);
    assert!(tree.resolve(&[deep.as_str()]).is_empty());
    let last = format!("n{MAX_DEPTH}");
    assert!(!tree.resolve(&[last.as_str()]).is_empty());
}
