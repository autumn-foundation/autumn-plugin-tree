//! Regression tests for review findings: disabled nodes, the node budget,
//! lazy nodes, duplicate keys and deep trees.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test fails loud"
)]

mod common;

use autumn_plugin_tree::{
    MAX_DEPTH, MAX_NODES, TreeField, TreeKey, TreeNode, nav_tree, tree_field,
};
use autumn_web::form::{Changeset, FieldOptions};
use common::{attr, one, parse, select, text};

#[derive(serde::Serialize, Default)]
struct Pick {
    files: Vec<String>,
}

fn render(tree: &TreeField, values: &[&str]) -> String {
    let changeset = Changeset::new(Pick {
        files: values.iter().map(|v| (*v).to_owned()).collect(),
    });
    tree_field(&changeset, "files", "Files", tree, &FieldOptions::new()).into_string()
}

fn checked(html: &str) -> Vec<String> {
    let doc = parse(html);
    select(&doc, "input[checked]")
        .iter()
        .filter_map(|i| attr(*i, "value").map(str::to_owned))
        .collect()
}

/// `g` holds a disabled `d` (with `x` and `y`) and `c`.
fn frozen() -> TreeField {
    TreeField::multiple([TreeNode::new("g", "G").children([
        TreeNode::new("d", "D")
            .disabled()
            .children([TreeNode::new("x", "X"), TreeNode::new("y", "Y")]),
        TreeNode::new("c", "C"),
    ])])
}

#[test]
fn a_disabled_parent_is_frozen_and_not_mixed() {
    let html = render(&frozen(), &["x"]);
    let doc = parse(&html);
    assert!(
        select(&doc, "li#files-n-d-node[data-autumn-tree-mixed]").is_empty(),
        "{html}"
    );
    assert_eq!(checked(&html), ["x"]);
}

#[test]
fn a_cascade_does_not_go_through_a_disabled_node() {
    assert_eq!(checked(&render(&frozen(), &["g"])), ["g", "c"]);
    assert_eq!(checked(&render(&frozen(), &["d"])), ["d"]);
}

#[test]
fn render_and_resolve_agree() {
    let tree = frozen();
    for values in [vec!["g"], vec!["x"], vec!["c"], vec!["x", "y", "c"]] {
        let resolved = tree.resolve(&values);
        let keys: Vec<&str> = resolved.iter().map(TreeKey::as_str).collect();
        let shown: Vec<String> = checked(&render(&tree, &values))
            .into_iter()
            .filter(|k| k != "d")
            .collect();
        assert_eq!(shown, keys, "values {values:?}");
    }
}

#[test]
fn set_size_counts_the_rendered_items_when_the_budget_cuts_a_level() {
    let nodes: Vec<TreeNode> = (0..MAX_NODES + 5)
        .map(|i| TreeNode::new(format!("k{i}"), "x"))
        .collect();
    let html = render(&TreeField::multiple(nodes), &[]);
    let doc = parse(&html);
    let first = one(&doc, "li#files-n-k0-node");
    assert_eq!(
        attr(first, "aria-setsize"),
        Some(MAX_NODES.to_string().as_str())
    );
}

#[test]
fn a_cut_parent_is_partial_and_keeps_its_own_state() {
    // One parent with more children than the budget: the cut children are
    // unknown, so all rendered children checked does not check the parent.
    let children: Vec<TreeNode> = (0..MAX_NODES + 5)
        .map(|i| TreeNode::new(format!("k{i}"), "x"))
        .collect();
    let rendered: Vec<String> = (0..MAX_NODES - 1).map(|i| format!("k{i}")).collect();
    let tree = TreeField::multiple([TreeNode::new("p", "P").children(children)]);
    let values: Vec<&str> = rendered.iter().map(String::as_str).collect();
    let html = render(&tree, &values);
    let doc = parse(&html);
    let parent = one(&doc, "li#files-n-p-node");
    assert_eq!(attr(parent, "data-autumn-tree-partial"), Some("true"));
    assert!(
        one(&doc, "input[value=p]")
            .value()
            .attr("checked")
            .is_none()
    );
    assert!(!tree.resolve(&values).iter().any(|k| k.as_str() == "p"));
}

