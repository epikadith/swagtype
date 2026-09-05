//! Persistent stats storage backed by SQLite.
//!
//! Stores typing-test and quiz-run history in a local database so users can
//! track progress over time.  Uses an in-process SQLite database via `rusqlite`.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A recorded typing-test run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingRun {
    pub id: Option<i64>,
    pub wpm: f64,
    pub accuracy: f64,
    pub time_secs: f64,
    pub sentence_length: usize,
    pub timestamp: DateTime<Utc>,
}

/// A recorded quiz run (mathwiz, sciencewiz, or customwiz).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizRun {
    pub id: Option<i64>,
    pub subtool: String,
    pub score: usize,
    pub total: usize,
    pub accuracy: f64,
    pub time_secs: f64,
    pub category: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Handle to the stats database.
pub struct StatsDb {
    conn: Connection,
}

impl StatsDb {
    /// Open (or create) the stats database at the given path.
    pub fn open(path: &Path) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Open an in-memory database (useful for testing).
    pub fn open_in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Create tables if they don't already exist.
    fn init_schema(&self) -> SqlResult<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS typing_runs (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                wpm             REAL    NOT NULL,
                accuracy        REAL    NOT NULL,
                time_secs       REAL    NOT NULL,
                sentence_length INTEGER NOT NULL,
                timestamp       TEXT    NOT NULL
            );

