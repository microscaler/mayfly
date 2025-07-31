//! Write-ahead log library module placeholder.

/// Placeholder public constant used for compile tests.
pub const PLACEHOLDER: &str = "wal";

/// Returns the placeholder string for testing.
///
/// # Examples
///
/// ```
/// use wal::get_placeholder;
/// assert_eq!("wal", get_placeholder());
/// ```
pub fn get_placeholder() -> &'static str {
    PLACEHOLDER
}
