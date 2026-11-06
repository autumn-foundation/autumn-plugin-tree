//! Render budgets.

/// The deepest level that a widget renders. A deeper node is not rendered.
pub const MAX_DEPTH: usize = 64;

/// The most nodes that one widget renders. The renderer stops at this count.
pub const MAX_NODES: usize = 10_000;
