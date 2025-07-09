use docsbookgen::tool_name;

#[test]
fn tool_name_returns_expected_value() {
    assert_eq!("docsbookgen", tool_name());
}
