use crate::{cadical_wrapper::CadicalCallbackWrapper, cnf_var::CnfVariable};
use cadical::Solver;
use egui::{
    text::{LayoutJob, TextFormat},
    Color32, FontId,
};

#[derive(Clone)]
pub struct BitVar {
    pub row: i32,
    pub col: i32,
    pub bit_index: i32,
    pub bit_value: bool,
}

#[derive(Clone)]
pub struct EqVar {
    pub row: i32,
    pub col: i32,
    pub row2: i32,
    pub col2: i32,
    pub bit_index: i32,
    pub equal: bool,
}

impl BitVar {
    fn new(identifier: i32) -> BitVar {
        let (row, col, bit_index, bit_value) = identifier_to_tuple(identifier);
        BitVar {
            row,
            col,
            bit_index,
            bit_value,
        }
    }
}

impl CnfVariable for BitVar {


    fn human_readable(
        &self,
        text_job: &mut LayoutJob,
        large_font: FontId,
        small_font: FontId,
        text_color: Color32,
    ) {
        let (lead_char, color) = if self.bit_value {
            ("B", text_color)
        } else {
            ("~B", Color32::RED)
        };

        text_job.append(
            &format!("{}{}", lead_char, self.bit_index),
            0.0,
            TextFormat {
                font_id: large_font.clone(),
                color,
                ..Default::default()
            },
        );
        text_job.append(
            &format!("({},{})", self.row, self.col),
            0.0,
            TextFormat {
                font_id: small_font.clone(),
                color,
                ..Default::default()
            },
        );
    }

    fn to_cnf(&self) -> i32 {
        if self.bit_value {
            cnf_identifier(self.row, self.col, self.bit_index)
        } else {
            -cnf_identifier(self.row, self.col, self.bit_index)
        }
    }
}

impl EqVar {
    fn new(identifier: i32) -> EqVar {
        let (row, col, row2, col2, bit_index, equal) = eq_identifier_to_tuple(identifier);
        EqVar {
            row,
            col,
            row2,
            col2,
            bit_index,
            equal,
        }
    }
}

impl CnfVariable for EqVar {


    fn human_readable(
        &self,
        text_job: &mut LayoutJob,
        large_font: FontId,
        small_font: FontId,
        text_color: Color32,
    ) {
        let (lead_char, color) = if self.equal {
            ("EQ", text_color)
        } else {
            ("~EQ", Color32::RED)
        };

        text_job.append(
            &format!("{}{}", lead_char, self.bit_index),
            0.0,
            TextFormat {
                font_id: large_font.clone(),
                color,
                ..Default::default()
            },
        );
        text_job.append(
            &format!("({},{});({},{})", self.row, self.col, self.row2, self.col2),
            0.0,
            TextFormat {
                font_id: small_font.clone(),
                color,
                ..Default::default()
            },
        );
    }

    fn to_cnf(&self) -> i32 {
        if self.equal {
            eq_cnf_identifier(self.row, self.col, self.row2, self.col2, self.bit_index)
        } else {
            -eq_cnf_identifier(self.row, self.col, self.row2, self.col2, self.bit_index)
        }
    }
}

#[allow(dead_code)]
pub fn sudoku_to_cnf(clues: &[Vec<Option<i32>>]) -> Vec<Vec<i32>> {
    // each vec inside represents one cnf "statement"
    let mut clauses: Vec<Vec<i32>> = Vec::new();

    // every number in each row is different
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

    // every number in each col is different
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

    // each sub-grid has all the numbers
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
    // no numbers > 9
    for row in 1..=9 {
        for col in 1..=9 {
            for forbidden in 9..16 {
                let mut cell_clause = Vec::with_capacity(4);
                let mut mask = 1;
                for index in 0..4 {
                    // Here we invert the bits, since we do not want to allow the forbidden numbers
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

/// Initialize variables that indicate 2 cells have same bits in some position
#[allow(dead_code)]
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

#[allow(dead_code)] // allowed since binary encoding isn't used yet
pub fn get_cell_value(solver: &Solver<CadicalCallbackWrapper>, row: i32, col: i32) -> i32 {
    let mut value: i32 = 1;
    for bit in 0..4 {
        if solver.value(cnf_identifier(row, col, bit)).unwrap() {
            value += 2_i32.pow(bit as u32);
        }
    }
    value
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
        + (row - 1) * 4 * 9 * 9 * 9
        + (col - 1) * 4 * 9 * 9
        + (row2 - 1) * 4 * 9
        + (col2 - 1) * 4
        + bit
        + 1
}

/// These do not work for the new encoding YET, which is why they are not used YET
#[allow(dead_code)]
#[inline(always)]
fn identifier_to_tuple(mut identifier: i32) -> (i32, i32, i32, bool) {
    // Reverse CNF-identifier creation
    // Return tuple of (row, col, bit_index, bit_value) from identifier
    // bit_value will be false for negative ids, positive otherwise
    let bit_value = identifier > 0;
    identifier = identifier.abs() - 1;
    (
        identifier / (9 * 4) + 1,
        (identifier % (9 * 4)) / 4 + 1,
        (identifier % 4),
        bit_value,
    )
}

#[allow(dead_code)]
fn eq_identifier_to_tuple(mut identifier: i32) -> (i32, i32, i32, i32, i32, bool) {
    // Reverse CNF-identifier creation for equality constraints
    // Return tuple of (row, col, row2, col2, bit_index, equal) from identifier
    // equal will be false, if the bits in the two cells are different
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

#[allow(dead_code)]
pub fn create_tuples_from_constraints(
    constraints: Vec<Vec<i32>>,
) -> Vec<Vec<(i32, i32, i32, bool)>> {
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

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
