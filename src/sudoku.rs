//! Functions focused on the Sudoku puzzle itself

use crate::{app_state::EncodingType, CadicalCallbackWrapper, GenericError, Solver};
use std::{fs, path::Path};

pub fn solve_sudoku(
    sudoku_clues: &[Vec<Option<i32>>],
    solver: &mut Solver<CadicalCallbackWrapper>,
    encoding: &EncodingType,
) -> Result<Vec<Vec<Option<i32>>>, String> {
    let mut solved: Vec<Vec<Option<i32>>> = Vec::new();
    let cnf_clauses = encoding.sudoku_to_cnf(sudoku_clues);

    for clause in cnf_clauses {
        solver.add_clause(clause);
    }

    if let Some(true) = solver.solve() {
        for row in 1..=9 {
            let mut row_values = Vec::with_capacity(9);
            for col in 1..=9 {
                let value = encoding.get_cell_value(solver, row, col);
                row_values.push(Some(value));
            }
            solved.push(row_values);
        }
        return Ok(solved);
    }
    Err(String::from("Solving sudoku failed!"))
}

/// Read sudoku from file
pub fn get_sudoku(filename: String) -> Result<Vec<Vec<Option<i32>>>, GenericError> {
    let sudoku_result = fs::read_to_string(filename);
    match sudoku_result {
        Ok(sudoku) => clues_from_string(sudoku, "."),
        Err(_) => Err(GenericError {
            msg: "Invalid filetype!".to_string(),
        }),
    }
}

/// Write sudoku to file
pub fn write_sudoku(sudoku: String, path: &Path) -> Result<(), GenericError> {
    let save_result = fs::write(path.display().to_string(), sudoku);
    match save_result {
        Err(_) => Err(GenericError {
            msg: "Saving the file failed".to_string(),
        }),
        _ => Ok(()),
    }
}

pub fn get_empty_sudoku() -> Result<Vec<Vec<Option<i32>>>, GenericError> {
    let empty = ".........
        .........
        .........
        .........
        .........
        .........
        .........
        .........
        ........."
        .to_string();

    clues_from_string(empty, ".")
}

/// Returns a 2D Vec from string to represent clues found in sudoku
pub fn clues_from_string(
    buf: String,
    empty_value: &str,
) -> Result<Vec<Vec<Option<i32>>>, GenericError> {
    let mut clues: Vec<Vec<Option<i32>>> = Vec::with_capacity(9);
    if buf.lines().collect::<Vec<&str>>().len() < 9 {
        return Err(GenericError {
            msg: "Invalid sudoku format!".to_owned(),
        });
    }
    for line in buf.lines() {
        let mut row_buf = Vec::with_capacity(9);
        for val in line.split("") {
            if val == empty_value {
                row_buf.push(None)
            }
            if let Ok(val) = val.parse() {
                row_buf.push(Some(val));
            }
        }
        if row_buf.len() != 9 {
            return Err(GenericError {
                msg: "Invalid sudoku format!".to_owned(),
            });
        }
        clues.push(row_buf);
    }

    Ok(clues)
}

