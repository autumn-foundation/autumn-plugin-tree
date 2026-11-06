# autumn-plugin-tree

A tree widget pack for [Autumn](https://github.com/autumn-foundation/autumn).
It follows the WAI-ARIA APG [tree view pattern](https://www.w3.org/WAI/ARIA/apg/patterns/treeview/).

| Widget | Use it for | Markup | JavaScript |
|---|---|---|---|
| `TreeField` | A file picker, a folder scope, a category choice: one choice (radios) or many (checkboxes). | `role="tree"` after the upgrade | `tree.js` |
| `nav_tree` | Site and docs navigation. | `<nav>`, nested lists, `<details>`, links | None |

Do not use `TreeField` for site navigation. The APG says the disclosure
pattern is better for web sites. Use `nav_tree`.

## Install

```toml
[dependencies]
autumn-plugin-tree = "0.1"
```

```rust
use autumn_plugin_tree::TreePlugin;

autumn_web::app().plugin(TreePlugin)
```

`TreePlugin` serves `tree.css` and `tree.js`, and adds the stories to the
`/_stories` gallery. Link the assets after the core assets:

```rust
head {
    link rel="stylesheet" href=(autumn_web::ui::WIDGETS_CSS_PATH);
    script src="/static/js/htmx.min.js" defer {}
    script src="/static/js/autumn-widgets.js" defer {}
    (autumn_plugin_tree::pack().head())
}
```

## Tree field

```rust
use autumn_plugin_tree::{TreeField, TreeNode, tree_field};
use autumn_web::form::{Changeset, FieldOptions};

#[derive(serde::Serialize, serde::Deserialize)]
struct Scope {
    #[serde(default)]
    folders: Vec<String>,
}

let changeset = Changeset::new(Scope { folders: Vec::new() });
let tree = TreeField::multiple([
    TreeNode::new("src", "src").children([
        TreeNode::new("lib", "lib.rs"),
        TreeNode::new("bin", "bin").lazy("/folders/children?node=bin"),
    ]),
    TreeNode::new("readme", "README.md"),
]);
let html = tree_field(&changeset, "folders", "Folders", &tree, &FieldOptions::new());
```

With `form_for`, use `.override_field("folders", FieldControl::custom(tree))`.

### Check the post

Always check the posted keys on the server. Build the tree with all the
nodes that the user can see, the lazy ones too:

```rust
let keys = tree.resolve(&form.folders);
```

`resolve` keeps only keys of the tree that are not disabled. With
`Cascade::Descendants` (the default), a parent adds its children, and a
parent with all children checked is added. A disabled node is frozen: a
cascade does not go through it. A single tree gives one key or none. Keys
must be unique in one tree.

The script sends a `change` event from the input that the user changed.
Listen on the form to get each change.

### Options

| Method | Effect |
|---|---|
| `TreeField::single(nodes)` | Radios: one choice. |
| `TreeField::multiple(nodes)` | Checkboxes: many choices. |
| `.cascade(Cascade::Independent)` | A check on a parent does not change its children. |
| `.expanded(keys)` | Open these nodes. The ancestors of a checked node are open. |
| `.lines()` | Show guide lines. |
| `.compact()` | Use less space. |
| `.text(TreeText { .. })` | Change the texts (default: English). |

### Node options

| Method | Effect |
|---|---|
| `TreeNode::new(key, label)` | An end node. The form posts the key. |
| `.child(node)`, `.children(nodes)` | Add child nodes. |
| `.lazy(url)` | Load the children from `url` when the node opens. |
| `.href(url)` | The link of the node in `nav_tree`. |
| `.link_text(text)` | The text of the own link of a parent in `nav_tree`. |
| `.disabled()` | The node cannot change. |

### Lazy children

The children load with htmx. Return `TreeField::branch` from the route:

```rust
#[derive(serde::Deserialize)]
struct Branch {
    node: String,
}

#[get("/folders/children")]
async fn children(Query(branch): Query<Branch>) -> AutumnResult<Markup> {
    // Check that the user can see these nodes.
    let nodes = load_children(&branch.node).await?;
    Ok(tree().branch("folders", 2, &nodes, &[] as &[&str]))
}
```

The second argument is the level of the children. The root level is 1.
The last argument is the checked keys. A checked parent checks its new
children in the browser.

With JavaScript off, the node links to the same URL. To show a full page
there, check the `HX-Request` header and put the fragment in your layout.
Without htmx, a tree with a lazy node stays a native list.

`examples/files.rs` is a complete app: `cargo run --example files`.

## Navigation tree

```rust
use autumn_plugin_tree::{TreeKey, nav_tree};
use autumn_web::form::FieldOptions;

let html = nav_tree("Docs", &nodes, Some(&TreeKey::new("forms")), &FieldOptions::new());
```

The current link gets `aria-current="page"`. Its ancestors are open.

## Keyboard

| Key | Effect | Who |
|---|---|---|
| ArrowDown, ArrowUp, Home, End, a letter | Move over the visible items. | Keyboard core |
| ArrowRight, ArrowLeft, `*` | Open, close, and move between parent and child. | Keyboard core (see below) |
| Enter, Space | Select (single) or check (multiple). | `tree.js` |
| Ctrl+A | Check all (multiple). | `tree.js` |
| Click on the chevron | Open or close. | `tree.js` |
| Click on the row | Select or check. | `tree.js` |

Focus does not change the selection.

**Note:** ArrowRight, ArrowLeft and `*` need the tree keys of the keyboard
core (autumn-foundation/autumn#3417). They are not in `autumn-web` 0.10.0.
With 0.10.0 the other keys work, and the chevron opens and closes a node.
With 0.10.0 a disabled node cannot take focus; use the chevron.

## Without JavaScript

The tree is a nested list of native radios or checkboxes. A parent is a
`<details>`. The form posts the inputs. A mixed parent shows a text. A lazy
node has a link to its children.

## Security

- maud escapes each label and key.
- `nav_tree` links and lazy URLs must be relative, `http`, `https` or
  `mailto`. Other links become text.
- The renderer stops at 64 levels (`MAX_DEPTH`) and 10,000 nodes
  (`MAX_NODES`).
- Each id starts with the field name. Keys are encoded to `[A-Za-z0-9_]`.
- The CSS and JS come from the pack bundle. There is no inline script, no
  inline style and no `style` attribute.
- Authorize each request for lazy children.

## Develop

```bash
cargo test
```

```bash
cargo test --features system-tests --test browser -- --include-ignored --test-threads=1
```

The browser tests need Chromium. The tree-key tests need the keyboard core
from the autumn default branch, `trunk-dev`:

```bash
cargo test --features system-tests,core-tree-keys --test browser \
  --config 'patch.crates-io.autumn-web.git="https://github.com/autumn-foundation/autumn"' \
  --config 'patch.crates-io.autumn-web.branch="trunk-dev"' \
  -- --include-ignored --test-threads=1
```

## License

Apache-2.0. `tests/fixtures/axe-core` is axe-core (MPL-2.0).
