pub mod binary_encoding;
pub mod decimal_encoding;

use std::collections::HashSet;

use crate::app_state::EncodingType;

/// Enum that (hopefully) fixes everything
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum CnfVariable {
    Decimal {
        row: i32,
        col: i32,
        value: i32,
    },
    Bit {
        row: i32,
        col: i32,
        bit_index: i32,
        value: bool,
    },
    Equality {
        row: i32,
        col: i32,
        row2: i32,
        col2: i32,
        bit_index: i32,
        equal: bool,
    },
}

impl CnfVariable {
    pub fn from_cnf(identifier: i32, encoding: &EncodingType) -> Self {
        match encoding {
            EncodingType::Binary => {
                if identifier.abs() > 9 * 9 * 4 {
                    let (row, col, row2, col2, bit_index, equal) =
                        binary_encoding::eq_identifier_to_tuple(identifier);
                    Self::Equality {
                        row,
                        col,
                        row2,
                        col2,
                        bit_index,
                        equal,
                    }
                } else {
                    let (row, col, bit_index, value) =
                        binary_encoding::identifier_to_tuple(identifier);
                    Self::Bit {
                        row,
                        col,
                        bit_index,
                        value,
                    }
                }
            }
            EncodingType::Decimal { .. } => {
                let (row, col, value) = decimal_encoding::identifier_to_tuple(identifier);
                Self::Decimal { row, col, value }
            }
        }
    }

    pub fn to_cnf(&self) -> i32 {
        match self {
            Self::Decimal { row, col, value } => {
                decimal_encoding::cnf_identifier(*row, *col, *value)
            }
            Self::Bit {
                row,
                col,
                bit_index,
                value,
            } => {
                if *value {
                    binary_encoding::cnf_identifier(*row, *col, *bit_index)
                } else {
                    -binary_encoding::cnf_identifier(*row, *col, *bit_index)
                }
            }
            Self::Equality {
                row,
                col,
                row2,
                col2,
                bit_index,
                equal,
            } => {
                if *equal {
                    binary_encoding::eq_cnf_identifier(*row, *col, *row2, *col2, *bit_index)
                } else {
                    -binary_encoding::eq_cnf_identifier(*row, *col, *row2, *col2, *bit_index)
                }
            }
        }
    }

    /// Returns HashSet of possible numbers, empty if self is equality variable, since
    /// the concept of possible values does not work for equality constraints.
    /// Used in drawing little numbers.
    pub fn get_possible_numbers(&self) -> HashSet<i32> {
        match self {
            Self::Equality { .. } => HashSet::new(),
            Self::Decimal { value, .. } => HashSet::from([*value]),
            Self::Bit {
                bit_index, value, ..
            } => {
                let mut possibilities: HashSet<i32> = HashSet::new();
                for i in 0..9 {
                    if (i & (1 << bit_index) > 0) == *value {
                        possibilities.insert(i + 1);
                    }
                }
                possibilities
            }
        }
    }
}

