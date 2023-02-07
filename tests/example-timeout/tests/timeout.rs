use example_timeout::do_something_forever;

#[test]
fn test_infinite_loop() {
    assert_eq!(0, do_something_forever());
}