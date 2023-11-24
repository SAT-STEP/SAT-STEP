//! Tests for lib.rs

use super::*;

#[test]
fn test_parse_numeric_input_valid_input() {
    let max_length = String::from("10");

    let applied = parse_numeric_input(&max_length);
    assert_eq!(applied, Some(10));
}

#[test]
fn test_parse_numeric_input_negative() {
    let max_length = String::from("-10");

    let applied = parse_numeric_input(&max_length);
    assert_eq!(applied, None);
}

#[test]
fn test_parse_numeric_input_not_numeric() {
    let max_length = String::from("test");

    let applied = parse_numeric_input(&max_length);
    assert_eq!(applied, None);
}

#[test]
fn test_parse_numeric_input_empty() {
    let max_length = String::new();

    let applied = parse_numeric_input(&max_length);
    assert_eq!(applied, None);
}

#[test]
fn test_constraint_list() {
    let constraints = vec![vec![1, 2, 3], vec![10, 11, 12]];
    let mut c_list = ConstraintList::_new(Rc::new(RefCell::new(constraints)));

    assert_eq!(c_list.len(), 2);

    c_list.push(vec![5, 6, 7]);
    assert_eq!(c_list.len(), 3);

    c_list.clear();
    assert_eq!(c_list.len(), 0);
    assert_eq!(c_list.is_empty(), true);
}

#[test]
fn test_trail() {
    let conflict_literals = vec![vec![100, 101], vec![300, 301]];
    let trail_data = vec![vec![1, 2, 3], vec![4, 5, 6]];
    let mut trail = Trail::new();

    trail.push(conflict_literals[0].clone(), trail_data[0].clone());
    trail.push(conflict_literals[1].clone(), trail_data[1].clone());
    assert_eq!(trail.len(), 2);
    assert_eq!(trail.trail_at_index(1), vec![4, 5, 6]);
    assert_eq!(trail.is_empty(), false);

    trail.clear();
    assert_eq!(trail.len(), 0);
    assert_eq!(trail.is_empty(), true);
    assert_eq!(trail.conflict_literals.borrow().len(), 0);
}
