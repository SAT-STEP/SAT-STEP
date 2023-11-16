use crate::cadical_wrapper::CadicalCallbackWrapper;
use cadical::Solver;

pub fn sudoku_to_cnf(clues: &[Vec<Option<i32>>]) -> Vec<Vec<i32>> {
    // each vec inside represents one cnf "statement"
    let mut clauses: Vec<Vec<i32>> = Vec::new();

    // each cell has at least one value
    for row in 1..=9 {
        for col in 1..=9 {
            let mut cell_cnf: Vec<i32> = Vec::with_capacity(9);
            for val in 1..=9 {
                cell_cnf.push(cnf_identifier(row, col, val));
            }
            clauses.push(cell_cnf);
        }
    }

    // each cell has at most one value
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

    // each row has all the numbers
    for val in 1..=9 {
        for row in 1..=9 {
            let mut row_cnf: Vec<i32> = Vec::with_capacity(9);
            for col in 1..=9 {
                row_cnf.push(cnf_identifier(row, col, val));
            }
            clauses.push(row_cnf);
        }
    }

    // each column has all the numbers
    for val in 1..=9 {
        for col in 1..=9 {
            let mut col_cnf: Vec<i32> = Vec::with_capacity(9);
            for row in 1..=9 {
                col_cnf.push(cnf_identifier(row, col, val));
            }
            clauses.push(col_cnf);
        }
    }

    // each sub-grid has all the numbers
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

    // respect all the clues
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
pub fn cnf_identifier(row: i32, col: i32, val: i32) -> i32 {
    // Creates cnf identifier from sudoku based on row, column and value
    // So every row, column and value combination has a unique identifier
    (row - 1) * 9 * 9 + (col - 1) * 9 + val
}

#[inline(always)]
pub fn identifier_to_tuple(mut identifier: i32) -> (i32, i32, i32) {
    // Reverse CNF-identifier creation
    // Return tuple of (row, col, val) from identifier
    // Val will be negative for negative ids, positive otherwise
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
        if solver.value(cnf_identifier(row, col, val)).unwrap() {
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
        let clauses = sudoku_to_cnf(&clues);

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

        solve_sudoku(&sudoku, &mut solver, &EncodingType::Decimal).unwrap();

        let cell_value2 = get_cell_value(&solver, 1, 3);
        assert_eq!(cell_value2, 3)
    }
}
