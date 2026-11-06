//! The tree field: the server markup (the JavaScript-off baseline).

mod common;

use autumn_plugin_tree::{Cascade, TreeField, TreeNode, TreeText, tree_field};
use autumn_web::form::{Changeset, FieldControl, FieldOptions, FormField, FormModel, form_for};
use common::{attr, files, one, parse, select, text};
use serde::Serialize;

#[derive(Serialize, Default)]
struct Pick {
    file: String,
    files: Vec<String>,
}

impl FormModel for Pick {
    fn form_fields() -> Vec<FormField> {
        vec![FormField::new("file", "File", FieldControl::Text, false)]
    }
}

fn single(value: &str) -> String {
    let changeset = Changeset::new(Pick { file: value.into(), ..Pick::default() });
    tree_field(&changeset, "file", "File", &TreeField::single(files()), &FieldOptions::new())
        .into_string()
}

fn multiple(values: &[&str], tree: &TreeField) -> String {
    let changeset = Changeset::new(Pick {
        files: values.iter().map(|v| (*v).to_owned()).collect(),
        ..Pick::default()
    });
    tree_field(&changeset, "files", "Files", tree, &FieldOptions::new()).into_string()
}

#[test]
fn the_field_is_a_fieldset_with_a_legend() {
    let doc = parse(&single(""));
    assert_eq!(text(one(&doc, "fieldset > legend")), "File");
}

#[test]
fn the_root_is_a_list_with_the_tree_data_attributes() {
    let doc = parse(&single(""));
    let root = one(&doc, "fieldset > ul");
    assert_eq!(attr(root, "id"), Some("file-tree"));
    assert_eq!(attr(root, "aria-label"), Some("File"));
    assert_eq!(attr(root, "data-autumn-tree"), Some("single"));
    assert!(attr(root, "class").unwrap_or_default().contains("autumn-tree"));
}

#[test]
fn the_server_markup_has_no_tree_role() {
    let html = single("");
    assert!(!html.contains("role=\"tree"), "{html}");
    assert!(!html.contains("treeitem"), "{html}");
}

#[test]
fn a_single_tree_has_one_radio_per_node() {
    let doc = parse(&single(""));
    let radios = select(&doc, "input[type=radio]");
    let values: Vec<&str> = radios.iter().filter_map(|r| attr(*r, "value")).collect();
    assert_eq!(values, ["src", "lib", "bin", "main", "readme"]);
    for radio in radios {
        assert_eq!(attr(radio, "name"), Some("file"));
    }
}

#[test]
fn ids_start_with_the_field_and_use_the_key_token() {
    let doc = parse(&single(""));
    let input = one(&doc, "input[value=lib]");
    assert_eq!(attr(input, "id"), Some("file-n-lib"));
    assert_eq!(attr(input, "aria-labelledby"), Some("file-n-lib-label"));
    assert_eq!(text(one(&doc, "#file-n-lib-label")), "lib.rs");
    one(&doc, "li#file-n-lib-node");
}

#[test]
fn an_end_node_has_a_label_for_its_input() {
    let doc = parse(&single(""));
    let label = one(&doc, "label[for=file-n-readme]");
    assert_eq!(text(label), "README.md");
}

#[test]
fn a_parent_puts_its_input_before_a_details() {
    let doc = parse(&single(""));
    let node = one(&doc, "li#file-n-src-node");
    let first = node.children().filter_map(scraper::ElementRef::wrap).next().map(|e| e.value().name());
    assert_eq!(first, Some("input"));
    one(&doc, "li#file-n-src-node > details > summary > span#file-n-src-label");
    one(&doc, "li#file-n-src-node > details > ul.autumn-tree-group#file-n-src-group");
    // No input in a summary.
    assert!(select(&doc, "summary input").is_empty());
}

#[test]
fn each_node_declares_its_level_and_position() {
    let doc = parse(&single(""));
    let expect = [
        ("src", "1", "2", "1"),
        ("lib", "2", "2", "1"),
        ("bin", "2", "2", "2"),
        ("main", "3", "1", "1"),
        ("readme", "1", "2", "2"),
    ];
    for (key, level, size, pos) in expect {
        let node = one(&doc, &format!("li#file-n-{key}-node"));
        assert_eq!(attr(node, "aria-level"), Some(level), "{key}");
        assert_eq!(attr(node, "aria-setsize"), Some(size), "{key}");
        assert_eq!(attr(node, "aria-posinset"), Some(pos), "{key}");
    }
}

