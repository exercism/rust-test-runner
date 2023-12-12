use two_plus_two::two_plus_two;

#[test]
fn is_four() {
    assert_eq!(two_plus_two(), 4);
}

#[test]
fn is_five() {
    assert_eq!(two_plus_two(), 5);
}