            CREATE TABLE IF NOT EXISTS quiz_runs (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                subtool   TEXT    NOT NULL,
                score     INTEGER NOT NULL,
                total     INTEGER NOT NULL,
                accuracy  REAL    NOT NULL,
                time_secs REAL    NOT NULL,
                category  TEXT,
                timestamp TEXT    NOT NULL
            );",
        )?;
        Ok(())
    }

    // ── Typing runs ──────────────────────────────────────────────────────

    /// Record a completed typing run.
    pub fn record_typing_run(&self, run: &TypingRun) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO typing_runs (wpm, accuracy, time_secs, sentence_length, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                run.wpm,
                run.accuracy,
                run.time_secs,
                run.sentence_length as i64,
                run.timestamp.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Retrieve typing-run history, most recent first.
    pub fn get_typing_history(&self, limit: usize) -> SqlResult<Vec<TypingRun>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, wpm, accuracy, time_secs, sentence_length, timestamp
             FROM typing_runs
             ORDER BY timestamp DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            let ts_str: String = row.get(5)?;
            let timestamp = DateTime::parse_from_rfc3339(&ts_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e)))?;

            Ok(TypingRun {
                id: Some(row.get(0)?),
                wpm: row.get(1)?,
                accuracy: row.get(2)?,
                time_secs: row.get(3)?,
                sentence_length: row.get::<_, i64>(4)? as usize,
                timestamp,
            })
        })?;

        rows.collect()
    }

    // ── Quiz runs ────────────────────────────────────────────────────────

    /// Record a completed quiz run.
    pub fn record_quiz_run(&self, run: &QuizRun) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO quiz_runs (subtool, score, total, accuracy, time_secs, category, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                run.subtool,
                run.score as i64,
                run.total as i64,
                run.accuracy,
                run.time_secs,
                run.category,
                run.timestamp.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Clear all history from the database.
    pub fn clear_history(&self) -> SqlResult<()> {
        self.conn.execute("DELETE FROM quiz_runs", [])?;
        self.conn.execute("DELETE FROM typing_runs", [])?; // Just in case any old typing_runs exist
        Ok(())
    }

    /// Retrieve quiz-run history, most recent first.  Optionally filter by
    /// subtool name (e.g. `"mathwiz"`).
    pub fn get_quiz_history(
        &self,
        subtool_filter: Option<&str>,
        limit: usize,
    ) -> SqlResult<Vec<QuizRun>> {
        let (sql, filter_val);
        match subtool_filter {
            Some(s) => {
                sql = "SELECT id, subtool, score, total, accuracy, time_secs, category, timestamp
                       FROM quiz_runs
                       WHERE subtool = ?1
                       ORDER BY timestamp DESC
                       LIMIT ?2";
                filter_val = Some(s.to_owned());
            }
            None => {
                sql = "SELECT id, subtool, score, total, accuracy, time_secs, category, timestamp
                       FROM quiz_runs
                       ORDER BY timestamp DESC
                       LIMIT ?1";
                filter_val = None;
            }
        }

        let mut stmt = self.conn.prepare(sql)?;

        let rows = if let Some(ref f) = filter_val {
            stmt.query_map(params![f, limit as i64], Self::map_quiz_row)?
        } else {
            stmt.query_map(params![limit as i64], Self::map_quiz_row)?
        };

        rows.collect()
    }

    fn map_quiz_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<QuizRun> {
        let ts_str: String = row.get(7)?;
        let timestamp = DateTime::parse_from_rfc3339(&ts_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    7,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

        Ok(QuizRun {
            id: Some(row.get(0)?),
            subtool: row.get(1)?,
            score: row.get::<_, i64>(2)? as usize,
            total: row.get::<_, i64>(3)? as usize,
            accuracy: row.get(4)?,
            time_secs: row.get(5)?,
            category: row.get(6)?,
            timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_typing_run(wpm: f64, accuracy: f64) -> TypingRun {
        TypingRun {
            id: None,
            wpm,
            accuracy,
            time_secs: 30.0,
            sentence_length: 50,
            timestamp: Utc::now(),
        }
    }

    fn make_quiz_run(subtool: &str, score: usize, total: usize) -> QuizRun {
        QuizRun {
            id: None,
            subtool: subtool.to_owned(),
            score,
            total,
            accuracy: score as f64 / total as f64 * 100.0,
            time_secs: 60.0,
            category: Some("general".to_owned()),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn open_in_memory_succeeds() {
        let db = StatsDb::open_in_memory();
        assert!(db.is_ok());
    }

    #[test]
    fn record_and_retrieve_typing_run() {
        let db = StatsDb::open_in_memory().unwrap();
        let run = make_typing_run(75.0, 96.5);
        db.record_typing_run(&run).unwrap();

        let history = db.get_typing_history(10).unwrap();
        assert_eq!(history.len(), 1);
        assert!((history[0].wpm - 75.0).abs() < f64::EPSILON);
        assert!((history[0].accuracy - 96.5).abs() < f64::EPSILON);
        assert_eq!(history[0].sentence_length, 50);
    }

    #[test]
    fn typing_history_ordered_most_recent_first() {
        let db = StatsDb::open_in_memory().unwrap();

        // Insert two runs with slightly different timestamps.
        let mut run1 = make_typing_run(60.0, 90.0);
        run1.timestamp = Utc::now() - chrono::Duration::seconds(10);
        db.record_typing_run(&run1).unwrap();

        let run2 = make_typing_run(80.0, 98.0);
        db.record_typing_run(&run2).unwrap();

        let history = db.get_typing_history(10).unwrap();
        assert_eq!(history.len(), 2);
        // Most recent (run2, 80 WPM) should be first.
        assert!((history[0].wpm - 80.0).abs() < f64::EPSILON);
        assert!((history[1].wpm - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn typing_history_respects_limit() {
        let db = StatsDb::open_in_memory().unwrap();
        for i in 0..5 {
            db.record_typing_run(&make_typing_run(50.0 + i as f64, 90.0))
                .unwrap();
        }
        let history = db.get_typing_history(3).unwrap();
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn record_and_retrieve_quiz_run() {
        let db = StatsDb::open_in_memory().unwrap();
        let run = make_quiz_run("mathwiz", 8, 10);
        db.record_quiz_run(&run).unwrap();

        let history = db.get_quiz_history(None, 10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].subtool, "mathwiz");
        assert_eq!(history[0].score, 8);
        assert_eq!(history[0].total, 10);
    }

    #[test]
    fn quiz_history_filters_by_subtool() {
        let db = StatsDb::open_in_memory().unwrap();
        db.record_quiz_run(&make_quiz_run("mathwiz", 7, 10))
            .unwrap();
        db.record_quiz_run(&make_quiz_run("sciencewiz", 9, 10))
            .unwrap();
        db.record_quiz_run(&make_quiz_run("mathwiz", 6, 10))
            .unwrap();

        let math = db.get_quiz_history(Some("mathwiz"), 10).unwrap();
        assert_eq!(math.len(), 2);
        assert!(math.iter().all(|r| r.subtool == "mathwiz"));

        let science = db.get_quiz_history(Some("sciencewiz"), 10).unwrap();
        assert_eq!(science.len(), 1);

        let all = db.get_quiz_history(None, 10).unwrap();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn quiz_history_respects_limit() {
        let db = StatsDb::open_in_memory().unwrap();
        for _ in 0..5 {
            db.record_quiz_run(&make_quiz_run("mathwiz", 5, 10))
                .unwrap();
        }
        let history = db.get_quiz_history(None, 2).unwrap();
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn empty_history_returns_empty_vec() {
        let db = StatsDb::open_in_memory().unwrap();
        assert!(db.get_typing_history(10).unwrap().is_empty());
        assert!(db.get_quiz_history(None, 10).unwrap().is_empty());
    }

    #[test]
    fn quiz_run_category_can_be_none() {
        let db = StatsDb::open_in_memory().unwrap();
        let mut run = make_quiz_run("customwiz", 3, 5);
        run.category = None;
        db.record_quiz_run(&run).unwrap();

        let history = db.get_quiz_history(None, 10).unwrap();
        assert_eq!(history.len(), 1);
        assert!(history[0].category.is_none());
    }
}
