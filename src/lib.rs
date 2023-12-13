mod app_state;
mod cadical_wrapper;
mod cnf;
mod ctrl_obj;
mod error;
mod filtering;
pub mod gui;
mod sudoku;
mod warning;

#[cfg(test)]
mod tests;

use std::{cell::RefCell, num::ParseIntError, rc::Rc};

use crate::app_state::EncodingType;

use cadical::Solver;

use cadical_wrapper::CadicalCallbackWrapper;
use cnf::CnfVariable;
use error::GenericError;
use gui::sudoku_cell::SudokuCell;
use sudoku::string_from_grid;

/// ConstraintList is used to store the learned cnf_clauses inside a `Rc<RefCell<Vec<Vec<i32>>>>`
/// This allows for more flexibility with the ownership and borrowing system of Rust
/// See: <https://doc.rust-lang.org/book/ch15-05-interior-mutability.html#having-multiple-owners-of-mutable-data-by-combining-rct-and-refcellt>
#[derive(Clone)]
pub struct ConstraintList(Rc<RefCell<Vec<Vec<i32>>>>);

impl ConstraintList {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(Vec::new())))
    }

    /// TODO: rename to `from_constraints`
    pub fn _new(constraints: Rc<RefCell<Vec<Vec<i32>>>>) -> Self {
        Self(constraints)
    }

    pub fn clone_constraints(&self) -> Vec<Vec<i32>> {
        self.0.borrow().clone()
    }

    pub fn push(&mut self, constraint: Vec<i32>) {
        self.0.borrow_mut().push(constraint);
    }

    pub fn clear(&mut self) {
        self.0.borrow_mut().clear();
    }

    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }

    pub fn borrow(&self) -> std::cell::Ref<'_, Vec<Vec<i32>>> {
        self.0.borrow()
    }
}

impl Default for ConstraintList {
    fn default() -> Self {
        Self::new()
    }
}

/// Datastructure to hold conflict literals and trail data
#[derive(Clone)]
pub struct Trail {
    pub conflict_literals: Rc<RefCell<Vec<Vec<i32>>>>,
    pub trail: Rc<RefCell<Vec<Vec<i32>>>>,
    pub var_is_propagated: Rc<RefCell<Vec<Vec<bool>>>>,
}

impl Trail {
    pub fn new() -> Self {
        Self {
            conflict_literals: Rc::new(RefCell::new(Vec::new())),
            trail: Rc::new(RefCell::new(Vec::new())),
            var_is_propagated: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn as_cnf(&mut self, encoding: &EncodingType) -> Vec<Vec<CnfVariable>> {
        (*self.conflict_literals.borrow())
            .clone()
            .into_iter()
            .map(|conflict| {
                conflict
                    .into_iter()
                    .map(|literal| CnfVariable::from_cnf(literal, encoding))
                    .collect()
            })
            .collect()
    }

    pub fn push(
        &mut self,
        conflict_literals: Vec<i32>,
        trail: Vec<i32>,
        var_is_propagated: Vec<bool>,
    ) {
        self.conflict_literals.borrow_mut().push(conflict_literals);
        self.trail.borrow_mut().push(trail);
        self.var_is_propagated.borrow_mut().push(var_is_propagated);
    }

    pub fn clear(&mut self) {
        self.conflict_literals.borrow_mut().clear();
        self.trail.borrow_mut().clear();
        self.var_is_propagated.borrow_mut().clear();
    }

    pub fn trail_at_index(&self, index: usize) -> Vec<i32> {
        self.trail.borrow()[index].clone()
    }

    pub fn literals_at_index(&self, index: usize) -> Vec<i32> {
        self.conflict_literals.borrow()[index].clone()
    }

    pub fn var_is_propagated_at_index(&self, index: usize) -> Vec<bool> {
        self.var_is_propagated.borrow()[index].clone()
    }

    pub fn len(&self) -> usize {
        self.trail.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.trail.borrow().is_empty()
    }
}

impl Default for Trail {
    fn default() -> Self {
        Self::new()
    }
}

/// Parses numeric inputs given by the user. Inputs are for the max_length filter and page length.
pub fn parse_numeric_input(input: &str) -> Option<i32> {
    let parse_result: Result<i32, ParseIntError> = input.parse();
    match parse_result {
        Ok(parsed) => {
            if parsed < 1 {
                return None;
            }
            Some(parsed)
        }
        Err(_err) => None,
    }
}

/// Get reference to SudokuCell.
pub fn get_cell(sudoku: &mut [Vec<SudokuCell>], row: i32, column: i32) -> &mut SudokuCell {
    &mut sudoku[row as usize - 1][column as usize - 1]
}