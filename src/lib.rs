//! A tree widget pack for [Autumn](https://github.com/autumn-foundation/autumn).
//!
//! The pack has two widgets:
//!
//! - [`TreeField`]: a form field. The server sends a nested list of radios
//!   or checkboxes in `<details>`. With JavaScript on, `tree.js` changes it
//!   into a WAI-ARIA APG tree (`role="tree"`).
//! - [`nav_tree`]: navigation links in nested `<details>`. It is not a
//!   `role="tree"`, because a tree is the wrong pattern for site navigation.
//!
//! Install the pack with [`TreePlugin`]. Link `pack().head()` (see [`pack`]) in the
//! layout.

mod budget;
mod field;
mod flat;
mod href;
mod key;
mod nav;
mod node;
mod pack;

pub use budget::{MAX_DEPTH, MAX_NODES};
pub use field::{Cascade, TreeField, TreeText, tree_field};
pub use href::safe_href;
pub use key::TreeKey;
pub use nav::nav_tree;
pub use node::{Children, TreeNode};
pub use pack::{ASSETS, PACK_NAME, TreePlugin, pack};