/// Check if the encoding rules are enough for Cadial to properly solve a sudoku
pub fn cnf_encoding_rules_ok(
    cell_at_least_one: bool,
    cell_at_most_one: bool,
    sudoku_has_all_values: bool,
    sudoku_has_unique_values: bool,
) -> bool {
    (cell_at_least_one && sudoku_has_unique_values) || (cell_at_most_one && sudoku_has_all_values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_cnf_and_back_bit() {
        let variable = CnfVariable::Bit {
            row: 1,
            col: 2,
            bit_index: 3,
            value: true,
        };
        let variable2 = CnfVariable::from_cnf(variable.to_cnf(), &EncodingType::Binary);
        assert_eq!(variable, variable2);

        let variable3 = CnfVariable::Bit {
            row: 9,
            col: 9,
            bit_index: 3,
            value: false,
        };
        let variable4 = CnfVariable::from_cnf(variable3.to_cnf(), &EncodingType::Binary);
        assert_eq!(variable3, variable4);
    }

    #[test]
    fn test_to_cnf_and_back_eq() {
        let variable = CnfVariable::Equality {
            row: 1,
            col: 2,
            row2: 3,
            col2: 4,
            bit_index: 0,
            equal: true,
        };
        let variable2 = CnfVariable::from_cnf(variable.to_cnf(), &EncodingType::Binary);
        assert_eq!(variable, variable2);

        let variable3 = CnfVariable::Equality {
            row: 8,
            col: 9,
            row2: 9,
            col2: 9,
            bit_index: 3,
            equal: false,
        };
        let variable4 = CnfVariable::from_cnf(variable3.to_cnf(), &EncodingType::Binary);
        assert_eq!(variable3, variable4);
    }

    #[test]
    fn test_to_cnf_and_back_decimal() {
        let variable = CnfVariable::Decimal {
            row: 1,
            col: 2,
            value: 3,
        };
        let encoding = EncodingType::Decimal {
            cell_at_least_one: true,
            cell_at_most_one: true,
            sudoku_has_all_values: true,
            sudoku_has_unique_values: true,
        };
        let variable2 = CnfVariable::from_cnf(variable.to_cnf(), &encoding);
        assert_eq!(variable, variable2);

        let variable3 = CnfVariable::Decimal {
            row: 9,
            col: 9,
            value: 9,
        };

        let variable4 = CnfVariable::from_cnf(-variable3.to_cnf(), &encoding);
        let variable5 = CnfVariable::Decimal {
            row: 9,
            col: 9,
            value: -9,
        };
        assert_eq!(variable4, variable5);
    }

    #[test]
    fn test_get_possible_numbers_decimal() {
        let variable = CnfVariable::Decimal {
            row: 1,
            col: 1,
            value: 3,
        };
        assert_eq!(variable.get_possible_numbers(), HashSet::from([3]));
    }

    #[test]
    fn test_get_possible_numbers_eq() {
        let variable = CnfVariable::Equality {
            row: 1,
            col: 1,
            row2: 1,
            col2: 2,
            bit_index: 0,
            equal: true,
        };
        assert_eq!(variable.get_possible_numbers(), HashSet::new());
    }

    #[test]
    fn test_get_possible_numbers_bit() {
        let variable = CnfVariable::Bit {
            row: 1,
            col: 1,
            bit_index: 2,
            value: true,
        };
        let variable2 = CnfVariable::Bit {
            row: 1,
            col: 1,
            bit_index: 1,
            value: false,
        };

        assert_eq!(variable.get_possible_numbers(), HashSet::from([5, 6, 7, 8]));
        assert_eq!(
            variable2.get_possible_numbers(),
            HashSet::from([1, 2, 5, 6, 9])
        );
    }
    #[test]
    fn test_encoding_rules_shouldbe_ok() {
        // Doesn't encompass all cases
        let cell_at_least_one = true;
        let cell_at_most_one = false;
        let sudoku_has_all_values = false;
        let sudoku_has_unique_values = true;

        assert!(cnf_encoding_rules_ok(
            cell_at_least_one,
            cell_at_most_one,
            sudoku_has_all_values,
            sudoku_has_unique_values
        ));

        let cell_at_least_one = false;
        let cell_at_most_one = true;
        let sudoku_has_all_values = true;
        let sudoku_has_unique_values = false;

        assert!(cnf_encoding_rules_ok(
            cell_at_least_one,
            cell_at_most_one,
            sudoku_has_all_values,
            sudoku_has_unique_values
        ));
    }

    #[test]
    fn test_encoding_rules_shouldbe_not_ok() {
        // Doesn't encompass all cases
        let cell_at_least_one = true;
        let cell_at_most_one = true;
        let sudoku_has_all_values = false;
        let sudoku_has_unique_values = false;

        assert!(!cnf_encoding_rules_ok(
            cell_at_least_one,
            cell_at_most_one,
            sudoku_has_all_values,
            sudoku_has_unique_values
        ));

        let cell_at_least_one = true;
        let cell_at_most_one = false;
        let sudoku_has_all_values = false;
        let sudoku_has_unique_values = false;

        assert!(!cnf_encoding_rules_ok(
            cell_at_least_one,
            cell_at_most_one,
            sudoku_has_all_values,
            sudoku_has_unique_values
        ));
    }
}
