use super::*;
use std::path::Path;

#[test]
fn test_get_sudoku() {
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
    let sudoku = get_sudoku("data/sample_sudoku.txt".to_string()).unwrap();
    let mut solver = cadical::Solver::with_config("plain").unwrap();
    let callback_wrapper = CadicalCallbackWrapper::new(ConstraintList::new(), Trail::new());
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
    let conflict_literals = vec![(100, 101), (300, 301)];
    let trail_data = vec![vec![1, 2, 3], vec![4, 5, 6]];
    let mut trail = Trail::new();

    trail.push(conflict_literals[0], trail_data[0].clone());
    trail.push(conflict_literals[1], trail_data[1].clone());
    assert_eq!(trail.len(), 2);
    assert_eq!(trail.trail_at_index(1), vec![4, 5, 6]);
    assert_eq!(trail.is_empty(), false);

    trail.clear();
    assert_eq!(trail.len(), 0);
    assert_eq!(trail.is_empty(), true);
    assert_eq!(trail.conflict_literals.borrow().len(), 0);
}

#[test]
fn test_get_no_sudoku() {
    let test_file: String = "./data/foo_sudoku.txt".to_string();
    let test_file_exists: bool = Path::new("./data/foo_sudoku.txt").exists();
    let test_getting_sudoku = get_sudoku(test_file);

    assert_eq!(test_file_exists, false);
    assert!(test_getting_sudoku.is_err());
}

#[test]
fn test_get_wrong_filetype() {
    let test_file: String = "./data/foo.exe".to_string();
    let file_exists: bool = Path::new("./data/foo.exe").exists();
    let assumed_error_message = "Invalid filetype!".to_string();
    let test_result = get_sudoku(test_file);

    assert_eq!(file_exists, false);
    assert_eq!(test_result.err().unwrap().msg, assumed_error_message);
}

#[test]
fn test_write_sudoku() {
    let test_text: String = "00000000".to_string();
    let test_path: &Path = Path::new("./data/test_sudoku.txt");
    let written = write_sudoku(test_text, &test_path);
    let read_to_text = fs::read_to_string(test_path).unwrap();

    assert!(written.is_ok());
    assert_eq!(read_to_text, "00000000".to_string());
}

#[test]
fn test_write_no_sudoku() {
    let test_text: String = "".to_string();
    let test_path: &Path = Path::new("");
    let written = write_sudoku(test_text, &test_path);

    assert!(written.is_err());
}

#[test]
fn test_write_no_valid_path() {
    let test_text2: String = "".to_string();
    let test_path2: &Path = Path::new("./foo/foo.txt");
    let path_exists: bool = test_path2.exists();
    let assumed_error_message = "Saving the file failed".to_string();
    let test_result = write_sudoku(test_text2, &test_path2);

    assert_eq!(path_exists, false);
    assert_eq!(test_result.err().unwrap().msg, assumed_error_message);
}
