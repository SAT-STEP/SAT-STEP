use crate::error::GenericError;

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
pub fn cnf_identifier(row: i32, col: i32, bit: i32) -> i32 {
    // Creates cnf identifier from sudoku based on row, column and value
    // So every row, column and value combination has a unique identifier
    (row - 1) * 4 * 9 + (col - 1) * 4 + bit
}

#[inline(always)]
pub fn identifier_to_tuple(mut identifier: i32) -> (i32, i32, i32) {
    // Reverse CNF-identifier creation
    // Return tuple of (row, col, val) from identifier
    // Val will be negative for negative ids, positive otherwise
    let negation_multiplier = if identifier > 0 { 1 } else { -1 };
    identifier = identifier.abs() - 1;
    (
        identifier / (9 * 4) + 1,
        (identifier % (9 * 4)) / 4 + 1,
        negation_multiplier * (identifier % 4 + 1),
    )
}

pub fn create_tuples_from_constraints(constraints: Vec<Vec<i32>>) -> Vec<Vec<(i32, i32, i32)>> {
    let mut tuples = Vec::new();
    for constraint in constraints.iter() {
        let mut temp = Vec::with_capacity(constraint.len());
        for value in constraint {
            temp.push(identifier_to_tuple(*value));
        }
        tuples.push(temp);
    }
    tuples
}
