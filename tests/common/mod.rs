//! Test helpers.

#![allow(dead_code, reason = "each test binary uses some helpers")]

use autumn_plugin_tree::TreeNode;
use scraper::{ElementRef, Html, Selector};

/// Parse a fragment.
pub fn parse(html: &str) -> Html {
    Html::parse_fragment(html)
}

/// All elements that match `css`.
pub fn select<'a>(doc: &'a Html, css: &str) -> Vec<ElementRef<'a>> {
    let selector = Selector::parse(css).expect("valid selector");
    doc.select(&selector).collect()
}

/// The one element that matches `css`.
pub fn one<'a>(doc: &'a Html, css: &str) -> ElementRef<'a> {
    let found = select(doc, css);
    assert_eq!(
        found.len(),
        1,
        "expected one `{css}`, found {}",
        found.len()
    );
    found[0]
}

/// An attribute value.
pub fn attr<'a>(el: ElementRef<'a>, name: &str) -> Option<&'a str> {
    el.value().attr(name)
}

/// The text of an element, trimmed.
pub fn text(el: ElementRef<'_>) -> String {
    el.text().collect::<String>().trim().to_owned()
}

/// A small file tree:
///
/// ```text
/// src
///   lib.rs
///   bin
///     main.rs
/// README.md
/// ```
pub fn files() -> Vec<TreeNode> {
    vec![
        TreeNode::new("src", "src").children([
            TreeNode::new("lib", "lib.rs"),
            TreeNode::new("bin", "bin").child(TreeNode::new("main", "main.rs")),
        ]),
        TreeNode::new("readme", "README.md"),
    ]
}
