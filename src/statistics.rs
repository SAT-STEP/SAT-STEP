use crate::app_state::EncodingType;
use cadical::CadicalStats;


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
    pub sudoku: Vec<Vec<Option<i32>>>,
}


impl Statistics {
    pub fn from_cadical_stats(stats: CadicalStats, encoding: EncodingType, sudoku: Vec<Vec<Option<i32>>>) -> Self {
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
            sudoku,
        }
    }
}
