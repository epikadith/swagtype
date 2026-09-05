//! Runs a customwiz interactive session.

use crate::parser::load_quiz_from_file;
use core::config::ensure_data_dir;
use core::stats::{QuizRun, StatsDb};
use quiz_engine::{QuizConfig, QuizEndAction};
use std::io;
use std::path::PathBuf;
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct CustomwizConfig {
    pub file_path: PathBuf,
}

pub fn run_customwiz(config: CustomwizConfig) -> io::Result<Option<QuizEndAction>> {
    // Load and parse — convert technical errors to friendly messages
    let (title, questions) = load_quiz_from_file(&config.file_path).map_err(|e| {
        let friendly = match e.kind() {
            io::ErrorKind::NotFound => format!(
                "Could not find the file: '{}'\nPlease check the path and try again.",
                config.file_path.display()
            ),
            io::ErrorKind::InvalidData => {
                let msg = e.to_string();
                // Strip the leading "JSON error: " prefix for cleaner output
                let detail = msg.strip_prefix("JSON error: ").unwrap_or(&msg);
                // Make common serde errors human-readable
                if detail.contains("missing field") {
                        let field = detail.split('`').nth(1).unwrap_or("unknown");
                        format!("The quiz file is missing a required field: '{field}'.\nPlease check that your JSON file matches the expected format.")
                    } else if detail.contains("invalid type") || detail.contains("expected") {
                        format!("The quiz file has an unexpected value in it.\nDetails: {detail}\nPlease check that your JSON file matches the expected format.")
                    } else {
                        format!("The quiz file has invalid JSON.\nDetails: {detail}")
                    }
            }
            _ => format!(
                "Failed to read the quiz file: '{}'\n{}",
                config.file_path.display(),
                e
            ),
        };
        io::Error::new(e.kind(), friendly)
    })?;

    if questions.is_empty() {
        eprintln!("The quiz file contains no questions.");
        return Ok(None);
    }

    let quiz = QuizConfig {
        questions,
        title: title.clone(),
        shuffle: false,
    };

    let result_opt = quiz_engine::run_quiz(quiz)?;

    if let Some((result, action)) = result_opt {
        let run = QuizRun {
            id: None,
            subtool: "customwiz".to_string(),
            score: result.correct,
            total: result.total,
            accuracy: result.accuracy,
            time_secs: result.time_secs,
            category: Some(title),
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