#[test]
fn the_checked_radio_comes_from_the_changeset_and_its_ancestors_are_open() {
    let doc = parse(&single("main"));
    let checked: Vec<&str> =
        select(&doc, "input:checked").iter().filter_map(|i| attr(*i, "value")).collect();
    assert_eq!(checked, ["main"]);
    let open: Vec<&str> = select(&doc, "details[open]")
        .iter()
        .filter_map(|d| d.parent().and_then(scraper::ElementRef::wrap))
        .filter_map(|li| attr(li, "id"))
        .collect();
    assert_eq!(open, ["file-n-src-node", "file-n-bin-node"]);
}

#[test]
fn expanded_keys_open_their_nodes() {
    let tree = TreeField::single(files()).expanded(["bin"]);
    let changeset = Changeset::new(Pick::default());
    let doc = parse(&tree_field(&changeset, "file", "File", &tree, &FieldOptions::new()).into_string());
    assert_eq!(select(&doc, "details[open]").len(), 1);
    one(&doc, "li#file-n-bin-node > details[open]");
}

#[test]
fn a_multiple_tree_has_checkboxes_and_says_its_mode() {
    let html = multiple(&[], &TreeField::multiple(files()));
    let doc = parse(&html);
    assert_eq!(select(&doc, "input[type=checkbox]").len(), 5);
    assert!(select(&doc, "input[type=radio]").is_empty());
    let root = one(&doc, "ul.autumn-tree");
    assert_eq!(attr(root, "data-autumn-tree"), Some("multiple"));
    assert_eq!(attr(root, "data-autumn-tree-cascade"), Some("descendants"));
}

#[test]
fn a_checked_parent_checks_its_descendants_with_cascade() {
    let doc = parse(&multiple(&["src"], &TreeField::multiple(files())));
    let checked: Vec<&str> =
        select(&doc, "input:checked").iter().filter_map(|i| attr(*i, "value")).collect();
    assert_eq!(checked, ["src", "lib", "bin", "main"]);
}

#[test]
fn a_parent_with_some_checked_children_is_mixed() {
    let doc = parse(&multiple(&["main"], &TreeField::multiple(files())));
    let mixed: Vec<&str> = select(&doc, "li[data-autumn-tree-mixed]")
        .iter()
        .filter_map(|li| attr(*li, "id"))
        .collect();
    assert_eq!(mixed, ["files-n-src-node"]);
    // `bin` has one child, and it is checked, so `bin` is checked.
    let checked: Vec<&str> =
        select(&doc, "input:checked").iter().filter_map(|i| attr(*i, "value")).collect();
    assert_eq!(checked, ["bin", "main"]);
    // A mixed parent has a hidden text for screen readers.
    let note = one(&doc, "li#files-n-src-node > details > summary .autumn-tree-mixed");
    assert_eq!(text(note), "Partly selected");
}

#[test]
fn independent_nodes_do_not_cascade_or_mix() {
    let tree = TreeField::multiple(files()).cascade(Cascade::Independent);
    let doc = parse(&multiple(&["src"], &tree));
    let checked: Vec<&str> =
        select(&doc, "input:checked").iter().filter_map(|i| attr(*i, "value")).collect();
    assert_eq!(checked, ["src"]);
    assert_eq!(attr(one(&doc, "ul.autumn-tree"), "data-autumn-tree-cascade"), Some("independent"));
    let doc = parse(&multiple(&["main"], &tree));
    assert!(select(&doc, "[data-autumn-tree-mixed]").is_empty());
}

#[test]
fn a_disabled_node_has_a_disabled_input_and_does_not_count_for_its_parent() {
    let nodes = vec![TreeNode::new("p", "P").children([
        TreeNode::new("a", "A"),
        TreeNode::new("b", "B").disabled(),
    ])];
    let doc = parse(&multiple(&["a"], &TreeField::multiple(nodes)));
    assert!(select(&doc, "input[value=b]:disabled").len() == 1);
    // `b` is disabled, so `a` is all the parent has: the parent is checked.
    let checked: Vec<&str> =
        select(&doc, "input:checked").iter().filter_map(|i| attr(*i, "value")).collect();
    assert_eq!(checked, ["p", "a"]);
}