#[test]
fn an_expanded_lazy_node_is_open() {
    let tree = TreeField::multiple([TreeNode::new("a", "A").lazy("/c")]).expanded(["a"]);
    let doc = parse(&render(&tree, &[]));
    one(&doc, "li#files-n-a-node > details[open]");
}

#[test]
fn a_lazy_group_does_not_take_the_form_htmx_attributes() {
    let tree = TreeField::multiple([TreeNode::new("a", "A").lazy("/c")]);
    let doc = parse(&render(&tree, &[]));
    let group = one(&doc, "ul.autumn-tree-group");
    assert_eq!(attr(group, "hx-target"), Some("this"));
    for name in ["hx-select", "hx-include", "hx-vals", "hx-confirm"] {
        assert_eq!(attr(group, name), Some("unset"), "{name}");
    }
    assert_eq!(attr(group, "hx-push-url"), Some("false"));
}

#[test]
fn a_mixed_note_names_the_input_with_javascript_off() {
    let tree = TreeField::multiple([
        TreeNode::new("p", "P").children([TreeNode::new("a", "A"), TreeNode::new("b", "B")])
    ]);
    let doc = parse(&render(&tree, &["a"]));
    let input = one(&doc, "input[value=p]");
    assert_eq!(
        attr(input, "aria-labelledby"),
        Some("files-n-p-label files-n-p-mixed")
    );
    assert_eq!(text(one(&doc, "#files-n-p-mixed")), "Partly selected");
}

#[test]
fn duplicate_keys_give_one_result() {
    let tree = TreeField::multiple([
        TreeNode::new("a", "A").child(TreeNode::new("k", "K")),
        TreeNode::new("b", "B").child(TreeNode::new("k", "K")),
    ]);
    let keys: Vec<String> = tree
        .resolve(&["k"])
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(keys, ["a", "k"]);
}

#[test]
fn many_submitted_values_resolve_fast() {
    let nodes: Vec<TreeNode> = (0..MAX_NODES)
        .map(|i| TreeNode::new(format!("k{i}"), "x"))
        .collect();
    let submitted: Vec<String> = (0..200_000).map(|i| format!("v{i}")).collect();
    let start = std::time::Instant::now();
    assert!(
        TreeField::single(nodes.clone())
            .resolve(&submitted)
            .is_empty()
    );
    assert!(TreeField::multiple(nodes).resolve(&submitted).is_empty());
    assert!(
        start.elapsed() < std::time::Duration::from_secs(5),
        "{:?}",
        start.elapsed()
    );
}

#[test]
fn a_very_deep_tree_drops_without_a_stack_overflow() {
    let mut node = TreeNode::new("leaf", "leaf");
    for i in 0..200_000 {
        node = TreeNode::new(format!("n{i}"), "x").child(node);
    }
    let html = nav_tree(
        "Deep",
        std::slice::from_ref(&node),
        None,
        &FieldOptions::new(),
    )
    .into_string();
    assert_eq!(
        parse(&html)
            .select(&scraper::Selector::parse("li").unwrap())
            .count(),
        MAX_DEPTH
    );
    drop(node);
}

#[test]
fn nav_link_text_names_the_own_link_of_a_parent() {
    let nodes = [TreeNode::new("guide", "Guide")
        .href("/guide")
        .link_text("Guide overview")
        .child(TreeNode::new("start", "Start").href("/guide/start"))];
    let doc = parse(&nav_tree("Docs", &nodes, None, &FieldOptions::new()).into_string());
    assert_eq!(text(one(&doc, "a[href=\"/guide\"]")), "Guide overview");
    assert_eq!(text(one(&doc, "summary")), "Guide");
}
