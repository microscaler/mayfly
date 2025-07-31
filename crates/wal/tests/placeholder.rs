use wal::get_placeholder;

#[test]
fn placeholder_returns_expected_value() {
    assert_eq!("wal", get_placeholder());
}
