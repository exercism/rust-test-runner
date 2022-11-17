#[test]
fn test_name_is_formatted_0() {
    let input = String::from("test_year_divisible_by_400_but_not_by_125_is_still_a_leap_year");
    let output = String::from("Year divisible by 400 but not by 125 is still a leap year");
    assert_eq!(formatter::format_test_name(input), output);
}
