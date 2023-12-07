use crate::app_state::EncodingType;
use crate::sudoku::string_from_grid;
use cadical::CadicalStats;

#[derive(Clone, Debug)]
pub struct Statistics {
    pub process_time: f64,
    pub real_time: f64,
    pub max_resident_set_size_mb: f64,
    pub conflicts: i64,
    pub learned_clauses: i64,
    pub learned_literals: i64,
    pub decisions: i64,
    pub restarts: i64,
    pub encoding: EncodingType,
    pub clues: Vec<Vec<Option<i32>>>,
    pub sudoku: Vec<Vec<Option<i32>>>,
}

impl Statistics {
    pub fn from_cadical_stats(
        stats: CadicalStats,
        encoding: EncodingType,
        clues: Vec<Vec<Option<i32>>>,
        sudoku: Vec<Vec<Option<i32>>>,
    ) -> Self {
        Self {
            process_time: stats.process_time,
            real_time: stats.real_time,
            max_resident_set_size_mb: stats.max_resident_set_size_mb,
            conflicts: stats.conflicts,
            learned_clauses: stats.learned_clauses,
            learned_literals: stats.learned_literals,
            decisions: stats.decisions,
            restarts: stats.restarts,
            encoding,
            clues,
            sudoku,
        }
    }

    pub fn csv_header() -> String {
        "process_time;\
            real_time;\
            max_resident_set_size_mb;\
            conflicts;\
            learned_clauses;\
            learned_literals;\
            decisions;\
            restarts;\
            is_binary;\
            cell_at_least_one;\
            cell_at_most_one;\
            sudoku_has_all_values;\
            sudoku_has_unique_values;\
            clues;\
            sudoku\n"
            .to_string()
    }

    pub fn csv(&self) -> String {
        let clues_string = string_from_grid(self.clues.clone()).replace('\n', "");
        let sudoku_string = string_from_grid(self.sudoku.clone()).replace('\n', "");
        let (
            is_binary,
            cell_at_least_one,
            cell_at_most_one,
            sudoku_has_all_values,
            sudoku_has_unique_values,
        ) = if let EncodingType::Decimal {
            cell_at_least_one,
            cell_at_most_one,
            sudoku_has_all_values,
            sudoku_has_unique_values,
        } = self.encoding
        {
            (
                false,
                cell_at_least_one,
                cell_at_most_one,
                sudoku_has_all_values,
                sudoku_has_unique_values,
            )
        } else {
            (true, false, false, false, false)
        };
        format!(
            "{};{};{};{};{};{};{};{};{};{};{};{};{};\"{}\";\"{}\"\n",
            self.process_time,
            self.real_time,
            self.max_resident_set_size_mb,
            self.conflicts,
            self.learned_clauses,
            self.learned_literals,
            self.decisions,
            self.restarts,
            is_binary,
            cell_at_least_one,
            cell_at_most_one,
            sudoku_has_all_values,
            sudoku_has_unique_values,
            clues_string,
            sudoku_string,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        app_state::EncodingType,
        cadical_wrapper::CadicalCallbackWrapper,
        sudoku::{get_sudoku, solve_sudoku},
        ConstraintList, Trail,
    };

    use super::Statistics;

    #[test]
    fn test_statistics() {
        let clues = get_sudoku("data/sample_sudoku.txt".to_string()).unwrap();
        let mut solver = cadical::Solver::with_config("plain").unwrap();
        let constraints = ConstraintList::new();
        let callback_wrapper = CadicalCallbackWrapper::new(constraints.clone(), Trail::new());
        solver.set_callbacks(Some(callback_wrapper.clone()));

        let encoding = EncodingType::Decimal {
            cell_at_least_one: true,
            cell_at_most_one: false,
            sudoku_has_all_values: false,
            sudoku_has_unique_values: true,
        };
        let solved_sudoku = solve_sudoku(&clues, &mut solver, &encoding);

        let cadical_stats = solver.stats();
        let stats =
            Statistics::from_cadical_stats(cadical_stats, encoding, clues, solved_sudoku.unwrap());

        assert_eq!(stats.learned_clauses, constraints.len() as i64);
        assert!(stats.real_time > 0.0);
        assert!(stats.max_resident_set_size_mb > 0.0);
        assert!(stats.decisions > 0);
    }
}