#[test]
fn a_lazy_node_has_an_empty_group_with_a_load_trigger_and_a_fallback_link() {
    let nodes = vec![TreeNode::new("arch", "Archive").lazy("/files/children?node=arch")];
    let doc = parse(&multiple(&[], &TreeField::multiple(nodes)));
    let group = one(&doc, "li#files-n-arch-node > details > ul.autumn-tree-group");
    assert_eq!(attr(group, "data-autumn-tree-src"), Some("/files/children?node=arch"));
    assert_eq!(attr(group, "hx-get"), Some("/files/children?node=arch"));
    assert_eq!(attr(group, "hx-trigger"), Some("autumn-tree:load"));
    assert_eq!(attr(group, "hx-swap"), Some("innerHTML"));
    let link = one(&doc, "li.autumn-tree-placeholder > a");
    assert_eq!(attr(link, "href"), Some("/files/children?node=arch"));
    assert_eq!(text(link), "Show contents");
    assert!(select(&doc, "details[open]").is_empty());
}

#[test]
fn the_texts_can_change() {
    let text_set = TreeText {
        loading: "Laden…".into(),
        failed: "Fehler".into(),
        show: "Anzeigen".into(),
        mixed: "Teilweise".into(),
    };
    let nodes = vec![
        TreeNode::new("arch", "Archiv").lazy("/c"),
        TreeNode::new("p", "P").children([TreeNode::new("a", "A"), TreeNode::new("b", "B")]),
    ];
    let html = multiple(&["a"], &TreeField::multiple(nodes).text(text_set));
    let doc = parse(&html);
    assert_eq!(text(one(&doc, "li.autumn-tree-placeholder > a")), "Anzeigen");
    assert_eq!(text(one(&doc, ".autumn-tree-mixed")), "Teilweise");
    let root = one(&doc, "ul.autumn-tree");
    assert_eq!(attr(root, "data-autumn-tree-loading"), Some("Laden…"));
    assert_eq!(attr(root, "data-autumn-tree-failed"), Some("Fehler"));
}

#[test]
fn the_default_texts_are_english() {
    let t = TreeText::default();
    assert_eq!(t.loading, "Loading…");
    assert_eq!(t.failed, "Could not load. Open the branch again to try again.");
    assert_eq!(t.show, "Show contents");
    assert_eq!(t.mixed, "Partly selected");
}

#[test]
fn lines_and_compact_add_modifier_classes() {
    let html = multiple(&[], &TreeField::multiple(files()).lines().compact());
    let doc = parse(&html);
    let class = attr(one(&doc, "ul.autumn-tree"), "class").unwrap_or_default().to_owned();
    assert!(class.contains("autumn-tree--lines"), "{class}");
    assert!(class.contains("autumn-tree--compact"), "{class}");
}

#[test]
fn labels_and_keys_are_escaped() {
    let nodes = vec![TreeNode::new("\"><script>", "<img src=x onerror=alert(1)>")];
    let html = multiple(&[], &TreeField::multiple(nodes));
    assert!(!html.contains("<script>"), "{html}");
    assert!(!html.contains("<img"), "{html}");
    let doc = parse(&html);
    let input = one(&doc, "input[type=checkbox]");
    assert_eq!(attr(input, "value"), Some("\"><script>"));
    assert_eq!(attr(input, "id"), Some("files-n-_22_3e_3cscript_3e"));
}

#[test]
fn the_app_passthrough_goes_on_each_input() {
    let changeset = Changeset::new(Pick::default());
    let options = FieldOptions::new().hx("post", "/pick").hx("trigger", "change");
    let doc = parse(
        &tree_field(&changeset, "file", "File", &TreeField::single(files()), &options).into_string(),
    );
    let inputs = select(&doc, "input[type=radio]");
    assert_eq!(inputs.len(), 5);
    for input in inputs {
        assert_eq!(attr(input, "hx-post"), Some("/pick"));
    }
}

