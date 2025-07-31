//! Metrics library module placeholder.

/// Returns the placeholder string for testing.
///
/// # Examples
///
/// ```
/// use metrics::get_placeholder;
/// assert_eq!("metrics", get_placeholder());
/// ```
pub fn get_placeholder() -> &'static str {
    PLACEHOLDER
}

/// Placeholder public constant used for compile tests.
pub const PLACEHOLDER: &str = "metrics";
