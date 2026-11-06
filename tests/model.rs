//! The data model: keys, links and nodes.

use autumn_plugin_tree::{Children, TreeKey, TreeNode, safe_href};

#[test]
fn a_plain_key_is_its_own_token() {
    assert_eq!(TreeKey::new("src42").token(), "src42");
    assert_eq!(TreeKey::new("ABC").token(), "ABC");
}

#[test]
fn a_token_encodes_each_other_byte() {
    assert_eq!(TreeKey::new("a b").token(), "a_20b");
    assert_eq!(TreeKey::new("a-b").token(), "a_2db");
    assert_eq!(TreeKey::new("a_b").token(), "a_5fb");
    assert_eq!(TreeKey::new("a/b.rs").token(), "a_2fb_2ers");
    assert_eq!(TreeKey::new("é").token(), "_c3_a9");
    assert_eq!(TreeKey::new("\"><x").token(), "_22_3e_3cx");
}

#[test]
fn an_empty_key_has_a_token() {
    assert_eq!(TreeKey::new("").token(), "_");
}

#[test]
fn two_keys_do_not_share_a_token() {
    let keys = ["a-label", "a", "a_2dlabel", "a_label", "", "_"];
    let tokens: std::collections::BTreeSet<String> =
        keys.iter().map(|k| TreeKey::new(*k).token()).collect();
    assert_eq!(tokens.len(), keys.len());
}

#[test]
fn a_key_converts_from_text() {
    assert_eq!(TreeKey::from("x"), TreeKey::new("x"));
    assert_eq!(TreeKey::from(String::from("x")).as_str(), "x");
    assert_eq!(TreeKey::new("x").to_string(), "x");
}

#[test]
fn safe_links_pass() {
    for href in [
        "/docs",
        "docs/intro",
        "../up",
        "#top",
        "?page=2",
        "http://example.com",
        "https://example.com/a?b#c",
        "HTTPS://EXAMPLE.COM",
        "mailto:team@example.com",
        "//cdn.example.com/x",
    ] {
        assert_eq!(safe_href(href), Some(href), "{href}");
    }
}

#[test]
fn unsafe_links_fail() {
    for href in [
        "javascript:alert(1)",
        "JaVaScRiPt:alert(1)",
        " javascript:alert(1)",
        "java\tscript:alert(1)",
        "java\nscript:alert(1)",
        "\u{1}javascript:alert(1)",
        "data:text/html,<script>alert(1)</script>",
        "vbscript:msgbox",
        "file:///etc/passwd",
        "ftp://example.com",
    ] {
        assert_eq!(safe_href(href), None, "{href:?}");
    }
}

#[test]
fn a_relative_path_with_a_colon_after_a_slash_passes() {
    assert_eq!(safe_href("/a:b"), Some("/a:b"));
    assert_eq!(safe_href("./x:y"), Some("./x:y"));
}

#[test]
fn a_node_holds_its_data() {
    let node = TreeNode::new("docs", "Docs")
        .href("/docs")
        .disabled()
        .child(TreeNode::new("intro", "Intro"));
    assert_eq!(node.key().as_str(), "docs");
    assert_eq!(node.label(), "Docs");
    assert_eq!(node.link(), Some("/docs"));
    assert!(node.is_disabled());
    match node.child_nodes() {
        Children::Nodes(nodes) => assert_eq!(nodes[0].key().as_str(), "intro"),
        other => panic!("expected nodes, got {other:?}"),
    }
}

#[test]
fn children_add_in_order() {
    let node = TreeNode::new("p", "P")
        .child(TreeNode::new("a", "A"))
        .children([TreeNode::new("b", "B"), TreeNode::new("c", "C")]);
    let Children::Nodes(nodes) = node.child_nodes() else {
        panic!("expected nodes");
    };
    let keys: Vec<&str> = nodes.iter().map(|n| n.key().as_str()).collect();
    assert_eq!(keys, ["a", "b", "c"]);
}

#[test]
fn a_lazy_node_holds_its_url() {
    let node = TreeNode::new("p", "P").lazy("/children?node=p");
    assert_eq!(node.child_nodes(), &Children::Lazy("/children?node=p".into()));
}

#[test]
fn an_end_node_has_no_children() {
    assert_eq!(TreeNode::new("a", "A").child_nodes(), &Children::None);
    assert!(!TreeNode::new("a", "A").is_disabled());
    assert_eq!(TreeNode::new("a", "A").link(), None);
}
