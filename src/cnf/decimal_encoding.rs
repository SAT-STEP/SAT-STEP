//! Functions for decimal based CNF encoding

use crate::cadical_wrapper::CadicalCallbackWrapper;
use cadical::Solver;

/// Returns a Vec of CNF clauses (stored as Vec<i32>) which fully
/// encodes the rules of sudoku, and the clues given as an argument.
/// Check the link below for more details on the encoding:
/// https://docs.google.com/document/u/0/d/1VMQQ-wGp8Ji-V3uGQBcjKqTwO-OnSFk2WjuArnd57Fk/mobilebasic
pub fn sudoku_to_cnf(
    clues: &[Vec<Option<i32>>],
    cell_at_least_one: bool,
    cell_at_most_one: bool,
    sudoku_has_all_values: bool,
    sudoku_has_unique_values: bool,
) -> Vec<Vec<i32>> {
    // Each vec inside represents one cnf "statement"
    let mut clauses: Vec<Vec<i32>> = Vec::new();

    // Each cell has at least one value
    if cell_at_least_one {
        for row in 1..=9 {
            for col in 1..=9 {
                let mut cell_cnf: Vec<i32> = Vec::with_capacity(9);
                for val in 1..=9 {
                    cell_cnf.push(cnf_identifier(row, col, val));
                }
                clauses.push(cell_cnf);
            }
        }
    }

    // Each cell has at most one value
    if cell_at_most_one {
        for row in 1..=9 {
            for col in 1..=9 {
                for val1 in 1..=8 {
                    for val2 in (val1 + 1)..=9 {
                        let cell_cnf = vec![
                            -cnf_identifier(row, col, val1),
                            -cnf_identifier(row, col, val2),
                        ];
                        clauses.push(cell_cnf);
                    }
                }
            }
        }
    }

    if sudoku_has_all_values {
        // Each row has all the numbers
        for val in 1..=9 {
            for row in 1..=9 {
                let mut row_cnf: Vec<i32> = Vec::with_capacity(9);
                for col in 1..=9 {
                    row_cnf.push(cnf_identifier(row, col, val));
                }
                clauses.push(row_cnf);
            }
        }

        // Each column has all the numbers
        for val in 1..=9 {
            for col in 1..=9 {
                let mut col_cnf: Vec<i32> = Vec::with_capacity(9);
                for row in 1..=9 {
                    col_cnf.push(cnf_identifier(row, col, val));
                }
                clauses.push(col_cnf);
            }
        }

        // Each sub-grid has all the numbers
        for subgrid_row in 0..=2 {
            for subgrid_col in 0..=2 {
                for val in 1..=9 {
                    let mut subgrid_cnf: Vec<i32> = Vec::with_capacity(9);
                    for row in 1..=3 {
                        for col in 1..=3 {
                            subgrid_cnf.push(cnf_identifier(
                                row + 3 * subgrid_row,
                                col + 3 * subgrid_col,
                                val,
                            ));
                        }
                    }
                    clauses.push(subgrid_cnf);
                }
            }
        }
    }

    if sudoku_has_unique_values {
        // Each row has unique numbers (no duplicates)
        for row in 1..=9 {
            for col1 in 1..=8 {
                for col2 in (col1 + 1)..=9 {
                    for val in 1..=9 {
                        clauses.push(vec![
                            -cnf_identifier(row, col1, val),
                            -cnf_identifier(row, col2, val),
                        ]);
                    }
                }
            }
        }

        // Each column has unique numbers (no duplicates)
        for col in 1..=9 {
            for row1 in 1..=8 {
                for row2 in (row1 + 1)..=9 {
                    for val in 1..=9 {
                        clauses.push(vec![
                            -cnf_identifier(row1, col, val),
                            -cnf_identifier(row2, col, val),
                        ]);
                    }
                }
            }
        }

        // Each sub-grid has unique numbers (no duplicates)
        for subgrid_row in 0..=2 {
            for subgrid_col in 0..=2 {
                for index1 in 0..9 {
                    for index2 in (index1 + 1)..9 {
                        let row = 1 + subgrid_row * 3 + index1 % 3;
                        let col = 1 + subgrid_col * 3 + index1 / 3;
                        let row2 = 1 + subgrid_row * 3 + index2 % 3;
                        let col2 = 1 + subgrid_col * 3 + index2 / 3;
                        for val in 1..=9 {
                            clauses.push(vec![
                                -cnf_identifier(row, col, val),
                                -cnf_identifier(row2, col2, val),
                            ]);
                        }
                    }
                }
            }
        }
    }

    // Respect all the clues
    // Adds a unit clause (single variable clause) for each clue
    for (row, line) in clues.iter().enumerate() {
        for (col, val) in line.iter().enumerate() {
            if let Some(val) = val {
                clauses.push(vec![cnf_identifier(row as i32 + 1, col as i32 + 1, *val)]);
            }
        }
    }

    clauses
}

