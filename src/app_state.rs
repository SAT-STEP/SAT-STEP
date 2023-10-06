use crate::{filtering::ListFilter, ConstraintList,};

pub struct AppState {
    pub filter: ListFilter,
    pub max_length: Option<i32>,
    pub max_length_input: String,
    pub selected_cell: Option<(i32, i32)>,
    pub page_number: usize,
    pub page_length: usize,
    pub page_length_input: String,
}

impl AppState {
    pub fn new(constraints: ConstraintList) -> Self {
        let filter = ListFilter::new(constraints.clone());
        Self {
            filter,
            max_length: None,
            max_length_input: String::new(),
            selected_cell: None,
            page_number: 0,
            page_length: 50,
            page_length_input: String::new(),
        }
    }
}
