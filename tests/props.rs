//! Property tests: they check rules over many random inputs.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test fails loud"
)]

use std::collections::BTreeSet;

use autumn_plugin_tree::{Cascade, TreeField, TreeKey, TreeNode, safe_href};
use proptest::prelude::*;

/// A random tree of up to three levels. Keys are unique.
fn tree() -> impl Strategy<Value = Vec<TreeNode>> {
    let root = (any::<bool>(), prop::collection::vec(any::<bool>(), 0..4));
    prop::collection::vec(root, 1..5).prop_map(|roots| {
        let mut n = 0;
        let mut key = || {
            n += 1;
            format!("k{n}")
        };
        roots
            .into_iter()
            .map(|(disabled, kids)| {
                let mut node = TreeNode::new(key(), "x");
                if disabled {
                    node = node.disabled();
                }
                let children: Vec<TreeNode> = kids
                    .into_iter()
                    .map(|d| {
                        let child = TreeNode::new(key(), "y");
                        if d { child.disabled() } else { child }
                    })
                    .collect();
                if children.is_empty() {
                    node
                } else {
                    node.children(children)
                }
            })
            .collect()
    })
}

/// All keys in tree order, with their disabled flag.
fn keys(nodes: &[TreeNode], out: &mut Vec<(String, bool)>) {
    for node in nodes {
        out.push((node.key().as_str().to_owned(), node.is_disabled()));
        if let autumn_plugin_tree::Children::Nodes(kids) = node.child_nodes() {
            keys(kids, out);
        }
    }
}

proptest! {
    #[test]
    fn tokens_are_one_to_one_and_safe(a in ".{0,12}", b in ".{0,12}") {
        let (ta, tb) = (TreeKey::new(a.as_str()).token(), TreeKey::new(b.as_str()).token());
        prop_assert!(ta.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_'));
        prop_assert_eq!(a == b, ta == tb);
    }

    #[test]
    fn a_javascript_link_never_passes(prefix in "[ \\t\\n\\x01]{0,3}", case in "[jJ][aA][vV][aA]", tail in "[\\t\\n]{0,2}") {
        let href = format!("{prefix}{case}{tail}script:alert(1)");
        prop_assert_eq!(safe_href(&href), None);
    }

    #[test]
    fn resolve_keeps_known_enabled_keys_in_tree_order(
        nodes in tree(),
        picks in prop::collection::vec(0usize..20, 0..10),
        cascade in any::<bool>(),
    ) {
        let mut all = Vec::new();
        keys(&nodes, &mut all);
        let submitted: Vec<String> = picks.iter().map(|i| format!("k{i}")).collect();
        let field = TreeField::multiple(nodes)
            .cascade(if cascade { Cascade::Descendants } else { Cascade::Independent });
        let out: Vec<String> = field.resolve(&submitted).iter().map(|k| k.as_str().to_owned()).collect();
        // Only enabled keys of the tree.
        for key in &out {
            prop_assert!(all.iter().any(|(k, d)| k == key && !d), "{key} not an enabled key");
        }
        // Tree order, no duplicates.
        let order: Vec<&String> = all.iter().map(|(k, _)| k).filter(|k| out.contains(k)).collect();
        prop_assert_eq!(order, out.iter().collect::<Vec<_>>());
        prop_assert_eq!(out.iter().collect::<BTreeSet<_>>().len(), out.len());
        // Independent: exactly the valid submitted keys.
        if !cascade {
            let valid: BTreeSet<&String> = submitted.iter()
                .filter(|s| all.iter().any(|(k, d)| k == *s && !d)).collect();
            prop_assert_eq!(valid, out.iter().collect::<BTreeSet<_>>());
        }
        // A second pass changes nothing.
        let again: Vec<String> = field.resolve(&out).iter().map(|k| k.as_str().to_owned()).collect();
        prop_assert_eq!(again, out);
    }
}
