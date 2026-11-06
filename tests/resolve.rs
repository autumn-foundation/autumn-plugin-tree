//! `TreeField::resolve`: the server check of a submitted form.

mod common;

use autumn_plugin_tree::{Cascade, TreeField, TreeKey, TreeNode};
use common::files;

fn keys(keys: &[TreeKey]) -> Vec<&str> {
    keys.iter().map(TreeKey::as_str).collect()
}

#[test]
fn unknown_keys_are_dropped() {
    let tree = TreeField::multiple(files()).cascade(Cascade::Independent);
    assert_eq!(keys(&tree.resolve(&["lib", "nope", "../etc", ""])), ["lib"]);
}

#[test]
fn the_result_is_in_tree_order_without_duplicates() {
    let tree = TreeField::multiple(files()).cascade(Cascade::Independent);
    assert_eq!(keys(&tree.resolve(&["readme", "lib", "readme", "src"])), ["src", "lib", "readme"]);
}

#[test]
fn a_checked_parent_adds_its_descendants() {
    let tree = TreeField::multiple(files());
    assert_eq!(keys(&tree.resolve(&["src"])), ["src", "lib", "bin", "main"]);
}

#[test]
fn a_parent_with_all_children_checked_is_added() {
    let tree = TreeField::multiple(files());
    assert_eq!(keys(&tree.resolve(&["main"])), ["bin", "main"]);
    assert_eq!(keys(&tree.resolve(&["main", "lib"])), ["src", "lib", "bin", "main"]);
}

#[test]
fn a_parent_with_some_children_checked_is_not_added() {
    let tree = TreeField::multiple(files());
    assert_eq!(keys(&tree.resolve(&["lib"])), ["lib"]);
}

#[test]
fn disabled_keys_are_dropped_and_do_not_count() {
    let nodes = vec![TreeNode::new("p", "P").children([
        TreeNode::new("a", "A"),
        TreeNode::new("b", "B").disabled(),
    ])];
    let tree = TreeField::multiple(nodes);
    assert_eq!(keys(&tree.resolve(&["b"])), Vec::<&str>::new());
    // `b` is disabled, so a check on `a` checks the parent.
    assert_eq!(keys(&tree.resolve(&["a"])), ["p", "a"]);
    // A check on the parent does not add the disabled child.
    assert_eq!(keys(&tree.resolve(&["p"])), ["p", "a"]);
}

#[test]
fn a_lazy_parent_stays_as_submitted() {
    let nodes = vec![TreeNode::new("arch", "Archive").lazy("/c")];
    let tree = TreeField::multiple(nodes);
    assert_eq!(keys(&tree.resolve(&["arch"])), ["arch"]);
}

#[test]
fn a_single_tree_gives_the_first_valid_key() {
    let tree = TreeField::single(files());
    assert_eq!(keys(&tree.resolve(&["nope", "main", "lib"])), ["main"]);
    assert_eq!(keys(&tree.resolve::<&str>(&[])), Vec::<&str>::new());
}

#[test]
fn a_single_tree_does_not_cascade() {
    let tree = TreeField::single(files());
    assert_eq!(keys(&tree.resolve(&["src"])), ["src"]);
}

#[test]
fn resolve_accepts_owned_strings() {
    let tree = TreeField::multiple(files());
    let submitted = vec![String::from("lib")];
    assert_eq!(keys(&tree.resolve(&submitted)), ["lib"]);
}
