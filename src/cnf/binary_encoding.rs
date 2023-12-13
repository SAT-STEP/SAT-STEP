//! Functions for binary based CNF encoding

use crate::cadical_wrapper::CadicalCallbackWrapper;
use cadical::Solver;

/// Returns a Vec of CNF clauses (stored as `Vec<i32>`) which fully
/// encodes the rules of sudoku, and the clues given as an argument.
/// Check the link below for more details on the encoding:
/// <https://docs.google.com/document/u/0/d/1VMQQ-wGp8Ji-V3uGQBcjKqTwO-OnSFk2WjuArnd57Fk/mobilebasic>
pub fn sudoku_to_cnf(clues: &[Vec<Option<i32>>]) -> Vec<Vec<i32>> {
    // Each vec inside represents one cnf "statement"
    let mut clauses: Vec<Vec<i32>> = Vec::new();

    // Every number in each row is different
    // For each pair of cells in a row, at least one bit is NOT equal
    for row in 1..=9 {
        for col in 1..=9 {
            for col2 in (col + 1)..=9 {
                clauses.append(&mut eq_variable_init(row, col, row, col2));
                clauses.push(vec![
                    -eq_cnf_identifier(row, col, row, col2, 0),
                    -eq_cnf_identifier(row, col, row, col2, 1),
                    -eq_cnf_identifier(row, col, row, col2, 2),
                    -eq_cnf_identifier(row, col, row, col2, 3),
                ]);
            }
        }
    }

    // Every number in each col is different
    // For each pair of cells in a column, at least one bit is NOT equal
    for col in 1..=9 {
        for row in 1..=9 {
            for row2 in (row + 1)..=9 {
                clauses.append(&mut eq_variable_init(row, col, row2, col));
                clauses.push(vec![
                    -eq_cnf_identifier(row, col, row2, col, 0),
                    -eq_cnf_identifier(row, col, row2, col, 1),
                    -eq_cnf_identifier(row, col, row2, col, 2),
                    -eq_cnf_identifier(row, col, row2, col, 3),
                ]);
            }
        }
    }

    // Every number in each 3x3 cell sub-grid is different
    // For each pair of cells in a sub-grid, at least one bit is NOT equal
    for subgrid_row in 0..=2 {
        for subgrid_col in 0..=2 {
            for index1 in 0..9 {
                for index2 in (index1 + 1)..9 {
                    let row = 1 + subgrid_row * 3 + index1 % 3;
                    let col = 1 + subgrid_col * 3 + index1 / 3;
                    let row2 = 1 + subgrid_row * 3 + index2 % 3;
                    let col2 = 1 + subgrid_col * 3 + index2 / 3;
                    clauses.append(&mut eq_variable_init(row, col, row2, col2));
                    clauses.push(vec![
                        -eq_cnf_identifier(row, col, row2, col2, 0),
                        -eq_cnf_identifier(row, col, row2, col2, 1),
                        -eq_cnf_identifier(row, col, row2, col2, 2),
                        -eq_cnf_identifier(row, col, row2, col2, 3),
                    ]);
                }
            }
        }
    }

    // No numbers > 9 (> 8 in binary, since we are using the binary numbers 0 to 8)
    // Each cell must differ from every forbidden value by at least one bit
    for row in 1..=9 {
        for col in 1..=9 {
            for forbidden in 9..16 {
                let mut cell_clause = Vec::with_capacity(4);
                let mut mask = 1;
                for index in 0..4 {
                    // Here we invert the bits, since we do NOT want to allow the forbidden numbers
                    if (forbidden & mask) != 0 {
                        cell_clause.push(-cnf_identifier(row, col, index));
                    } else {
                        cell_clause.push(cnf_identifier(row, col, index));
                    }
                    mask *= 2;
                }
                clauses.push(cell_clause);
            }
        }
    }

    // Respect all the clues
    // Adds a unit clause (single variable clause) for each bit of each clue
    for (row, line) in clues.iter().enumerate() {
        for (col, val) in line.iter().enumerate() {
            if let Some(mut val) = val {
                val -= 1;
                let mut mask = 1;
                for index in 0..4 {
                    if (val & mask) != 0 {
                        clauses.push(vec![cnf_identifier(row as i32 + 1, col as i32 + 1, index)]);
                    } else {
                        clauses.push(vec![-cnf_identifier(row as i32 + 1, col as i32 + 1, index)]);
                    }
                    mask *= 2;
                }
            }
        }
    }

    clauses
}

/// Initialize EQ variable that indicate 2 cells have same bits in a specific position
/// There clauses are needed to ensure that the EQ var corresponds exactly to two bits being equal
/// since they are just variables from the perspective of the SAT-solver
fn eq_variable_init(row: i32, col: i32, row2: i32, col2: i32) -> Vec<Vec<i32>> {
    let mut clauses: Vec<Vec<i32>> = Vec::new();

    for bit in 0..4 {
        clauses.push(vec![
            -eq_cnf_identifier(row, col, row2, col2, bit),
            cnf_identifier(row, col, bit),
            -cnf_identifier(row2, col2, bit),
        ]);

        clauses.push(vec![
            -eq_cnf_identifier(row, col, row2, col2, bit),
            -cnf_identifier(row, col, bit),
            cnf_identifier(row2, col2, bit),
        ]);

        clauses.push(vec![
            eq_cnf_identifier(row, col, row2, col2, bit),
            -cnf_identifier(row, col, bit),
            -cnf_identifier(row2, col2, bit),
        ]);

        clauses.push(vec![
            eq_cnf_identifier(row, col, row2, col2, bit),
            cnf_identifier(row, col, bit),
            cnf_identifier(row2, col2, bit),
        ]);
    }

    clauses
}

