use crate::app_state::EncodingType;
use crate::binary_cnf;
use crate::cnf_converter;

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
                        binary_cnf::eq_identifier_to_tuple(identifier);
                    Self::Equality {
                        row,
                        col,
                        row2,
                        col2,
                        bit_index,
                        equal,
                    }
                } else {
                    let (row, col, bit_index, value) = binary_cnf::identifier_to_tuple(identifier);
                    Self::Bit {
                        row,
                        col,
                        bit_index,
                        value,
                    }
                }
            }
            EncodingType::Decimal => {
                let (row, col, value) = cnf_converter::identifier_to_tuple(identifier);
                Self::Decimal { row, col, value }
            }
        }
    }

    pub fn to_cnf(&self) -> i32 {
        match self {
            Self::Decimal { row, col, value } => cnf_converter::cnf_identifier(*row, *col, *value),
            Self::Bit {
                row,
                col,
                bit_index,
                value,
            } => {
                if *value {
                    binary_cnf::cnf_identifier(*row, *col, *bit_index)
                } else {
                    -binary_cnf::cnf_identifier(*row, *col, *bit_index)
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
                    binary_cnf::eq_cnf_identifier(*row, *col, *row2, *col2, *bit_index)
                } else {
                    -binary_cnf::eq_cnf_identifier(*row, *col, *row2, *col2, *bit_index)
                }
            }
        }
    }
}
