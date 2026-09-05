//! Runs the sciencewiz interactive session.

use crate::bank::{get_questions, ScienceCategory};
use core::config::ensure_data_dir;
use core::stats::{QuizRun, StatsDb};
use quiz_engine::{QuizConfig, QuizEndAction};
use std::io;
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct SciencewizConfig {
    pub category: Option<ScienceCategory>,
    pub count: usize,
}

/// Run sciencewiz and return the end action (None if early quit).
pub fn run_sciencewiz(config: SciencewizConfig) -> io::Result<Option<QuizEndAction>> {
    let questions = get_questions(config.category, config.count);

    if questions.is_empty() {
        println!("No questions found for the given criteria.");
        return Ok(None);
    }

    let title = if let Some(cat) = config.category {
        format!("{:?} Science Quiz", cat)
    } else {
        "General Science Quiz".to_string()
    };

    let quiz = QuizConfig {
        questions,
        title,
        shuffle: false,
    };

    let result_opt = quiz_engine::run_quiz(quiz)?;

    if let Some((result, action)) = result_opt {
        let run = QuizRun {
            id: None,
            subtool: "sciencewiz".to_string(),
            score: result.correct,
            total: result.total,
            accuracy: result.accuracy,
            time_secs: result.time_secs,
            category: config.category.map(|c| format!("{:?}", c)),
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