/// Returns a properly formatted string representation of the gived sudoku grid
pub fn string_from_grid(grid: Vec<Vec<Option<i32>>>) -> String {
    let mut return_string = String::new();
    for row in grid.iter().take(9) {
        for col in row.iter().take(9) {
            match col {
                Some(v) => {
                    return_string.push_str(&v.to_string());
                }
                None => {
                    return_string.push('.');
                }
            }
        }
        return_string.push('\n');
    }
    return_string
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConstraintList, Trail};

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
    fn test_solve_sudoku_decimal() {
        let sudoku = get_sudoku("data/sample_sudoku.txt".to_string()).unwrap();
        let mut solver = cadical::Solver::with_config("plain").unwrap();
        let callback_wrapper = CadicalCallbackWrapper::new(ConstraintList::new(), Trail::new());
        solver.set_callbacks(Some(callback_wrapper.clone()));

        let encoding = EncodingType::Decimal {
            cell_at_least_one: true,
            cell_at_most_one: true,
            sudoku_has_all_values: true,
            sudoku_has_unique_values: true,
        };
        let solved = solve_sudoku(&sudoku, &mut solver, &encoding).unwrap();
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
    fn test_solve_sudoku_binary() {
        let sudoku = get_sudoku("data/sample_sudoku.txt".to_string()).unwrap();
        let mut solver = cadical::Solver::with_config("plain").unwrap();
        let callback_wrapper = CadicalCallbackWrapper::new(ConstraintList::new(), Trail::new());
        solver.set_callbacks(Some(callback_wrapper.clone()));

        let solved = solve_sudoku(&sudoku, &mut solver, &EncodingType::Binary).unwrap();
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

    #[test]
    fn test_string_to_clues() {
        let test_sudoku = "..3......\n\
                 1........\n\
                 .........\n\
                 .........\n\
                 ..8......\n\
                 .........\n\
                 ......2..\n\
                 .........\n\
                 .....6...\n";

        let clues = clues_from_string(test_sudoku.to_owned(), ".").unwrap();
        assert_eq!(clues[0][2], Some(3));
        assert_eq!(clues[1][0], Some(1));
        assert_eq!(clues[4][2], Some(8));
    }

    #[test]
    fn test_string_from_grid() {
        let mut test_vec = vec![vec![<Option<i32>>::None; 9]; 9];
        test_vec[0][0] = Some(1);
        test_vec[1][1] = Some(2);
        test_vec[2][2] = Some(3);
        test_vec[3][3] = Some(4);
        let output_string = string_from_grid(test_vec);

        let test_string = "1........\n\
                 .2.......\n\
                 ..3......\n\
                 ...4.....\n\
                 .........\n\
                 .........\n\
                 .........\n\
                 .........\n\
                 .........\n";

        assert_eq!(output_string, test_string)
    }

    #[test]
    fn test_string_from_grid_second() {
        let mut test_vec = vec![vec![<Option<i32>>::None; 9]; 9];
        test_vec[0][5] = Some(1);
        test_vec[2][3] = Some(2);
        test_vec[4][7] = Some(3);
        test_vec[7][5] = Some(4);
        let output_string = string_from_grid(test_vec);

        let test_string = ".....1...\n\
                 .........\n\
                 ...2.....\n\
                 .........\n\
                 .......3.\n\
                 .........\n\
                 .........\n\
                 .....4...\n\
                 .........\n";

        assert_eq!(output_string, test_string)
    }

    #[test]
    fn test_invalid_string_to_sudoku() {
        // Not enough cols
        let sudoku = "...".to_string();
        let result = clues_from_string(sudoku, ".");
        assert!(result.is_err());

        // Second type of error (not enough rows)
        let sudoku2 = ".........\n".to_string();
        let result2 = clues_from_string(sudoku2, ".");
        println!("{result2:?}");
        assert!(result2.is_err());

        // Third type of error (not numbers)
        let sudoku3 = "tlnaoeut.\n\
                 .........\n\
                 ...2.....\n\
                 tetete...\n\
                 .......3.\n\
                 .........\n\
                 .........\n\
                 .....4...\n\
                 .........\n";
        let result3 = clues_from_string(sudoku3.to_string(), ".");
        assert!(result3.is_err());
    }

    #[test]
    fn test_solve_sudoku_fails() {
        let sudoku_string = ".........\n\
                 .........\n\
                 ...22....\n\
                 .........\n\
                 .......3.\n\
                 .........\n\
                 .........\n\
                 .....4...\n\
                 .........\n"
            .to_string();
        let sudoku = clues_from_string(sudoku_string, ".").unwrap();
        let mut solver = cadical::Solver::with_config("plain").unwrap();
        let callback_wrapper = CadicalCallbackWrapper::new(ConstraintList::new(), Trail::new());
        solver.set_callbacks(Some(callback_wrapper.clone()));

        let encoding = EncodingType::Decimal {
            cell_at_least_one: true,
            cell_at_most_one: true,
            sudoku_has_all_values: true,
            sudoku_has_unique_values: true,
        };
        let solved = solve_sudoku(&sudoku, &mut solver, &encoding);
        assert!(solved.is_err());
    }

    #[test]
    fn test_get_empty_sudoku() {
        let sudoku = get_empty_sudoku();
        assert_eq!(sudoku.is_ok(), true);
        if let Ok(sudoku) = sudoku {
            assert_eq!(sudoku[0][0], None);
            assert_eq!(sudoku[8][8], None);
        }
    }
}
