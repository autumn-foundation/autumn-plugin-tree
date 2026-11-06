//! The tree in Chromium: the upgrade, the keys, the cascade, lazy loading,
//! the form post, the no-JS baseline and the axe gate.
//!
//! Run:
//!
//! ```text
//! cargo test --features system-tests --test browser -- --include-ignored
//! ```

#![cfg(feature = "system-tests")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test fails loud"
)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use autumn_plugin_tree::{TreeField, TreeKey, TreeNode, nav_tree, pack, tree_field};
use autumn_web::form::{Changeset, FieldOptions};
use autumn_web::plugin_conformance::widgets::browser::run_widget_browser_conformance;
use autumn_web::prelude::*;
use autumn_web::system_test::{Page, SystemTest, SystemTestRunner};
use maud::{DOCTYPE, Markup, html};

// ── The test app ────────────────────────────────────────────────────────

#[derive(serde::Serialize, Default)]
struct Pick {
    file: String,
    files: Vec<String>,
}

fn files() -> Vec<TreeNode> {
    vec![
        TreeNode::new("src", "src").children([
            TreeNode::new("lib", "lib.rs"),
            TreeNode::new("bin", "bin").children([
                TreeNode::new("main", "main.rs"),
                TreeNode::new("cli", "cli.rs"),
            ]),
            TreeNode::new("build", "build.rs").disabled(),
        ]),
        TreeNode::new("docs", "docs").lazy("/tree/children"),
        TreeNode::new("broken", "broken").lazy("/tree/broken"),
        TreeNode::new("empty", "empty").lazy("/tree/empty"),
        TreeNode::new("readme", "README.md"),
    ]
}

fn layout(body: &Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                title { "Tree" }
                link rel="stylesheet" href="/static/css/autumn-widgets.css";
                script src="/static/js/htmx.min.js" defer {}
                script src="/static/js/autumn-widgets.js" defer {}
                (pack().head())
            }
            body { main { (body) } }
        }
    }
}

#[get("/tree")]
async fn tree_page() -> Markup {
    let single = Changeset::new(Pick {
        file: "lib".into(),
        ..Pick::default()
    });
    let multiple = Changeset::new(Pick::default());
    layout(&html! {
        h1 { "Trees" }
        form id="single-form" method="get" action="/done" {
            (tree_field(&single, "file", "File", &TreeField::single(files()), &FieldOptions::new()))
        }
        form id="multi-form" method="get" action="/done" {
            (tree_field(&multiple, "files", "Files", &TreeField::multiple(files()), &FieldOptions::new()))
            button type="submit" { "Save" }
        }
        (nav_tree("Docs", &[
            TreeNode::new("guide", "Guide").child(TreeNode::new("start", "Start").href("/guide/start")),
        ], Some(&TreeKey::new("start")), &FieldOptions::new()))
    })
}

static CHILD_LOADS: AtomicUsize = AtomicUsize::new(0);

#[get("/tree/children")]
async fn tree_children() -> Markup {
    CHILD_LOADS.fetch_add(1, Ordering::SeqCst);
    let tree = TreeField::multiple(Vec::new());
    tree.branch(
        "files",
        2,
        &[
            TreeNode::new("guide", "guide.md"),
            TreeNode::new("api", "api.md"),
        ],
        &[] as &[&str],
    )
}

#[get("/tree/broken")]
async fn tree_broken() -> autumn_web::AutumnResult<Markup> {
    Err(autumn_web::AutumnError::internal_server_error_msg(
        "no children",
    ))
}

#[get("/tree/empty")]
async fn tree_empty() -> Markup {
    html! {}
}

#[get("/done")]
async fn done() -> Markup {
    layout(&html! { p { "Saved" } })
}

