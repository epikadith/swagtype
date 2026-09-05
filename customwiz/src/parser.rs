//! Parser for custom JSON quizzes.

use quiz_engine::Question;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomQuizFile {
    pub title: String,
    pub questions: Vec<CustomQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomQuestion {
    pub text: String,
    pub options: Vec<String>,
    pub correct_index: usize,
    pub category: Option<String>,
}

/// Load and parse a custom quiz from a JSON file.
pub fn load_quiz_from_file(path: &Path) -> io::Result<(String, Vec<Question>)> {
    let content = fs::read_to_string(path)?;
    let parsed: CustomQuizFile = serde_json::from_str(&content).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, format!("JSON error: {}", e))
    })?;

    let questions = parsed
        .questions
        .into_iter()
        .map(|cq| Question {
            text: cq.text,
            options: cq.options,
            correct_index: cq.correct_index,
            category: cq.category,
        })
        .collect::<Vec<_>>();

    // Validate questions
    for (i, q) in questions.iter().enumerate() {
        if !q.is_valid() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid question at index {}", i),
            ));
        }
    }

    Ok((parsed.title, questions))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_json_parsing() {
        let json = r#"{
            "title": "My Custom Quiz",
            "questions": [
                {
                    "text": "What is 2 + 2?",
                    "options": ["3", "4", "5"],
                    "correct_index": 1,
                    "category": "Math"
                }
            ]
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", json).unwrap();

        let (title, questions) = load_quiz_from_file(file.path()).unwrap();
        assert_eq!(title, "My Custom Quiz");
        assert_eq!(questions.len(), 1);
        assert_eq!(questions[0].text, "What is 2 + 2?");
        assert_eq!(questions[0].correct_index, 1);
    }

    #[test]
    fn test_invalid_json_format() {
        let json = r#"{ "title": "Bad Quiz" }"#; // Missing questions
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", json).unwrap();

        let result = load_quiz_from_file(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_question_data() {
        let json = r#"{
            "title": "Bad Options",
            "questions": [
                {
                    "text": "Too few options?",
                    "options": ["Yes"],
                    "correct_index": 0
                }
            ]
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", json).unwrap();

        let result = load_quiz_from_file(file.path());
        assert!(result.is_err()); // Validation fails due to < 2 options
    }
}
