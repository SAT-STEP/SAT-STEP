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
fn test_apply_max_length_valid_input() {
    use super::*;

    let max_length = String::from("10");

    let applied = apply_max_length(&max_length);
    assert_eq!(applied, Some(10));
}

#[test]
fn test_apply_max_length_negative() {
    use super::*;

    let max_length = String::from("-10");

    let applied = apply_max_length(&max_length);
    assert_eq!(applied, None);
}

#[test]
fn test_apply_max_length_not_numeric() {
    use super::*;

    let max_length = String::from("test");

    let applied = apply_max_length(&max_length);
    assert_eq!(applied, None);
}

#[test]
fn test_apply_max_length_empty() {
    use super::*;

    let max_length = String::new();

    let applied = apply_max_length(&max_length);
    assert_eq!(applied, None);
}

#[test]
fn test_filter_by_max_length() {
    use super::*;
    let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
        vec![0; 10],
        vec![0; 3],
        vec![0; 5],
    ])));
    let mut filter: ListFilter = ListFilter::new(constraints.clone());

    filter.by_max_length(4);
    let filtered = filter.get_filtered();
    assert_eq!(filtered.len(), 1);

    filter.by_max_length(5);
    let filtered2 = filter.get_filtered();
    assert_eq!(filtered2.len(), 2);

    filter.by_max_length(1);
    let filtered3 = filter.get_filtered();
    assert_eq!(filtered3.len(), 0)
}

#[test]
fn test_filter_by_cell() {
    use super::*;
    let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
        vec![1; 10],
        vec![10; 3],
        vec![10; 3],
    ])));
    let mut filter: ListFilter = ListFilter::new(constraints.clone());
    filter.reinit();

    filter.by_cell(1, 1);
    let filtered = filter.get_filtered();
    assert_eq!(filtered.len(), 1);

    filter.by_cell(1, 2);
    let filtered2 = filter.get_filtered();
    assert_eq!(filtered2.len(), 2);

    filter.by_cell(2, 2);
    let filtered3 = filter.get_filtered();
    assert_eq!(filtered3.len(), 0);
}

#[test]
fn test_clear_filters_and_multiple_filters() {
    use super::*;
    let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
        vec![1; 10],
        vec![1; 3],
        vec![10; 3],
    ])));
    let mut filter: ListFilter = ListFilter::new(constraints);
    filter.reinit();

    filter.by_cell(1, 1);
    let filtered = filter.get_filtered();
    assert_eq!(filtered.len(), 2);
    filter.by_max_length(3);
    let filtered2 = filter.get_filtered();
    assert_eq!(filtered2.len(), 1);
    filter.clear_cell();
    let cleared = filter.get_filtered();
    assert_eq!(cleared.len(), 2);
    filter.clear_length();
    let cleared2 = filter.get_filtered();
    assert_eq!(cleared2.len(), 3);

    let _ = filter.by_cell(1, 1);
    filter.by_max_length(3);
    let filtered4 = filter.get_filtered();
    assert_eq!(filtered4.len(), 1);
    filter.clear_all();
    let cleared3 = filter.get_filtered();
    assert_eq!(cleared3.len(), 3);
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
