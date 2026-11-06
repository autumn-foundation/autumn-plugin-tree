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

Always check the posted keys on the server:

```rust
let keys = tree.resolve(&form.folders);
```

`resolve` keeps only keys of the tree that are not disabled. With
`Cascade::Descendants` (the default), a parent adds its children, and a
parent with all children checked is added. A single tree gives one key or
none.

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
| `.disabled()` | The node cannot change. |

### Lazy children

The children load with htmx. Return `TreeField::branch` from the route:

```rust
#[get("/folders/children")]
async fn children(Query(q): Query<Node>) -> AutumnResult<Markup> {
    // Check that the user can see these nodes.
    let nodes = load_children(&q.node).await?;
    Ok(tree.branch("folders", q.level, &nodes, &checked))
}
```

With JavaScript off, the node has a link to the same URL. Return a full
page when the request has no `HX-Request` header.

## Navigation tree

```rust
use autumn_plugin_tree::{TreeKey, TreeNode, nav_tree};

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
cargo test --features system-tests --test browser -- --include-ignored
```

The browser tests need Chromium. The tree-key tests need the keyboard core
from autumn `main`:

```bash
cargo test --features system-tests,core-tree-keys --test browser \
  --config 'patch.crates-io.autumn-web.git="https://github.com/autumn-foundation/autumn"' \
  -- --include-ignored
```

## License

Apache-2.0. `tests/fixtures/axe-core` is axe-core (MPL-2.0).
