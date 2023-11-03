use crate::{binary_cnf::{BitVar, EqVar},cnf_converter::DecimalVar};

#[derive(Clone, Debug, PartialEq)]
pub enum CnfVariableType {
    Decimal(DecimalVar),
    Bit(BitVar),
    Equality(EqVar),
}
