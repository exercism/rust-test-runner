#[test]
fn test_regular_test_name() {
    let input = String::from("test_year_divisible_by_400_but_not_by_125_is_still_a_leap_year");
    let output = String::from("Year divisible by 400 but not by 125 is still a leap year");
    assert_eq!(formatter::format_test_name(input), output);
}

#[test]
fn test_trailing_whitespace() {
    let input = String::from("test_reviving_dead_level9_player  ");
    let output = String::from("Reviving dead level9 player");
    assert_eq!(formatter::format_test_name(input), output);
}

#[test]
fn test_extra_underscore() {
    let input = String::from("test___cast_spell_with_insufficient_mana");
    let output = String::from("Cast spell with insufficient mana");
    assert_eq!(formatter::format_test_name(input), output);
}

#[test]
fn test_trailing_extra_underscore() {
    let input = String::from("test_cast_large_spell_with_no_mana_pool__");
    let output = String::from("Cast large spell with no mana pool");
    assert_eq!(formatter::format_test_name(input), output);
}

#[test]
fn test_extra_underscore_at_beginning() {
    let input = String::from("__test_using_ascii_value_for_doubled_nondigit_isnt_allowed");
    let output = String::from("Using ascii value for doubled nondigit isnt allowed");
    assert_eq!(formatter::format_test_name(input), output);
}

#[test]
fn test_extra_underscore_middle() {
    let input =
        String::from("test_valid_strings_with____a_nondigit_added_at_the_end_become_invalid");
    let output = String::from("Valid strings with a nondigit added at the end become invalid");
    assert_eq!(formatter::format_test_name(input), output);
}

#[test]
fn test_several_extra_whitespaces() {
    let input =
        String::from("test_invalid _char _in _middle _with_sum_divisible_by_10_isnt_allowed");
    let output = String::from("Invalid char in middle with sum divisible by 10 isnt allowed");
    assert_eq!(formatter::format_test_name(input), output);
}

#[test]
fn test_uppercase_in_the_middle_with_multiple_underscores() {
    let input =
        String::from("test__Valid_strIngs_wIth_nuMeric__Unicode_charActers_become__invaliD");
    let output = String::from("Valid strings with numeric unicode characters become invalid");
    assert_eq!(formatter::format_test_name(input), output);
}

#[test]
fn test_no_test_prefix() {
    let input = String::from("to_quadruple_byte");
    let output = String::from("To quadruple byte");
    assert_eq!(formatter::format_test_name(input), output);
}

#[test]
fn test_short_name() {
    let input = String::from("from_bytes");
    let output = String::from("From bytes");
    assert_eq!(formatter::format_test_name(input), output);
}

#[test]
fn test_remove_multiple_underscores_and_whitespaces() {
    let input = String::from("_ _ _apple_ _ _");
    let output = String::from("Apple");
    assert_eq!(formatter::format_test_name(input), output);
}