#[test]
fn a_required_multiple_tree_marks_the_root_not_each_box() {
    let changeset = Changeset::new(Pick::default());
    let options = FieldOptions::new().required();
    let doc = parse(
        &tree_field(&changeset, "files", "Files", &TreeField::multiple(files()), &options)
            .into_string(),
    );
    assert_eq!(attr(one(&doc, "ul.autumn-tree"), "aria-required"), Some("true"));
    // `required` on a box means "check this box", so no box gets it.
    assert!(select(&doc, "input[required]").is_empty());
}

#[test]
fn a_required_single_tree_keeps_required_on_the_radios() {
    let changeset = Changeset::new(Pick::default());
    let options = FieldOptions::new().required();
    let doc = parse(
        &tree_field(&changeset, "file", "File", &TreeField::single(files()), &options).into_string(),
    );
    assert_eq!(select(&doc, "input[type=radio][required]").len(), 5);
    assert_eq!(attr(one(&doc, "ul.autumn-tree"), "aria-required"), Some("true"));
}

#[test]
fn errors_mark_the_inputs_invalid() {
    let mut changeset = Changeset::new(Pick::default());
    changeset.add_error("file", "Pick a file.");
    let doc = parse(
        &tree_field(&changeset, "file", "File", &TreeField::single(files()), &FieldOptions::new())
            .into_string(),
    );
    assert_eq!(attr(one(&doc, "fieldset"), "aria-invalid"), Some("true"));
    for input in select(&doc, "input[type=radio]") {
        assert_eq!(attr(input, "aria-invalid"), Some("true"));
    }
}

#[test]
fn form_for_binds_the_tree_with_a_custom_control() {
    let changeset = Changeset::new(Pick { file: "lib".into(), ..Pick::default() });
    let html = form_for(&changeset, "/pick", "post")
        .override_field("file", FieldControl::custom(TreeField::single(files())))
        .render()
        .into_string();
    let doc = parse(&html);
    one(&doc, "form fieldset ul.autumn-tree");
    assert_eq!(attr(one(&doc, "input:checked"), "value"), Some("lib"));
}

#[test]
fn every_class_is_in_the_pack_namespace_or_core() {
    let html = multiple(&["main"], &TreeField::multiple(files()).lines());
    let doc = parse(&html);
    for el in select(&doc, "[class]") {
        for class in attr(el, "class").unwrap_or_default().split_whitespace() {
            assert!(
                class.starts_with("autumn-tree") || class.starts_with("autumn-field"),
                "{class}"
            );
        }
    }
}

#[test]
fn branch_renders_lazy_children_at_their_level() {
    let tree = TreeField::multiple(Vec::new());
    let children = [TreeNode::new("a", "A"), TreeNode::new("b", "B").child(TreeNode::new("c", "C"))];
    let html = tree.branch("files", 2, &children, &["c"]).into_string();
    let doc = parse(&html);
    let a = one(&doc, "li#files-n-a-node");
    assert_eq!(attr(a, "aria-level"), Some("2"));
    assert_eq!(attr(a, "aria-setsize"), Some("2"));
    assert_eq!(attr(a, "aria-posinset"), Some("1"));
    assert_eq!(attr(one(&doc, "li#files-n-c-node"), "aria-level"), Some("3"));
    let input = one(&doc, "input[value=c]");
    assert_eq!(attr(input, "type"), Some("checkbox"));
    assert_eq!(attr(input, "name"), Some("files"));
    assert!(input.value().attr("checked").is_some());
    // `b` has all children checked.
    assert!(one(&doc, "input[value=b]").value().attr("checked").is_some());
    assert!(!html.contains("<ul class=\"autumn-tree"), "a branch is items only: {html}");
}

#[test]
fn branch_of_a_single_tree_has_radios() {
    let tree = TreeField::single(Vec::new());
    let html = tree.branch("file", 1, &[TreeNode::new("a", "A")], &["a"]).into_string();
    let doc = parse(&html);
    let input = one(&doc, "input[type=radio]");
    assert!(input.value().attr("checked").is_some());
}

#[test]
fn the_renderer_name_and_group_flag() {
    use autumn_web::form::FieldRenderer;
    let tree = TreeField::single(files());
    assert_eq!(tree.name(), "tree");
    assert!(tree.grouped());
    assert!(!tree.multipart());
    assert_eq!(tree.nodes().len(), 2);
}
