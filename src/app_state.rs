use crate::{filtering::ListFilter, parse_numeric_input, ConstraintList};

pub struct AppState {
    filter: ListFilter,
    pub max_length: Option<i32>,
    pub max_length_input: String,
    pub selected_cell: Option<(i32, i32)>,
    pub clicked_constraint_index: Option<usize>,
    pub page_number: usize,
    pub page_length: usize,
    pub page_length_input: String,
    pub filtered_length: usize,
}

impl AppState {
    pub fn new(constraints: ConstraintList) -> Self {
        let filter = ListFilter::new(constraints.clone());
        Self {
            filter,
            max_length: None,
            max_length_input: String::new(),
            selected_cell: None,
            clicked_constraint_index: None,
            page_number: 0,
            page_length: 100,
            page_length_input: "100".to_string(),
            filtered_length: 0,
        }
    }

    pub fn get_filtered(&mut self) -> Vec<Vec<i32>> {
        let (list, length) = self.filter.get_filtered(self.page_number, self.page_length);
        self.filtered_length = length;
        list
    }

    pub fn reinit(&mut self) {
        self.clear_filters();
        self.filter.reinit();

        self.page_number = 0;
        self.page_length = 100;
        self.page_length_input = "100".to_string();
        self.filtered_length = 0;
    }

    pub fn filter_by_max_length(&mut self) {
        self.max_length = parse_numeric_input(self.max_length_input.as_str());

        if let Some(max_length) = self.max_length {
            self.clicked_constraint_index = None;
            self.page_number = 0;

            self.filter.by_max_length(max_length);
        }
    }

    pub fn select_cell(&mut self, row: i32, col: i32) {
        self.clicked_constraint_index = None;
        self.page_number = 0;

        self.selected_cell = Some((row, col));
        self.filter.by_cell(row, col);
    }

    pub fn select_constraint(&mut self, index: usize) {
        self.clicked_constraint_index = Some(index);
    }

    pub fn clear_selected_constraint(&mut self) {
        self.clicked_constraint_index = None;
    }

    pub fn clear_filters(&mut self) {
        self.clicked_constraint_index = None;
        self.page_number = 0;

        self.clear_length();
        self.clear_cell();
    }

    pub fn clear_length(&mut self) {
        self.clicked_constraint_index = None;
        self.page_number = 0;

        self.max_length = None;
        self.max_length_input = "".to_string();
        self.filter.clear_length();
    }

    pub fn clear_cell(&mut self) {
        self.clicked_constraint_index = None;
        self.page_number = 0;

        self.selected_cell = None;
        self.filter.clear_cell();
    }
}
