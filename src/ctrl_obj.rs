use crate::app_state::AppState;
use crate::cnf_var::CnfVariable;
use crate::Trail;
pub trait ControllableObj {
    fn clicked(&self, state: &mut AppState, i: usize);
    fn get_clicked(&self, state: &AppState) -> Option<usize>;
    fn clauses(&self, state: &AppState) -> Vec<Vec<CnfVariable>>;
    fn combiner(&self) -> String;
    fn move_up(&self, state: &mut AppState);
    fn move_down(&self, state: &mut AppState);
}

pub struct ConstraintList {
    pub clauses: Vec<Vec<CnfVariable>>,
    pub combiner: String,
}
pub struct ConflictList {
    pub clauses: Vec<Vec<CnfVariable>>,
    pub combiner: String,
    pub trail: Trail,
}

impl ControllableObj for ConstraintList {
    fn clicked(&self, state: &mut AppState, i: usize) {
        state.clear_trail();
        match state.clicked_constraint_index {
            Some(index) => {
                // clicking constraint again clears little numbers
                if index == i {
                    state.clicked_constraint_index = None;
                } else {
                    state.clicked_constraint_index = Some(i);
                }
            }
            None => state.clicked_constraint_index = Some(i),
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
        state.clicked_constraint_index = Some(current - 1_usize);
    }
    fn move_down(&self, state: &mut AppState) {
        let current: usize = state.clicked_constraint_index.unwrap_or(0);
        state.clicked_constraint_index = Some(current + 1_usize);
    }
}

impl ControllableObj for ConflictList {
    fn clauses(&self, state: &AppState) -> Vec<Vec<CnfVariable>> {
        let start = (state.page_number) as usize * state.page_length;
        let end = (state.page_number + 1) as usize * state.page_length;
        self.clauses.clone()
            [std::cmp::min(start, self.clauses.len())..std::cmp::min(end, self.clauses.len())]
            .to_vec()
    }
    fn combiner(&self) -> String {
        self.combiner.clone()
    }
    fn clicked(&self, state: &mut AppState, i: usize) {
        println!("{:?}",i);
        let old_index = state.clicked_conflict_index;
        let old_page = state.page_number;
        state.clear_filters();
        match old_index {
            Some(index) => {
                if index != i {
                    let trail = self.trail.trail_at_index(i);
                    let enum_trail = trail
                        .iter()
                        .map(|&x| CnfVariable::from_cnf(x, &state.encoding))
                        .collect();
                    let vars = self.clauses[i].clone();
                    state.set_trail(i, (vars[0].clone(), vars[1].clone()), enum_trail);
                }
            }
            None => {
                let trail = self.trail.trail_at_index(i);
                let enum_trail = trail
                    .iter()
                    .map(|&x| CnfVariable::from_cnf(x, &state.encoding))
                    .collect();
                    let vars = self.clauses[i].clone();
                    state.set_trail(i, (vars[0].clone(), vars[1].clone()), enum_trail);
            }
        }
        state.page_number = old_page;
    }
    fn get_clicked(&self, state: &AppState) -> Option<usize> {
        state.clicked_conflict_index
    }
    fn move_up(&self, state: &mut AppState) {
        let current: usize = state.clicked_conflict_index.unwrap_or(0);
        self.clicked(state, current - 1_usize);
    }
    fn move_down(&self, state: &mut AppState) {
        let current: usize = state.clicked_conflict_index.unwrap_or(0);
        self.clicked(state, current + 1_usize);
    }
}
