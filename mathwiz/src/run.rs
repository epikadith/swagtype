//! Runs the mathwiz interactive session.

use crate::generators::{generate_quiz, MathCategory, MathDifficulty};
use core::config::ensure_data_dir;
use core::stats::{QuizRun, StatsDb};
use quiz_engine::QuizEndAction;
use std::io;
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct MathwizConfig {
    pub category: MathCategory,
    pub difficulty: MathDifficulty,
    pub count: usize,
}

/// Run mathwiz and return the end action (None if early quit).
pub fn run_mathwiz(config: MathwizConfig) -> io::Result<Option<QuizEndAction>> {
    let quiz = generate_quiz(config.category, config.difficulty, config.count);

    let result_opt = quiz_engine::run_quiz(quiz)?;

    if let Some((result, action)) = result_opt {
        let run = QuizRun {
            id: None,
            subtool: "mathwiz".to_string(),
            score: result.correct,
            total: result.total,
            accuracy: result.accuracy,
            time_secs: result.time_secs,
            category: Some(format!("{:?} ({:?})", config.category, config.difficulty)),
            timestamp: Utc::now(),
        };
        if let Ok(dir) = ensure_data_dir() {
            let db_path = dir.join("stats.db");
            if let Ok(db) = StatsDb::open(&db_path) {
                let _ = db.record_quiz_run(&run);
            }
        }
        Ok(Some(action))
    } else {
        Ok(None)
    }
}