async fn start() -> (SystemTestRunner, Page) {
    let runner = Box::pin(
        SystemTest::new()
            .routes(routes![
                tree_page,
                tree_children,
                tree_broken,
                tree_empty,
                done
            ])
            .plugin_assets(&autumn_plugin_tree::ASSETS)
            .build(),
    )
    .await
    .expect("start runner");
    let page = runner.page().await.expect("page");
    page.visit("/tree").await.expect("visit");
    assert!(poll(&page, "document.readyState === 'complete'").await);
    assert!(
        poll(
            &page,
            "document.querySelectorAll('[role=tree]').length === 2"
        )
        .await,
        "the trees did not upgrade"
    );
    (runner, page)
}

// ── Helpers ─────────────────────────────────────────────────────────────

async fn poll(page: &Page, js: &str) -> bool {
    for _ in 0..100 {
        if eval_bool(page, js).await {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    false
}

async fn eval_bool(page: &Page, js: &str) -> bool {
    page.evaluate(js)
        .await
        .ok()
        .and_then(|r| r.into_value::<bool>().ok())
        .unwrap_or(false)
}

async fn eval_string(page: &Page, js: &str) -> String {
    page.evaluate(js)
        .await
        .ok()
        .and_then(|r| r.into_value::<String>().ok())
        .unwrap_or_default()
}

/// The id of the focused element.
async fn focused(page: &Page) -> String {
    eval_string(page, "document.activeElement.id").await
}

async fn focus(page: &Page, id: &str) {
    assert!(
        eval_bool(page, &format!("(() => {{ const el = document.getElementById('{id}'); el.focus(); return document.activeElement === el; }})()")).await,
        "cannot focus {id}"
    );
}

async fn attr(page: &Page, id: &str, name: &str) -> String {
    eval_string(
        page,
        &format!("String(document.getElementById('{id}').getAttribute('{name}'))"),
    )
    .await
}

async fn click_toggle(page: &Page, id: &str) {
    page.click(&format!("#{id} > .autumn-tree-row > .autumn-tree-toggle"))
        .await
        .expect("click toggle");
}

// ── The upgrade ─────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires Chromium"]
async fn the_server_list_becomes_an_apg_tree() {
    let (_runner, page) = start().await;
    let checks = [
        (
            "multiple tree is multiselectable",
            "document.getElementById('files-tree').getAttribute('aria-multiselectable') === 'true'",
        ),
        (
            "single tree is not multiselectable",
            "!document.getElementById('file-tree').hasAttribute('aria-multiselectable')",
        ),
        (
            "no details remain in a tree",
            "document.querySelectorAll('[role=tree] details').length === 0",
        ),
        (
            "each node is a treeitem",
            "[...document.querySelectorAll('[role=tree] li.autumn-tree-node')].every(li => li.getAttribute('role') === 'treeitem')",
        ),
        (
            "each child list is a group",
            "[...document.querySelectorAll('[role=tree] .autumn-tree-group')].every(ul => ul.getAttribute('role') === 'group')",
        ),
        (
            "a parent has aria-expanded",
            "document.getElementById('files-n-src-node').getAttribute('aria-expanded') === 'false'",
        ),
        (
            "an end node has no aria-expanded",
            "!document.getElementById('files-n-readme-node').hasAttribute('aria-expanded')",
        ),
        (
            "a closed group is hidden",
            "document.getElementById('files-n-src-group').hidden",
        ),
        (
            "the label names the item",
            "document.getElementById('files-n-lib-node').getAttribute('aria-labelledby') === 'files-n-lib-label'",
        ),
        (
            "the inputs stay, hidden",
            "[...document.querySelectorAll('[role=tree] input')].every(i => i.hidden)",
        ),
        (
            "a disabled node is aria-disabled",
            "document.getElementById('files-n-build-node').getAttribute('aria-disabled') === 'true'",
        ),
        (
            "single: aria-selected on each item",
            "document.getElementById('file-n-lib-node').getAttribute('aria-selected') === 'true' && document.getElementById('file-n-readme-node').getAttribute('aria-selected') === 'false'",
        ),
        (
            "multiple: aria-checked on each item",
            "document.getElementById('files-n-readme-node').getAttribute('aria-checked') === 'false'",
        ),
        (
            "the selected item is the tab stop",
            "document.getElementById('file-n-lib-node').tabIndex === 0",
        ),
        (
            "its ancestors are open",
            "document.getElementById('file-n-src-node').getAttribute('aria-expanded') === 'true'",
        ),
        (
            "one tab stop per tree",
            "[...document.querySelectorAll('[role=tree]')].every(t => t.querySelectorAll('[role=treeitem][tabindex=\"0\"]').length === 1)",
        ),
        (
            "the nav tree is not a tree",
            "!document.querySelector('nav [role=tree], nav [role=treeitem]')",
        ),
    ];
    for (name, js) in checks {
        assert!(poll(&page, js).await, "{name}: {js}");
    }
}

// ── Keys of the keyboard core ───────────────────────────────────────────

#[tokio::test]
#[ignore = "requires Chromium"]
async fn the_keyboard_core_moves_over_visible_items_only() {
    let (_runner, page) = start().await;
    focus(&page, "files-n-src-node").await;
    page.press("ArrowDown").await.unwrap();
    // `src` is closed: its children are skipped.
    assert_eq!(focused(&page).await, "files-n-docs-node");
    page.press("End").await.unwrap();
    assert_eq!(focused(&page).await, "files-n-readme-node");
    page.press("Home").await.unwrap();
    assert_eq!(focused(&page).await, "files-n-src-node");
    page.press("r").await.unwrap();
    assert_eq!(focused(&page).await, "files-n-readme-node");
    assert_eq!(
        eval_string(
            &page,
            "String(document.getElementById('files-n-readme-node').tabIndex)"
        )
        .await,
        "0"
    );
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn the_keyboard_core_moves_into_an_open_branch() {
    let (_runner, page) = start().await;
    // The single tree opens `src`, the ancestor of the selected `lib`.
    focus(&page, "file-n-src-node").await;
    page.press("ArrowDown").await.unwrap();
    assert_eq!(focused(&page).await, "file-n-lib-node");
    page.press("ArrowDown").await.unwrap();
    assert_eq!(focused(&page).await, "file-n-bin-node");
    // `build` is disabled: the core skips it.
    page.press("ArrowDown").await.unwrap();
    assert_eq!(focused(&page).await, "file-n-docs-node");
}

// ── Selection ───────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires Chromium"]
async fn enter_selects_and_focus_does_not() {
    let (_runner, page) = start().await;
    focus(&page, "file-n-lib-node").await;
    page.press("ArrowDown").await.unwrap();
    assert_eq!(focused(&page).await, "file-n-bin-node");
    // Selection does not follow focus.
    assert_eq!(
        attr(&page, "file-n-lib-node", "aria-selected").await,
        "true"
    );
    assert_eq!(
        attr(&page, "file-n-bin-node", "aria-selected").await,
        "false"
    );
    page.press("Enter").await.unwrap();
    assert_eq!(
        attr(&page, "file-n-bin-node", "aria-selected").await,
        "true"
    );
    assert_eq!(
        attr(&page, "file-n-lib-node", "aria-selected").await,
        "false"
    );
    assert!(eval_bool(&page, "document.getElementById('file-n-bin').checked").await);
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn a_click_on_the_row_selects() {
    let (_runner, page) = start().await;
    page.click("#file-n-readme-node > .autumn-tree-row .autumn-tree-label")
        .await
        .unwrap();
    assert_eq!(
        attr(&page, "file-n-readme-node", "aria-selected").await,
        "true"
    );
    assert_eq!(focused(&page).await, "file-n-readme-node");
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn space_checks_a_parent_and_its_children_and_parents_mix() {
    let (_runner, page) = start().await;
    click_toggle(&page, "files-n-src-node").await;
    click_toggle(&page, "files-n-bin-node").await;
    focus(&page, "files-n-bin-node").await;
    page.press(" ").await.unwrap();
    assert_eq!(
        attr(&page, "files-n-bin-node", "aria-checked").await,
        "true"
    );
    assert_eq!(
        attr(&page, "files-n-main-node", "aria-checked").await,
        "true"
    );
    assert_eq!(
        attr(&page, "files-n-cli-node", "aria-checked").await,
        "true"
    );
    // `src` has `lib` unchecked and `build` disabled: it is mixed.
    assert_eq!(
        attr(&page, "files-n-src-node", "aria-checked").await,
        "mixed"
    );
    assert!(
        eval_bool(
            &page,
            "document.getElementById('files-n-src').indeterminate"
        )
        .await
    );
    assert_eq!(
        attr(&page, "files-n-src-node", "aria-describedby").await,
        "files-n-src-node-mixed"
    );
    // Uncheck one child: `bin` is mixed too.
    focus(&page, "files-n-cli-node").await;
    page.press("Enter").await.unwrap();
    assert_eq!(
        attr(&page, "files-n-bin-node", "aria-checked").await,
        "mixed"
    );
    // Check `lib` and `cli`: all enabled children of `src` are checked.
    page.press(" ").await.unwrap();
    focus(&page, "files-n-lib-node").await;
    page.press(" ").await.unwrap();
    assert_eq!(
        attr(&page, "files-n-src-node", "aria-checked").await,
        "true"
    );
    assert!(eval_bool(&page, "document.getElementById('files-n-src').checked").await);
    assert!(!eval_bool(&page, "document.getElementById('files-n-build').checked").await);
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn ctrl_a_checks_all_enabled_items() {
    let (_runner, page) = start().await;
    focus(&page, "files-n-readme-node").await;
    assert!(
        eval_bool(
            &page,
            "(() => { const e = new KeyboardEvent('keydown', { key: 'a', ctrlKey: true, bubbles: true, cancelable: true }); \
             document.activeElement.dispatchEvent(e); return e.defaultPrevented; })()"
        )
        .await,
        "Ctrl+A was not handled"
    );
    assert!(
        eval_bool(
            &page,
            "[...document.querySelectorAll('#files-tree input')].every(i => i.disabled || i.checked)"
        )
        .await
    );
    assert!(!eval_bool(&page, "document.getElementById('files-n-build').checked").await);
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn a_change_event_reaches_the_app() {
    let (_runner, page) = start().await;
    assert!(
        eval_bool(
            &page,
            "(() => { window.changes = 0; document.getElementById('multi-form') \
             .addEventListener('change', () => window.changes++); return true; })()"
        )
        .await
    );
    focus(&page, "files-n-readme-node").await;
    page.press(" ").await.unwrap();
    assert!(eval_bool(&page, "window.changes === 1").await);
}

// ── Open and close ──────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires Chromium"]
async fn the_chevron_opens_and_closes_and_focus_goes_to_the_parent() {
    let (_runner, page) = start().await;
    click_toggle(&page, "files-n-src-node").await;
    assert_eq!(
        attr(&page, "files-n-src-node", "aria-expanded").await,
        "true"
    );
    assert!(!eval_bool(&page, "document.getElementById('files-n-src-group').hidden").await);
    // The chevron does not check the node.
    assert_eq!(
        attr(&page, "files-n-src-node", "aria-checked").await,
        "false"
    );
    focus(&page, "files-n-lib-node").await;
    click_toggle(&page, "files-n-src-node").await;
    assert_eq!(
        attr(&page, "files-n-src-node", "aria-expanded").await,
        "false"
    );
    assert_eq!(focused(&page).await, "files-n-src-node");
}

// ── Lazy children ───────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires Chromium"]
async fn a_lazy_branch_loads_once_and_upgrades_its_children() {
    let (_runner, page) = start().await;
    let before = CHILD_LOADS.load(Ordering::SeqCst);
    click_toggle(&page, "files-n-docs-node").await;
    assert!(
        poll(
            &page,
            "document.getElementById('files-n-guide-node')?.getAttribute('role') === 'treeitem'"
        )
        .await,
        "the lazy children did not load and upgrade"
    );
    let checks = [
        "document.getElementById('files-n-docs-group').getAttribute('aria-busy') === null",
        "document.getElementById('files-n-docs-group').hasAttribute('data-autumn-tree-loaded')",
        "document.getElementById('files-n-guide-node').getAttribute('aria-level') === '2'",
        "document.getElementById('files-n-guide-node').tabIndex === -1",
        "document.getElementById('files-n-guide').hidden",
        "!document.querySelector('#files-n-docs-group .autumn-tree-placeholder')",
    ];
    for js in checks {
        assert!(poll(&page, js).await, "{js}");
    }
    click_toggle(&page, "files-n-docs-node").await;
    click_toggle(&page, "files-n-docs-node").await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(
        CHILD_LOADS.load(Ordering::SeqCst),
        before + 1,
        "loaded more than once"
    );
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn a_checked_lazy_parent_checks_its_new_children() {
    let (_runner, page) = start().await;
    focus(&page, "files-n-docs-node").await;
    page.press(" ").await.unwrap();
    click_toggle(&page, "files-n-docs-node").await;
    assert!(
        poll(
            &page,
            "document.getElementById('files-n-api-node')?.getAttribute('aria-checked') === 'true'"
        )
        .await,
        "the new children are not checked"
    );
    assert_eq!(
        attr(&page, "files-n-docs-node", "aria-checked").await,
        "true"
    );
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn a_failed_branch_shows_an_error_and_tries_again() {
    let (_runner, page) = start().await;
    click_toggle(&page, "files-n-broken-node").await;
    assert!(
        poll(
            &page,
            "document.querySelector('#files-n-broken-group .autumn-tree-placeholder').textContent.startsWith('Could not load')"
        )
        .await,
        "no error text"
    );
    assert!(
        poll(
            &page,
            "document.getElementById('files-n-broken-group').getAttribute('aria-busy') === null"
        )
        .await
    );
    assert!(!eval_bool(&page, "document.getElementById('files-n-broken-group').hasAttribute('data-autumn-tree-loaded')").await);
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn an_empty_branch_becomes_an_end_node() {
    let (_runner, page) = start().await;
    click_toggle(&page, "files-n-empty-node").await;
    assert!(
        poll(
            &page,
            "!document.getElementById('files-n-empty-node').hasAttribute('aria-expanded')"
        )
        .await,
        "the empty branch kept aria-expanded"
    );
    assert!(!eval_bool(&page, "!!document.getElementById('files-n-empty-group')").await);
}

// ── Forms ───────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires Chromium"]
async fn the_hidden_inputs_post_with_the_form() {
    let (_runner, page) = start().await;
    focus(&page, "files-n-readme-node").await;
    page.press(" ").await.unwrap();
    focus(&page, "files-n-src-node").await;
    page.press(" ").await.unwrap();
    page.click("#multi-form button[type=submit]").await.unwrap();
    assert!(
        poll(&page, "location.pathname === '/done'").await,
        "the form did not post"
    );
    let query = eval_string(&page, "location.search").await;
    for key in [
        "files=src",
        "files=lib",
        "files=bin",
        "files=main",
        "files=cli",
        "files=readme",
    ] {
        assert!(query.contains(key), "{key} not in {query}");
    }
    assert!(!query.contains("files=build"), "{query}");
}

// ── No JavaScript ───────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires Chromium"]
async fn with_javascript_off_the_tree_is_a_native_list() {
    let runner = Box::pin(
        SystemTest::new()
            .routes(routes![
                tree_page,
                tree_children,
                tree_broken,
                tree_empty,
                done
            ])
            .plugin_assets(&autumn_plugin_tree::ASSETS)
            .build(),
    )
    .await
    .expect("start runner");
    let page = runner.page().await.expect("page");
    page.set_javascript_enabled(false).await.unwrap();
    page.visit("/tree").await.unwrap();
    assert!(poll(&page, "document.readyState === 'complete'").await);
    let checks = [
        "document.querySelectorAll('[role=tree]').length === 0",
        "document.querySelectorAll('#files-tree details').length === 5",
        "!document.getElementById('files-n-readme').hidden",
        "document.querySelector('#files-n-docs-group .autumn-tree-placeholder a').getAttribute('href') === '/tree/children'",
    ];
    for js in checks {
        assert!(eval_bool(&page, js).await, "{js}");
    }
    page.click("label[for=files-n-readme]").await.unwrap();
    assert!(eval_bool(&page, "document.getElementById('files-n-readme').checked").await);
    page.click("#files-n-src-node > details > summary")
        .await
        .unwrap();
    assert!(
        eval_bool(
            &page,
            "document.querySelector('#files-n-src-node > details').open"
        )
        .await
    );
}

// ── CSP and axe ─────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires Chromium"]
async fn the_page_has_no_console_error() {
    let (_runner, page) = start().await;
    click_toggle(&page, "files-n-src-node").await;
    focus(&page, "files-n-lib-node").await;
    page.press(" ").await.unwrap();
    tokio::time::sleep(Duration::from_millis(250)).await;
    let errors = page.console_errors();
    assert!(errors.is_empty(), "{errors:?}");
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn the_open_tree_has_no_axe_violation() {
    let (_runner, page) = start().await;
    click_toggle(&page, "files-n-src-node").await;
    click_toggle(&page, "files-n-bin-node").await;
    focus(&page, "files-n-bin-node").await;
    page.press(" ").await.unwrap();
    click_toggle(&page, "files-n-docs-node").await;
    assert!(poll(&page, "!!document.getElementById('files-n-api-node')").await);
    let axe = std::fs::read_to_string("tests/fixtures/axe-core/axe.min.js").expect("axe-core");
    let violations = page.axe_violations(&axe).await.expect("axe runs");
    assert!(violations.is_empty(), "{violations:#?}");
}

#[tokio::test]
#[ignore = "requires Chromium"]
async fn the_pack_passes_the_browser_gate() {
    let axe = std::fs::read_to_string("tests/fixtures/axe-core/axe.min.js").expect("axe-core");
    let report = run_widget_browser_conformance(&pack(), &axe)
        .await
        .expect("browser starts");
    assert!(report.passed(), "{}", report.to_text_report());
}

// ── Tree keys of the keyboard core ──────────────────────────────────────

#[cfg(feature = "core-tree-keys")]
mod tree_keys {
    use super::*;

    #[tokio::test]
    #[ignore = "requires Chromium"]
    async fn right_and_left_open_close_and_move() {
        let (_runner, page) = start().await;
        focus(&page, "files-n-src-node").await;
        page.press("ArrowRight").await.unwrap();
        assert_eq!(
            attr(&page, "files-n-src-node", "aria-expanded").await,
            "true"
        );
        assert_eq!(focused(&page).await, "files-n-src-node");
        page.press("ArrowRight").await.unwrap();
        assert_eq!(focused(&page).await, "files-n-lib-node");
        page.press("ArrowLeft").await.unwrap();
        assert_eq!(focused(&page).await, "files-n-src-node");
        page.press("ArrowLeft").await.unwrap();
        assert_eq!(
            attr(&page, "files-n-src-node", "aria-expanded").await,
            "false"
        );
        // Keys do not check a node.
        assert_eq!(
            attr(&page, "files-n-src-node", "aria-checked").await,
            "false"
        );
    }

    #[tokio::test]
    #[ignore = "requires Chromium"]
    async fn right_loads_a_lazy_branch() {
        let (_runner, page) = start().await;
        focus(&page, "files-n-docs-node").await;
        page.press("ArrowRight").await.unwrap();
        assert!(
            poll(
                &page,
                "document.getElementById('files-n-guide-node')?.getAttribute('role') === 'treeitem'"
            )
            .await,
            "Right did not load the branch"
        );
        page.press("ArrowRight").await.unwrap();
        assert_eq!(focused(&page).await, "files-n-guide-node");
    }

    #[tokio::test]
    #[ignore = "requires Chromium"]
    async fn star_opens_the_siblings() {
        let (_runner, page) = start().await;
        focus(&page, "files-n-readme-node").await;
        page.press("*").await.unwrap();
        assert_eq!(
            attr(&page, "files-n-src-node", "aria-expanded").await,
            "true"
        );
        assert!(
            poll(&page, "!!document.getElementById('files-n-api-node')").await,
            "`*` did not load the lazy sibling"
        );
    }
}
