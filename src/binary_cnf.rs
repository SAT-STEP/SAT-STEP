pub fn sudoku_to_cnf(clues: &[Vec<Option<i32>>]) -> Vec<Vec<i32>> {
    // each vec inside represents one cnf "statement"
    let mut clauses: Vec<Vec<i32>> = Vec::new();

    // define how equivalence constraints work
    for row in 1..=9 {
        for col in 1..=9 {
            for row2 in 1..=9 {
                for col2 in 1..=9 {
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
                }
            }
        }
    }

    // every number in each row is different
    for row in 1..=9 {
        for col in 1..=9 {
            for col2 in (col + 1)..=9 {
                clauses.push(vec![
                    -eq_cnf_identifier(row, col, row, col2, 0),
                    -eq_cnf_identifier(row, col, row, col2, 1),
                    -eq_cnf_identifier(row, col, row, col2, 2),
                    -eq_cnf_identifier(row, col, row, col2, 3),
                ]);
            }
        }
    }

    // every number in each col is different
    for col in 1..=9 {
        for row in 1..=9 {
            for row2 in (row + 1)..=9 {
                clauses.push(vec![
                    -eq_cnf_identifier(row, col, row2, col, 0),
                    -eq_cnf_identifier(row, col, row2, col, 1),
                    -eq_cnf_identifier(row, col, row2, col, 2),
                    -eq_cnf_identifier(row, col, row2, col, 3),
                ]);
            }
        }
    }

    // each sub-grid has all the numbers
    for subgrid_row in 0..=2 {
        for subgrid_col in 0..=2 {
            for index1 in 0..9 {
                for index2 in (index1 + 1)..9 {
                    let row = 1 + subgrid_row * 3 + index1 % 3;
                    let col = 1 + subgrid_col * 3 + index1 / 3;
                    let row2 = 1 + subgrid_row * 3 + index2 % 3;
                    let col2 = 1 + subgrid_col * 3 + index2 / 3;
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
    // no numbers > 9
    for row in 1..=9 {
        for col in 1..=9 {
            for forbidden in 9..16 {
                let mut cell_clause = Vec::with_capacity(4);
                let mut mask = 1;
                for index in 0..4 {
                    // Here we invert the bits, since we do not want to allow the forbidden numbers
                    if (forbidden & mask) != 0 {
                        cell_clause.push(-cnf_identifier(row as i32, col as i32, index));
                    } else {
                        cell_clause.push(cnf_identifier(row as i32, col as i32, index));
                    }
                    mask *= 2;
                }
                clauses.push(cell_clause);
            }
        }
    }

    // respect all the clues
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

#[inline(always)]
pub fn cnf_identifier(row: i32, col: i32, bit: i32) -> i32 {
    // Creates cnf identifier from sudoku based on row, column and value
    // So every row, column and value combination has a unique identifier
    (row - 1) * 4 * 9 + (col - 1) * 4 + bit + 1
}

#[inline(always)]
pub fn eq_cnf_identifier(row: i32, col: i32, row2: i32, col2: i32, bit: i32) -> i32 {
    9 * 9 * 4
        + (row - 1) * 4 * 9
        + (col - 1) * 4
        + (row2 - 1) * 4 * 9 * 9 * 9
        + (col2 - 1) * 4 * 9 * 9
        + bit
        + 1
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
