//! The navigation tree: a disclosure list of links, not `role="tree"`.

mod common;

use autumn_plugin_tree::{TreeKey, TreeNode, nav_tree};
use autumn_web::form::FieldOptions;
use common::{attr, one, parse, select, text};

fn docs() -> Vec<TreeNode> {
    vec![
        TreeNode::new("home", "Home").href("/"),
        TreeNode::new("guide", "Guide").children([
            TreeNode::new("start", "Getting started").href("/guide/start"),
            TreeNode::new("forms", "Forms")
                .href("/guide/forms")
                .child(TreeNode::new("tree", "Tree field").href("/guide/forms/tree")),
        ]),
        TreeNode::new("api", "API").href("/api").child(TreeNode::new("ref", "Reference").href("/api/ref")),
    ]
}

fn render(current: Option<&str>) -> String {
    let key = current.map(TreeKey::new);
    nav_tree("Docs", &docs(), key.as_ref(), &FieldOptions::new()).into_string()
}

#[test]
fn the_root_is_a_named_nav_landmark() {
    let doc = parse(&render(None));
    let nav = one(&doc, "nav");
    assert_eq!(attr(nav, "aria-label"), Some("Docs"));
    assert_eq!(attr(nav, "class"), Some("autumn-tree-nav"));
}

#[test]
fn it_never_uses_tree_roles() {
    let html = render(Some("tree"));
    assert!(!html.contains("role=\"tree"), "{html}");
    assert!(!html.contains("aria-expanded"), "{html}");
}

#[test]
fn an_end_node_is_a_link() {
    let doc = parse(&render(None));
    let link = one(&doc, "a[href=\"/guide/start\"]");
    assert_eq!(text(link), "Getting started");
    assert_eq!(attr(link, "class"), Some("autumn-tree-nav-link"));
}

#[test]
fn a_parent_is_a_details_with_a_summary() {
    let doc = parse(&render(None));
    let summaries: Vec<String> = select(&doc, "details > summary").into_iter().map(text).collect();
    assert_eq!(summaries, ["Guide", "Forms", "API"]);
}

#[test]
fn a_parent_link_is_the_first_item_of_its_list() {
    let doc = parse(&render(None));
    let first = one(&doc, "details:has(> summary[id]) > ul > li:first-child > a[href=\"/api\"]");
    assert_eq!(text(first), "API");
    // The summary holds no link.
    assert!(select(&doc, "summary a").is_empty());
}

#[test]
fn the_current_link_has_aria_current_and_its_ancestors_are_open() {
    let doc = parse(&render(Some("tree")));
    let current = one(&doc, "[aria-current]");
    assert_eq!(attr(current, "aria-current"), Some("page"));
    assert_eq!(attr(current, "href"), Some("/guide/forms/tree"));
    let open: Vec<String> = select(&doc, "details[open] > summary").into_iter().map(text).collect();
    assert_eq!(open, ["Guide", "Forms"]);
}

#[test]
fn a_current_parent_marks_its_own_link() {
    let doc = parse(&render(Some("api")));
    let current = one(&doc, "[aria-current]");
    assert_eq!(attr(current, "href"), Some("/api"));
    let open: Vec<String> = select(&doc, "details[open] > summary").into_iter().map(text).collect();
    assert_eq!(open, ["API"]);
}

#[test]
fn no_current_means_all_closed() {
    let doc = parse(&render(None));
    assert!(select(&doc, "details[open]").is_empty());
    assert!(select(&doc, "[aria-current]").is_empty());
}

#[test]
fn an_unsafe_link_becomes_text() {
    let nodes = [TreeNode::new("x", "Click").href("javascript:alert(1)")];
    let html = nav_tree("Docs", &nodes, None, &FieldOptions::new()).into_string();
    assert!(!html.contains("javascript:"), "{html}");
    let doc = parse(&html);
    assert!(select(&doc, "a").is_empty());
    assert_eq!(text(one(&doc, "li")), "Click");
}

#[test]
fn a_disabled_node_is_text() {
    let nodes = [TreeNode::new("x", "Soon").href("/soon").disabled()];
    let doc = parse(&nav_tree("Docs", &nodes, None, &FieldOptions::new()).into_string());
    assert!(select(&doc, "a").is_empty());
    assert_eq!(text(one(&doc, "li span")), "Soon");
}

#[test]
fn labels_are_escaped() {
    let nodes = [TreeNode::new("x", "<script>alert(1)</script>").href("/x")];
    let html = nav_tree("<b>", &nodes, None, &FieldOptions::new()).into_string();
    assert!(!html.contains("<script>"), "{html}");
    assert!(!html.contains("<b>"), "{html}");
    assert!(html.contains("&lt;script&gt;"), "{html}");
}

#[test]
fn the_root_gets_the_app_passthrough() {
    let options = FieldOptions::new().data("test-id", "docs").hx("boost", "true");
    let doc = parse(&nav_tree("Docs", &docs(), None, &options).into_string());
    let nav = one(&doc, "nav");
    assert_eq!(attr(nav, "data-test-id"), Some("docs"));
    assert_eq!(attr(nav, "hx-boost"), Some("true"));
}

#[test]
fn every_class_is_in_the_pack_namespace() {
    let html = render(Some("tree"));
    let doc = parse(&html);
    for el in select(&doc, "[class]") {
        for class in attr(el, "class").unwrap_or_default().split_whitespace() {
            assert!(class.starts_with("autumn-tree-"), "{class}");
        }
    }
}

#[test]
fn a_lazy_node_renders_as_a_leaf() {
    let nodes = [TreeNode::new("x", "Archive").href("/archive").lazy("/archive/children")];
    let doc = parse(&nav_tree("Docs", &nodes, None, &FieldOptions::new()).into_string());
    assert!(select(&doc, "details").is_empty());
    assert_eq!(attr(one(&doc, "a"), "href"), Some("/archive"));
}
