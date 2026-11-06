//! Render budgets.

/// The deepest level that a widget renders. A deeper node is not rendered.
pub const MAX_DEPTH: usize = 64;

/// The most nodes that one widget renders. The renderer stops at this count.
pub const MAX_NODES: usize = 10_000;

/// The nodes that one render can still write.
#[derive(Debug)]
pub(crate) struct Budget {
    left: usize,
}

impl Budget {
    /// A full budget.
    pub(crate) const fn new() -> Self {
        Self { left: MAX_NODES }
    }

    /// Take one node. `false` when the budget is empty.
    pub(crate) const fn take(&mut self) -> bool {
        if self.left == 0 {
            return false;
        }
        self.left -= 1;
        true
    }
}

/// `true` when a node at `depth` can show its children.
pub(crate) const fn can_descend(depth: usize) -> bool {
    depth < MAX_DEPTH
}
