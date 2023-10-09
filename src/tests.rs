#[test]
fn test_get_sudoku() {
    use super::*;

    let sudoku = get_sudoku("data/sample_sudoku.txt".to_string()).unwrap();
    let should_be = vec![
        vec![None, None, None, None, None, None, None, Some(1), None],
        vec![Some(4), None, None, None, None, None, None, None, None],
        vec![None, Some(2), None, None, None, None, None, None, None],
        vec![
            None,
            None,
            None,
            None,
            Some(5),
            None,
            Some(4),
            None,
            Some(7),
        ],
        vec![None, None, Some(8), None, None, None, Some(3), None, None],
        vec![None, None, Some(1), None, Some(9), None, None, None, None],
        vec![
            Some(3),
            None,
            None,
            Some(4),
            None,
            None,
            Some(2),
            None,
            None,
        ],
        vec![None, Some(5), None, Some(1), None, None, None, None, None],
        vec![None, None, None, Some(8), None, Some(6), None, None, None],
    ];
    assert_eq!(sudoku, should_be);
}

#[test]
fn test_solve_sudoku() {
    use super::*;

    let sudoku = get_sudoku("data/sample_sudoku.txt".to_string()).unwrap();
    let mut solver = cadical::Solver::with_config("plain").unwrap();
    let callback_wrapper = CadicalCallbackWrapper::new(ConstraintList::new());
    solver.set_callbacks(Some(callback_wrapper.clone()));

    let solved = solve_sudoku(&sudoku, &mut solver).unwrap();
    let should_be = vec![
        vec![
            Some(6),
            Some(9),
            Some(3),
            Some(7),
            Some(8),
            Some(4),
            Some(5),
            Some(1),
            Some(2),
        ],
        vec![
            Some(4),
            Some(8),
            Some(7),
            Some(5),
            Some(1),
            Some(2),
            Some(9),
            Some(3),
            Some(6),
        ],
        vec![
            Some(1),
            Some(2),
            Some(5),
            Some(9),
            Some(6),
            Some(3),
            Some(8),
            Some(7),
            Some(4),
        ],
        vec![
            Some(9),
            Some(3),
            Some(2),
            Some(6),
            Some(5),
            Some(1),
            Some(4),
            Some(8),
            Some(7),
        ],
        vec![
            Some(5),
            Some(6),
            Some(8),
            Some(2),
            Some(4),
            Some(7),
            Some(3),
            Some(9),
            Some(1),
        ],
        vec![
            Some(7),
            Some(4),
            Some(1),
            Some(3),
            Some(9),
            Some(8),
            Some(6),
            Some(2),
            Some(5),
        ],
        vec![
            Some(3),
            Some(1),
            Some(9),
            Some(4),
            Some(7),
            Some(5),
            Some(2),
            Some(6),
            Some(8),
        ],
        vec![
            Some(8),
            Some(5),
            Some(6),
            Some(1),
            Some(2),
            Some(9),
            Some(7),
            Some(4),
            Some(3),
        ],
        vec![
            Some(2),
            Some(7),
            Some(4),
            Some(8),
            Some(3),
            Some(6),
            Some(1),
            Some(5),
            Some(9),
        ],
    ];
    assert_eq!(solved, should_be);
}

#[test]
fn test_parse_numeric_input_valid_input() {
    use super::*;

    let max_length = String::from("10");

    let applied = parse_numeric_input(&max_length);
    assert_eq!(applied, Some(10));
}

#[test]
fn test_parse_numeric_input_negative() {
    use super::*;

    let max_length = String::from("-10");

    let applied = parse_numeric_input(&max_length);
    assert_eq!(applied, None);
}

#[test]
fn test_parse_numeric_input_not_numeric() {
    use super::*;

    let max_length = String::from("test");

    let applied = parse_numeric_input(&max_length);
    assert_eq!(applied, None);
}

#[test]
fn test_parse_numeric_input_empty() {
    use super::*;

    let max_length = String::new();

    let applied = parse_numeric_input(&max_length);
    assert_eq!(applied, None);
}

#[test]
fn test_constraint_list() {
    use super::*;

    let constraints = vec![vec![1, 2, 3], vec![10, 11, 12]];
    let mut c_list = ConstraintList::_new(Rc::new(RefCell::new(constraints)));

    assert_eq!(c_list.len(), 2);

    c_list.push(vec![5, 6, 7]);
    assert_eq!(c_list.len(), 3);

    c_list.clear();
    assert_eq!(c_list.len(), 0);
    assert_eq!(c_list.is_empty(), true);
}
