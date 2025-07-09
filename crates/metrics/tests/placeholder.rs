use metrics::get_placeholder;

#[test]
fn placeholder_returns_expected_value() {
    assert_eq!("metrics", get_placeholder());
}
