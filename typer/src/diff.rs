//! Character-level diff engine and scoring for the typing test.

/// Status of a typed character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharStatus {
    /// Correctly typed.
    Correct,
    /// Incorrectly typed.
    Incorrect,
    /// Not yet typed.
    Pending,
}

/// A single character in the target sentence and its current status.
#[derive(Debug, Clone)]
pub struct DiffResult {
    /// The character that should be typed.
    pub expected: char,
    /// The character the user actually typed (if any).
    pub actual: Option<char>,
    /// Whether they got it right, wrong, or haven't reached it.
    pub status: CharStatus,
}

/// Compute the character-by-character diff between expected and typed strings.
pub fn compute_diff(expected: &str, typed: &str) -> Vec<DiffResult> {
    let expected_chars: Vec<char> = expected.chars().collect();
    let typed_chars: Vec<char> = typed.chars().collect();

    expected_chars
        .into_iter()
        .enumerate()
        .map(|(i, expected_char)| {
            if i < typed_chars.len() {
                let actual_char = typed_chars[i];
                let status = if expected_char == actual_char {
                    CharStatus::Correct
                } else {
                    CharStatus::Incorrect
                };
                DiffResult {
                    expected: expected_char,
                    actual: Some(actual_char),
                    status,
                }
            } else {
                DiffResult {
                    expected: expected_char,
                    actual: None,
                    status: CharStatus::Pending,
                }
            }
        })
        .collect()
}

/// Calculate Words Per Minute (WPM).
/// Standard formula: (characters typed / 5) / minutes elapsed.
pub fn calculate_wpm(chars_typed: usize, elapsed_secs: f64) -> f64 {
    if elapsed_secs <= 0.0 || chars_typed == 0 {
        return 0.0;
    }
    let minutes = elapsed_secs / 60.0;
    let words = chars_typed as f64 / 5.0;
    words / minutes
}

/// Calculate accuracy percentage.
pub fn calculate_accuracy(diff: &[DiffResult]) -> f64 {
    let mut total_typed = 0;
    let mut correct = 0;

    for d in diff {
        if d.status != CharStatus::Pending {
            total_typed += 1;
            if d.status == CharStatus::Correct {
                correct += 1;
            }
        }
    }

    if total_typed == 0 {
        100.0
    } else {
        (correct as f64 / total_typed as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_diff_all_correct() {
        let diff = compute_diff("hello", "hello");
        assert_eq!(diff.len(), 5);
        assert!(diff.iter().all(|d| d.status == CharStatus::Correct));
    }

    #[test]
    fn compute_diff_partial() {
        let diff = compute_diff("hello", "hel");
        assert_eq!(diff[0].status, CharStatus::Correct);
        assert_eq!(diff[1].status, CharStatus::Correct);
        assert_eq!(diff[2].status, CharStatus::Correct);
        assert_eq!(diff[3].status, CharStatus::Pending);
        assert_eq!(diff[4].status, CharStatus::Pending);
    }

    #[test]
    fn compute_diff_with_errors() {
        let diff = compute_diff("hello", "heXlo");
        assert_eq!(diff[2].status, CharStatus::Incorrect);
        assert_eq!(diff[2].actual, Some('X'));
        assert_eq!(diff[4].status, CharStatus::Correct);
    }

    #[test]
    fn calculate_wpm_correctness() {
        // 50 chars = 10 words. In 12 seconds (0.2 mins), that's 50 WPM.
        let wpm = calculate_wpm(50, 12.0);
        assert!((wpm - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn calculate_wpm_zero_time() {
        assert_eq!(calculate_wpm(50, 0.0), 0.0);
    }

    #[test]
    fn calculate_accuracy_correctness() {
        let diff = compute_diff("hello", "heXlo");
        let acc = calculate_accuracy(&diff);
        // 4 correct out of 5 typed = 80%
        assert!((acc - 80.0).abs() < f64::EPSILON);
    }

    #[test]
    fn calculate_accuracy_empty_typed() {
        let diff = compute_diff("hello", "");
        assert_eq!(calculate_accuracy(&diff), 100.0);
    }
}
