//! The widget pack and the plugin.

use autumn_web::app::AppBuilder;
use autumn_web::assets::PluginAssets;
use autumn_web::form::{Changeset, FieldOptions};
use autumn_web::plugin::Plugin;
use autumn_web::plugin_contract::PluginContract;
use autumn_web::stories::story;
use autumn_web::widget_pack::WidgetPack;

use crate::field::{TreeField, tree_field};
use crate::key::TreeKey;
use crate::nav::nav_tree;
use crate::node::TreeNode;

/// The pack name. Classes start with `autumn-tree`.
pub const PACK_NAME: &str = "tree";

/// The pack CSS and JS.
pub static ASSETS: PluginAssets = PluginAssets::from_files(
    PACK_NAME,
    &[
        ("tree.css", include_bytes!("../assets/tree.css")),
        ("tree.js", include_bytes!("../assets/tree.js")),
    ],
);

/// The pack: its stories and its assets.
///
/// Link the assets in the layout after the core assets:
///
/// ```
/// let head = autumn_plugin_tree::pack().head().into_string();
/// assert!(head.contains(r#"<link rel="stylesheet" href="/static/_plugins/tree/"#));
/// assert!(head.contains(r#"<script src="/static/_plugins/tree/"#));
/// ```
#[must_use]
pub fn pack() -> WidgetPack {
    WidgetPack::new(PACK_NAME)
        .extend([
            story! {
                "Tree",
                "Tree: single tree",
                {
                    let changeset = Changeset::new(Pick { file: "main".into(), files: Vec::new() });
                    tree_field(
                        &changeset,
                        "file",
                        "File",
                        &TreeField::single(demo_files()),
                        &FieldOptions::new().hint("Pick one file."),
                    )
                }
            },
            story! {
                "Tree",
                "Tree: multiple tree",
                {
                    let changeset = Changeset::new(Pick {
                        file: String::new(),
                        files: vec!["main".into(), "readme".into()],
                    });
                    tree_field(
                        &changeset,
                        "files",
                        "Files",
                        &TreeField::multiple(demo_files()).lines(),
                        &FieldOptions::new().hint("A folder checks all its files."),
                    )
                }
            },
            story! {
                "Tree",
                "Tree: lazy tree",
                {
                    let changeset = Changeset::new(Pick::default());
                    let nodes = [
                        TreeNode::new("archive", "archive").lazy("/tree-demo/children?node=archive"),
                        TreeNode::new("notes", "notes.md"),
                    ];
                    tree_field(
                        &changeset,
                        "folder",
                        "Folder",
                        &TreeField::single(nodes).compact(),
                        &FieldOptions::new(),
                    )
                }
            },
            story! {
                "Tree",
                "Tree: navigation tree",
                {
                    nav_tree(
                        "Documentation",
                        &demo_docs(),
                        Some(&TreeKey::new("tree")),
                        &FieldOptions::new(),
                    )
                }
            },
        ])
        .assets(&ASSETS)
}

#[derive(serde::Serialize, Default)]
struct Pick {
    file: String,
    files: Vec<String>,
}

fn demo_files() -> Vec<TreeNode> {
    vec![
        TreeNode::new("src", "src").children([
            TreeNode::new("lib", "lib.rs"),
            TreeNode::new("bin", "bin").children([
                TreeNode::new("main", "main.rs"),
                TreeNode::new("cli", "cli.rs"),
            ]),
            TreeNode::new("build", "build.rs").disabled(),
        ]),
        TreeNode::new("tests", "tests").child(TreeNode::new("smoke", "smoke.rs")),
        TreeNode::new("readme", "README.md"),
    ]
}

fn demo_docs() -> Vec<TreeNode> {
    vec![
        TreeNode::new("home", "Home").href("/"),
        TreeNode::new("guide", "Guide").children([
            TreeNode::new("start", "Getting started").href("/guide/start"),
            TreeNode::new("widgets", "Widgets")
                .href("/guide/widgets")
                .children([
                    TreeNode::new("tree", "Tree").href("/guide/widgets/tree"),
                    TreeNode::new("tabs", "Tabs").href("/guide/widgets/tabs"),
                ]),
        ]),
        TreeNode::new("api", "API").href("/api"),
    ]
}

/// The plugin. It installs [`pack`]: it serves the assets and adds the
/// stories to the `/_stories` gallery.
///
/// ```
/// use autumn_plugin_tree::TreePlugin;
///
/// let _app = autumn_web::app().plugin(TreePlugin);
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(self, app: AppBuilder) -> AppBuilder {
        app.widget_pack(pack())
    }

    fn contract(&self) -> Option<PluginContract> {
        Some(
            PluginContract::new(env!("CARGO_PKG_NAME"))
                .plugin_version(env!("CARGO_PKG_VERSION"))
                .autumn_web("0.10"),
        )
    }
}
