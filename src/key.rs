//! Node keys.

/// The key of a tree node. The form posts the key as the value.
///
/// Keys must be unique in one tree.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TreeKey(String);

impl TreeKey {
    /// Make a key.
    #[must_use]
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    /// The key text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The key as an id token: `[A-Za-z0-9_]` only.
    ///
    /// Each other byte becomes `_` and two hex digits. The map is
    /// one-to-one, so two keys never give one token.
    #[must_use]
    pub fn token(&self) -> String {
        String::new()
    }
}

impl From<&str> for TreeKey {
    fn from(key: &str) -> Self {
        Self::new(key)
    }
}

impl From<String> for TreeKey {
    fn from(key: String) -> Self {
        Self(key)
    }
}

impl std::fmt::Display for TreeKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
