//! Traits and structs for constraints and conflicts. Displayed using controllable_list.rs

use crate::app_state::AppState;
use crate::cnf::CnfVariable;
use crate::Trail;

/// Trait for listed data e.g. constraints or conflicts
/// Replaces calls to specific lists in gui code
pub trait ControllableObj {
    fn clicked(&self, state: &mut AppState, i: usize);
    fn get_clicked(&self, state: &AppState) -> Option<usize>;
    fn clauses(&self, state: &AppState) -> Vec<Vec<CnfVariable>>;
    fn combiner(&self) -> String;
    fn move_up(&self, state: &mut AppState);
    fn move_down(&self, state: &mut AppState);
}

/// Struct for constraints to be shown in gui
pub struct ConstraintList {
    pub clauses: Vec<Vec<CnfVariable>>,
    pub trail: Trail,
    /// Combiner string to join variables for gui
    pub combiner: String,
}

impl ControllableObj for ConstraintList {
    fn clicked(&self, state: &mut AppState, i: usize) {
        match state.clicked_constraint_index {
            Some(index) => {
                // clicking constraint again clears little numbers
                if index == i {
                    state.clicked_constraint_index = None;
                    state.clear_trail();
                } else {
                    state.clicked_constraint_index = Some(i);

                    let trail = self.trail.trail_at_index(i);
                    let enum_trail = trail
                        .iter()
                        .map(|&x| CnfVariable::from_cnf(x, &state.encoding))
                        .collect();

                    let literals = self.trail.literals_at_index(i);
                    let enum_literals = literals
                        .iter()
                        .map(|&x| CnfVariable::from_cnf(x, &state.encoding))
                        .collect();

                    state.set_trail(
                        enum_literals,
                        enum_trail,
                        self.trail.var_is_propagated_at_index(i),
                    );
                }
            }
            None => {
                state.clicked_constraint_index = Some(i);

                let trail = self.trail.trail_at_index(i);
                let enum_trail = trail
                    .iter()
                    .map(|&x| CnfVariable::from_cnf(x, &state.encoding))
                    .collect();

                let literals = self.trail.literals_at_index(i);
                let enum_literals = literals
                    .iter()
                    .map(|&x| CnfVariable::from_cnf(x, &state.encoding))
                    .collect();

                state.set_trail(
                    enum_literals,
                    enum_trail,
                    self.trail.var_is_propagated_at_index(i),
                );
            }
        }
    }
    fn get_clicked(&self, state: &AppState) -> Option<usize> {
        state.clicked_constraint_index
    }
    fn clauses(&self, _state: &AppState) -> Vec<Vec<CnfVariable>> {
        self.clauses.clone()
    }
    fn combiner(&self) -> String {
        self.combiner.clone()
    }
    fn move_up(&self, state: &mut AppState) {
        let current: usize = state.clicked_constraint_index.unwrap_or(0);
        self.clicked(state, current - 1_usize);
    }
    fn move_down(&self, state: &mut AppState) {
        let current: usize = state.clicked_constraint_index.unwrap_or(0);
        self.clicked(state, current + 1_usize);
    }
}
