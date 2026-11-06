//! Link filter.

/// The schemes that a node link can use.
const SAFE_SCHEMES: [&str; 3] = ["http", "https", "mailto"];

/// Return `href` when it is safe for a link, else `None`.
///
/// A safe link is relative, or has the scheme `http`, `https` or `mailto`.
/// The check reads the link as a browser does: it ignores tab and newline
/// characters, and leading spaces and control characters.
///
/// ```
/// use autumn_plugin_tree::safe_href;
///
/// assert_eq!(safe_href("/docs"), Some("/docs"));
/// assert_eq!(safe_href("https://example.com"), Some("https://example.com"));
/// assert_eq!(safe_href("javascript:alert(1)"), None);
/// ```
#[must_use]
pub fn safe_href(href: &str) -> Option<&str> {
    let cleaned: String = href
        .chars()
        .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
        .collect();
    let url = cleaned.trim_start_matches(|c: char| c <= ' ');
    let Some(end) = url.find([':', '/', '?', '#']) else {
        return Some(href);
    };
    if !url[end..].starts_with(':') {
        return Some(href);
    }
    let scheme = &url[..end];
    let is_scheme = scheme.starts_with(|c: char| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    if !is_scheme {
        // The browser reads this as a relative path.
        return Some(href);
    }
    SAFE_SCHEMES
        .iter()
        .any(|safe| scheme.eq_ignore_ascii_case(safe))
        .then_some(href)
}
