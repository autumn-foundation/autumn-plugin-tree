//! The widget pack: stories, assets, the plugin and static conformance.

use autumn_plugin_tree::{ASSETS, PACK_NAME, TreePlugin, pack};
use autumn_web::plugin_conformance::widgets::run_widget_conformance;

#[test]
fn the_pack_conforms() {
    let report = run_widget_conformance(&pack());
    assert!(report.passed(), "{}", report.to_text_report());
}

#[test]
fn the_pack_has_a_story_for_each_widget() {
    let p = pack();
    let slugs: Vec<&str> = p.stories().iter().map(|s| s.slug()).collect();
    for slug in [
        "tree-single-tree",
        "tree-multiple-tree",
        "tree-lazy-tree",
        "tree-navigation-tree",
    ] {
        assert!(slugs.contains(&slug), "{slug} not in {slugs:?}");
    }
}

#[test]
fn the_pack_ships_css_and_js() {
    assert_eq!(PACK_NAME, "tree");
    assert_eq!(ASSETS.namespace(), "tree");
    let p = pack();
    assert_eq!(p.stylesheet_urls().len(), 1);
    assert_eq!(p.script_urls().len(), 1);
    let head = p.head().into_string();
    assert!(head.contains("defer"), "{head}");
    assert!(head.contains("integrity="), "{head}");
}

#[test]
fn the_script_has_no_arrow_key_code() {
    let js = asset("tree.js");
    for banned in ["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "'Home'", "'End'", "Escape"] {
        assert!(!js.contains(banned), "tree.js must use the keyboard core, found {banned}");
    }
    assert!(js.contains("autumn:init"), "tree.js sets up on autumn:init");
}

#[test]
fn the_css_has_no_raw_color() {
    let css = asset("tree.css");
    assert!(!css.contains('#'), "no hex colors");
    assert!(!css.contains("rgb("), "no rgb colors");
}

#[test]
fn the_plugin_declares_its_contract_and_installs_the_pack() {
    use autumn_web::plugin::Plugin;
    let contract = TreePlugin.contract().expect("a contract");
    assert_eq!(contract.label(), format!("autumn-plugin-tree {}", env!("CARGO_PKG_VERSION")));
    let verdict = autumn_web::plugin_contract::evaluate(&contract, "0.10.0");
    assert_eq!(verdict, autumn_web::plugin_contract::ContractVerdict::Compatible);
}

fn asset(name: &str) -> String {
    let file = ASSETS.get(name).unwrap_or_else(|| panic!("{name} in the bundle"));
    String::from_utf8(file.bytes().to_vec()).expect("utf-8")
}
