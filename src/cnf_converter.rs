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

pub fn clues_from_string(
    buf: String,
    empty_value: &str,
) -> Result<Vec<Vec<Option<i32>>>, GenericError> {
    // Creates 2d Vec from string to represent clues found in sudoku
    let mut clues: Vec<Vec<Option<i32>>> = Vec::with_capacity(9);
    if buf.len() < 9 {
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

pub fn string_from_grid(grid: Vec<Vec<Option<i32>>>) -> String {
    let mut return_string = String::new();
    for row in 0..=8 {
        for col in 0..=8 {
            match grid[row][col] {
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
    return_string.pop();
    println!("{:?}", return_string);
    return return_string;
}
#[cfg(test)]
mod tests {
    use super::{clues_from_string, *};

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
    fn test_create_tuples_from_constraints() {
        let constraints = vec![vec![1, 2, 3], vec![10, 11, 12]];
        let tuples = create_tuples_from_constraints(constraints);

        assert_eq!((1, 1, 1), tuples[0][0]);
        assert_eq!((1, 1, 2), tuples[0][1]);
        assert_eq!((1, 1, 3), tuples[0][2]);

        assert_eq!((1, 2, 1), tuples[1][0]);
        assert_eq!((1, 2, 2), tuples[1][1]);
        assert_eq!((1, 2, 3), tuples[1][2]);
    }
}
