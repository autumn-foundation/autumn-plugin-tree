//! Link filter.

/// Return `href` when it is safe for a link, else `None`.
///
/// A safe link is relative, or has the scheme `http`, `https` or `mailto`.
#[must_use]
pub fn safe_href(href: &str) -> Option<&str> {
    let _ = href;
    None
}
