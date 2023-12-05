//! Interfacing with cadical

use cadical::Callbacks;

use crate::{ConstraintList, Trail};

#[derive(Clone)]
pub struct CadicalCallbackWrapper {
    pub learned_clauses: ConstraintList,
    pub trail: Trail,
}

impl CadicalCallbackWrapper {
    pub fn new(learned_clauses: ConstraintList, trail: Trail) -> Self {
        Self {
            learned_clauses,
            trail,
        }
    }
}

impl Callbacks for CadicalCallbackWrapper {
    // when `solve` is called
    fn started(&mut self) {
        println!("Cadical started SAT solving!");
    }

    // called by the solver to check if it should terminate
    fn terminate(&mut self) -> bool {
        false
    }

    // Returns the maximum length of clauses to be passed to `learn`. This
    // methods will be called only once when `set_callbacks` is called.
    fn max_length(&self) -> i32 {
        i32::max_value()
    }

    // called by the solver when a new derived clause is learnt
    fn learn(&mut self, clause: &[i32]) {
        // println!("Learned clause: {:?}", clause.to_vec());
        let tmp_vector: Vec<i32> = clause.to_vec();
        if !clause.is_empty() {
            self.learned_clauses.push(tmp_vector);
        }
    }

    // called when a new derived clause is learnt
    fn learn_trail(&mut self, conflict_literals: &[i32], _is_propagated: &[i32], trail: &[i32]) {
        self.trail.push(conflict_literals.to_vec(), trail.to_vec())
    }
}