#[inline(always)]
/// Gives every variable (row, column and value combination) a unique identifier > 0
pub fn cnf_identifier(row: i32, col: i32, val: i32) -> i32 {
    (row - 1) * 9 * 9 + (col - 1) * 9 + val
}

#[inline(always)]
/// Reverse CNF-identifier creation
/// Return tuple of (row, col, val) from identifier
/// Val will be negative for negative ids, positive otherwise
pub fn identifier_to_tuple(mut identifier: i32) -> (i32, i32, i32) {
    let negation_multiplier = if identifier > 0 { 1 } else { -1 };
    identifier = identifier.abs() - 1;
    (
        identifier / (9 * 9) + 1,
        (identifier % 81) / 9 + 1,
        negation_multiplier * (identifier % 9 + 1),
    )
}

pub fn get_cell_value(solver: &Solver<CadicalCallbackWrapper>, row: i32, col: i32) -> i32 {
    let mut value = -1;
    for val in 1..=9 {
        if solver.value(cnf_identifier(row, col, val)).unwrap_or(false) {
            value = val;
            break;
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        clues_from_string, cnf::EncodingType, sudoku::solve_sudoku, CadicalCallbackWrapper,
        ConstraintList, Trail,
    };

    #[test]
    fn test_cnf_converter_respects_clues() {
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
        let clauses = sudoku_to_cnf(&clues, true, true, true, false);

        assert_eq!(clauses[clauses.len() - 1][0], cnf_identifier(9, 6, 6));
    }

    #[test]
    fn test_to_id_and_back() {
        assert_eq!((1, 1, 1), identifier_to_tuple(cnf_identifier(1, 1, 1)));
        assert_eq!((1, 2, 3), identifier_to_tuple(cnf_identifier(1, 2, 3)));
        assert_eq!((9, 9, 9), identifier_to_tuple(cnf_identifier(9, 9, 9)));
        assert_eq!(
            (6, 2, -8),
            identifier_to_tuple(-1 * cnf_identifier(6, 2, 8))
        );
    }

    #[test]
    fn test_get_cell_value() {
        let test_sudoku = "..3......\n\
                 1........\n\
                 .........\n\
                 .........\n\
                 ..8......\n\
                 .........\n\
                 ......2..\n\
                 .........\n\
                 .....6...\n"
            .to_string();

        let sudoku = clues_from_string(test_sudoku, ".").unwrap();

        let mut solver = cadical::Solver::with_config("plain").unwrap();
        let callback_wrapper = CadicalCallbackWrapper::new(ConstraintList::new(), Trail::new());
        solver.set_callbacks(Some(callback_wrapper.clone()));

        let encoding = EncodingType::Decimal {
            cell_at_least_one: true,
            cell_at_most_one: true,
            sudoku_has_all_values: true,
            sudoku_has_unique_values: true,
        };
        solve_sudoku(&sudoku, &mut solver, &encoding).unwrap();

        let cell_value2 = get_cell_value(&solver, 1, 3);
        assert_eq!(cell_value2, 3)
    }
}
