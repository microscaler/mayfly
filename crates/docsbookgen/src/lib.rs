//! Internal utilities for the docsbookgen binary.

/// The name of the docs generation tool used for testing.
pub const TOOL_NAME: &str = "docsbookgen";

/// Returns the tool name.
///
/// # Examples
///
/// ```
/// use docsbookgen::tool_name;
/// assert_eq!("docsbookgen", tool_name());
/// ```
pub fn tool_name() -> &'static str {
    TOOL_NAME
}
