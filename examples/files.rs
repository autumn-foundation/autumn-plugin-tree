//! A folder picker with lazy branches and a server check.
//!
//! Run: `cargo run --example files`, then open <http://localhost:3000>.

use autumn_plugin_tree::{Cascade, TreeField, TreeNode, TreePlugin, nav_tree, pack, tree_field};
use autumn_web::extract::Query;
use autumn_web::form::{Changeset, FieldOptions};
use autumn_web::prelude::*;
use maud::{DOCTYPE, Markup, html};

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Scope {
    #[serde(default)]
    folders: Vec<String>,
}

#[derive(serde::Deserialize)]
struct Branch {
    node: String,
}

/// The top of the tree. `archive` loads its children when it opens.
fn tree() -> TreeField {
    TreeField::multiple([
        TreeNode::new("src", "src").children([
            TreeNode::new("lib", "lib.rs"),
            TreeNode::new("bin", "bin").child(TreeNode::new("main", "main.rs")),
        ]),
        TreeNode::new("archive", "archive").lazy("/children?node=archive"),
        TreeNode::new("readme", "README.md"),
    ])
    // The default cascade, written out for the reader.
    .cascade(Cascade::Descendants)
    .lines()
}

/// The children of a lazy node. A real app checks that the user can see them.
fn archive() -> Vec<TreeNode> {
    vec![
        TreeNode::new("2025", "2025"),
        TreeNode::new("2026", "2026").child(TreeNode::new("q4", "q4.md")),
    ]
}

/// The full tree, for the server check.
fn full_tree() -> TreeField {
    TreeField::multiple([
        TreeNode::new("src", "src").children([
            TreeNode::new("lib", "lib.rs"),
            TreeNode::new("bin", "bin").child(TreeNode::new("main", "main.rs")),
        ]),
        TreeNode::new("archive", "archive").children(archive()),
        TreeNode::new("readme", "README.md"),
    ])
}

fn layout(body: &Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                title { "Folders" }
                link rel="stylesheet" href=(autumn_web::ui::WIDGETS_CSS_PATH);
                script src="/static/js/htmx.min.js" defer {}
                script src="/static/js/autumn-widgets.js" defer {}
                (pack().head())
            }
            body {
                (nav_tree("Pages", &[TreeNode::new("home", "Folders").href("/")], None, &FieldOptions::new()))
                main { (body) }
            }
        }
    }
}

#[get("/")]
#[public]
async fn index() -> Markup {
    let changeset = Changeset::new(Scope::default());
    layout(&html! {
        h1 { "Pick folders" }
        form method="get" action="/save" {
            (tree_field(&changeset, "folders", "Folders", &tree(), &FieldOptions::new().hint("A folder checks all its files.")))
            button type="submit" { "Save" }
        }
    })
}

#[get("/children")]
#[public]
async fn children(Query(branch): Query<Branch>) -> Markup {
    let nodes = if branch.node == "archive" {
        archive()
    } else {
        Vec::new()
    };
    tree().branch("folders", 2, &nodes, &[] as &[&str])
}

#[get("/save")]
#[public]
async fn save(Query(scope): Query<Scope>) -> Markup {
    let keys = full_tree().resolve(&scope.folders);
    layout(&html! {
        h1 { "Saved" }
        ul { @for key in &keys { li { (key) } } }
        a href="/" { "Back" }
    })
}

#[autumn_web::main]
async fn main() {
    // The app future is large. Box it, so the stack stays small.
    Box::pin(
        autumn_web::app()
            .plugin(TreePlugin)
            .routes(routes![index, children, save])
            .run(),
    )
    .await;
}
