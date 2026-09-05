//! Core data types for MCQ quizzes.

/// A single multiple-choice question.
#[derive(Debug, Clone)]
pub struct Question {
    /// The question text (supports Unicode).
    pub text: String,
    /// Answer options (at least 2).
    pub options: Vec<String>,
    /// Index into `options` of the correct answer.
    pub correct_index: usize,
    /// Optional category label (e.g. "Physics", "Multiplication").
    pub category: Option<String>,
}

impl Question {
    /// Validate that the question is well-formed.
    pub fn is_valid(&self) -> bool {
        self.options.len() >= 2 && self.correct_index < self.options.len()
    }
}

/// Configuration for running a quiz session.
#[derive(Debug, Clone)]
pub struct QuizConfig {
    /// The questions to present.
    pub questions: Vec<Question>,
    /// Display title for the quiz.
    pub title: String,
    /// Whether to shuffle question order before presenting.
    pub shuffle: bool,
}

/// Aggregated results from a completed quiz.
#[derive(Debug, Clone)]
pub struct QuizResult {
    /// Number of correct answers.
    pub correct: usize,
    /// Total number of questions.
    pub total: usize,
    /// Accuracy as a percentage (0.0–100.0).
    pub accuracy: f64,
    /// Total time in seconds.
    pub time_secs: f64,
    /// Per-question breakdown.
    pub answers: Vec<AnswerRecord>,
}

/// Record of a single answered question.
#[derive(Debug, Clone)]
pub struct AnswerRecord {
    /// Index of the question in the original config.
    pub question_index: usize,
    /// The option index the user selected.
    pub selected: usize,
    /// The correct option index.
    pub correct: usize,
    /// Time spent on this question in milliseconds.
    pub time_ms: u64,
}

impl AnswerRecord {
    /// Whether the user answered correctly.
    pub fn is_correct(&self) -> bool {
        self.selected == self.correct
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn question_validity_with_valid_question() {
        let q = Question {
            text: "What is 2+2?".to_string(),
            options: vec!["3".into(), "4".into(), "5".into(), "6".into()],
            correct_index: 1,
            category: None,
        };
        assert!(q.is_valid());
    }

    #[test]
    fn question_invalid_when_correct_index_out_of_bounds() {
        let q = Question {
            text: "Test".to_string(),
            options: vec!["A".into(), "B".into()],
            correct_index: 5,
            category: None,
        };
        assert!(!q.is_valid());
    }

    #[test]
    fn question_invalid_with_fewer_than_two_options() {
        let q = Question {
            text: "Test".to_string(),
            options: vec!["A".into()],
            correct_index: 0,
            category: None,
        };
        assert!(!q.is_valid());
    }

    #[test]
    fn question_valid_with_unicode() {
        let q = Question {
            text: "¿Cuál es la capital de España?".to_string(),
            options: vec!["Madrid".into(), "Barcelona".into(), "Sevilla".into()],
            correct_index: 0,
            category: Some("Geografía".into()),
        };
        assert!(q.is_valid());
    }

    #[test]
    fn answer_record_is_correct() {
        let rec = AnswerRecord {
            question_index: 0,
            selected: 2,
            correct: 2,
            time_ms: 1500,
        };
        assert!(rec.is_correct());
    }

    #[test]
    fn answer_record_is_incorrect() {
        let rec = AnswerRecord {
            question_index: 0,
            selected: 1,
            correct: 2,
            time_ms: 1500,
        };
        assert!(!rec.is_correct());
    }
}