/// Gets all bit values of a cell from the solver, and converts thet to a decimal value,
/// which is returned.
pub fn get_cell_value(solver: &Solver<CadicalCallbackWrapper>, row: i32, col: i32) -> i32 {
    let mut value: i32 = 1;
    for bit in 0..4 {
        // Add 2^(bit) to the value for each 1-bit of the cell
        if solver.value(cnf_identifier(row, col, bit)).unwrap() {
            value += 2_i32.pow(bit as u32);
        }
    }
    value
}

#[inline(always)]
/// Gives every bit variable (row, column and bit combination) a unique identifier 1 to 324
pub fn cnf_identifier(row: i32, col: i32, bit: i32) -> i32 {
    (row - 1) * 4 * 9 + (col - 1) * 4 + bit + 1
}

#[inline(always)]
/// Gives every equality variable (cell pair and bit combination) a unique identifier > 324
pub fn eq_cnf_identifier(row: i32, col: i32, row2: i32, col2: i32, bit: i32) -> i32 {
    9 * 9 * 4
        + (row - 1) * 4 * 9 * 9 * 9
        + (col - 1) * 4 * 9 * 9
        + (row2 - 1) * 4 * 9
        + (col2 - 1) * 4
        + bit
        + 1
}

#[inline(always)]
/// Reverse CNF-identifier creation
/// Return tuple of (row, col, bit_index, bit_value) from identifier
/// bit_value will be false for negative ids, positive otherwise
pub fn identifier_to_tuple(mut identifier: i32) -> (i32, i32, i32, bool) {
    let bit_value = identifier > 0;
    identifier = identifier.abs() - 1;
    (
        identifier / (9 * 4) + 1,
        (identifier % (9 * 4)) / 4 + 1,
        (identifier % 4),
        bit_value,
    )
}

/// Reverse CNF-identifier creation for equality variables
/// Return tuple of (row, col, row2, col2, bit_index, equal) from identifier
/// equal will be false, if the bits in the two cells are different
pub fn eq_identifier_to_tuple(mut identifier: i32) -> (i32, i32, i32, i32, i32, bool) {
    let equal = identifier > 0;
    identifier = identifier.abs() - 1 - 9 * 9 * 4;
    (
        identifier / (9 * 9 * 9 * 4) + 1,
        (identifier % (9 * 9 * 9 * 4)) / (4 * 9 * 9) + 1,
        (identifier % (9 * 9 * 4)) / (4 * 9) + 1,
        (identifier % (9 * 4)) / 4 + 1,
        (identifier % 4),
        equal,
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{app_state::EncodingType, sudoku::clues_from_string, sudoku::solve_sudoku};
    use crate::{ConstraintList, Trail};

    use super::*;

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
        let clauses = sudoku_to_cnf(&clues);

        let result = vec![
            clauses[clauses.len() - 4][0],
            clauses[clauses.len() - 3][0],
            clauses[clauses.len() - 2][0],
            clauses[clauses.len() - 1][0],
        ];

        let expected = vec![
            cnf_identifier(9, 6, 0), // we have 6 as the clue, inside the converter this is 5,
            -cnf_identifier(9, 6, 1), // so in binary 0101
            cnf_identifier(9, 6, 2),
            -cnf_identifier(9, 6, 3),
        ];

        assert_eq!(result, expected);
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

        solve_sudoku(&sudoku, &mut solver, &EncodingType::Binary).unwrap();

        let cell_value = get_cell_value(&solver, 1, 3);

        assert_eq!(cell_value, 3)
    }

    #[test]
    fn no_overlapping_identifiers() {
        let mut identifiers: Vec<i32> = Vec::new();
        let mut identifiers_set: HashSet<i32> = HashSet::new();

        for row in 1..=9 {
            for col in 1..=9 {
                for row2 in 1..=9 {
                    for col2 in 1..=9 {
                        for bit in 0..4 {
                            identifiers_set.insert(eq_cnf_identifier(row, col, row2, col2, bit));
                            identifiers.push(eq_cnf_identifier(row, col, row2, col2, bit));
                        }
                    }
                }
                for bit in 0..4 {
                    identifiers_set.insert(cnf_identifier(row, col, bit));
                    identifiers.push(cnf_identifier(row, col, bit));
                }
            }
        }

        assert_eq!(identifiers.len(), identifiers_set.len());
    }

    #[test]
    fn test_to_id_and_back() {
        assert_eq!(
            (1, 1, 0, true),
            identifier_to_tuple(cnf_identifier(1, 1, 0))
        );
        assert_eq!(
            (1, 2, 3, true),
            identifier_to_tuple(cnf_identifier(1, 2, 3))
        );
        assert_eq!(
            (9, 9, 3, true),
            identifier_to_tuple(cnf_identifier(9, 9, 3))
        );
        assert_eq!(
            (6, 2, 0, false),
            identifier_to_tuple(-1 * cnf_identifier(6, 2, 0))
        );
    }

    #[test]
    fn test_to_eq_id_and_back() {
        assert_eq!(
            (1, 1, 1, 1, 0, true),
            eq_identifier_to_tuple(eq_cnf_identifier(1, 1, 1, 1, 0))
        );
        assert_eq!(
            (1, 2, 3, 4, 0, true),
            eq_identifier_to_tuple(eq_cnf_identifier(1, 2, 3, 4, 0))
        );
        assert_eq!(
            (9, 9, 9, 9, 3, true),
            eq_identifier_to_tuple(eq_cnf_identifier(9, 9, 9, 9, 3))
        );
        assert_eq!(
            (6, 2, 8, 2, 0, false),
            eq_identifier_to_tuple(-1 * eq_cnf_identifier(6, 2, 8, 2, 0))
        );
    }
}
