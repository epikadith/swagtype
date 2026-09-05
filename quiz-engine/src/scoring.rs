//! Pure scoring logic — no I/O, fully testable.

use crate::question::{AnswerRecord, Question, QuizResult};

/// Check whether the selected option matches the correct one.
pub fn score_answer(selected: usize, correct: usize) -> bool {
    selected == correct
}

/// Compute aggregate quiz results from individual answer records.
pub fn calculate_results(
    answers: &[AnswerRecord],
    _questions: &[Question],
    total_time_secs: f64,
) -> QuizResult {
    let total = answers.len();
    let correct = answers.iter().filter(|a| a.is_correct()).count();
    let accuracy = if total == 0 {
        0.0
    } else {
        (correct as f64 / total as f64) * 100.0
    };

    QuizResult {
        correct,
        total,
        accuracy,
        time_secs: total_time_secs,
        answers: answers.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_answer(selected: usize, correct: usize) -> AnswerRecord {
        AnswerRecord {
            question_index: 0,
            selected,
            correct,
            time_ms: 1000,
        }
    }

    fn make_question() -> Question {
        Question {
            text: "Q?".into(),
            options: vec!["A".into(), "B".into(), "C".into(), "D".into()],
            correct_index: 1,
            category: None,
        }
    }

    #[test]
    fn score_answer_correct() {
        assert!(score_answer(2, 2));
    }

    #[test]
    fn score_answer_incorrect() {
        assert!(!score_answer(1, 3));
    }

    #[test]
    fn calculate_results_all_correct() {
        let answers = vec![
            make_answer(1, 1),
            make_answer(2, 2),
            make_answer(0, 0),
        ];
        let questions = vec![make_question(); 3];
        let result = calculate_results(&answers, &questions, 30.0);

        assert_eq!(result.correct, 3);
        assert_eq!(result.total, 3);
        assert!((result.accuracy - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn calculate_results_all_wrong() {
        let answers = vec![
            make_answer(0, 1),
            make_answer(1, 2),
            make_answer(2, 3),
        ];
        let questions = vec![make_question(); 3];
        let result = calculate_results(&answers, &questions, 45.0);

        assert_eq!(result.correct, 0);
        assert_eq!(result.total, 3);
        assert!((result.accuracy).abs() < f64::EPSILON);
    }

    #[test]
    fn calculate_results_mixed() {
        let answers = vec![
            make_answer(1, 1), // correct
            make_answer(0, 2), // wrong
            make_answer(3, 3), // correct
            make_answer(2, 0), // wrong
        ];
        let questions = vec![make_question(); 4];
        let result = calculate_results(&answers, &questions, 60.0);

        assert_eq!(result.correct, 2);
        assert_eq!(result.total, 4);
        assert!((result.accuracy - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn calculate_results_empty() {
        let result = calculate_results(&[], &[], 0.0);

        assert_eq!(result.correct, 0);
        assert_eq!(result.total, 0);
        assert!((result.accuracy).abs() < f64::EPSILON);
    }

    #[test]
    fn calculate_results_preserves_time() {
        let answers = vec![make_answer(1, 1)];
        let questions = vec![make_question()];
        let result = calculate_results(&answers, &questions, 42.5);

        assert!((result.time_secs - 42.5).abs() < f64::EPSILON);
    }
}
