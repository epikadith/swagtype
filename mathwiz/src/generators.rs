//! Procedural math question generators.

use quiz_engine::Question;
use rand::seq::IndexedRandom;
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathCategory {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Algebra,
    Mixed,
}

impl std::str::FromStr for MathCategory {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "add" | "addition" => Ok(MathCategory::Addition),
            "sub" | "subtraction" => Ok(MathCategory::Subtraction),
            "mul" | "multiplication" => Ok(MathCategory::Multiplication),
            "div" | "division" => Ok(MathCategory::Division),
            "algebra" => Ok(MathCategory::Algebra),
            "mixed" => Ok(MathCategory::Mixed),
            _ => Err("invalid category"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathDifficulty {
    Easy,
    Medium,
    Hard,
}

impl std::str::FromStr for MathDifficulty {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "easy" => Ok(MathDifficulty::Easy),
            "medium" => Ok(MathDifficulty::Medium),
            "hard" => Ok(MathDifficulty::Hard),
            _ => Err("invalid difficulty"),
        }
    }
}

pub fn generate_question(
    category: MathCategory,
    difficulty: MathDifficulty,
) -> Question {
    let mut rng = rand::rng();
    
    let actual_cat = if category == MathCategory::Mixed {
        let cats = [
            MathCategory::Addition,
            MathCategory::Subtraction,
            MathCategory::Multiplication,
            MathCategory::Division,
            MathCategory::Algebra,
        ];
        *cats.choose(&mut rng).unwrap()
    } else {
        category
    };

    let (text, answer, distractors) = match actual_cat {
        MathCategory::Addition => generate_addition(difficulty, &mut rng),
        MathCategory::Subtraction => generate_subtraction(difficulty, &mut rng),
        MathCategory::Multiplication => generate_multiplication(difficulty, &mut rng),
        MathCategory::Division => generate_division(difficulty, &mut rng),
        MathCategory::Algebra => generate_algebra(difficulty, &mut rng),
        MathCategory::Mixed => unreachable!(),
    };

    let mut options = vec![answer.to_string()];
    for d in distractors {
        options.push(d.to_string());
    }
    
    // Shuffle options
    options.shuffle(&mut rng);
    let correct_index = options.iter().position(|o| o == &answer.to_string()).unwrap();

    Question {
        text,
        options,
        correct_index,
        category: Some(format!("{:?}", actual_cat)),
    }
}

pub fn generate_quiz(
    category: MathCategory,
    difficulty: MathDifficulty,
    count: usize,
) -> quiz_engine::QuizConfig {
    let questions = (0..count)
        .map(|_| generate_question(category, difficulty))
        .collect();

    quiz_engine::QuizConfig {
        questions,
        title: format!("{:?} ({:?})", category, difficulty),
        shuffle: false, // Questions are already randomly generated
    }
}

// Generators return (Question Text, Correct Answer, Vec<Distractors>)

fn generate_addition(diff: MathDifficulty, rng: &mut impl Rng) -> (String, i64, Vec<i64>) {
    let (min, max) = match diff {
        MathDifficulty::Easy => (1, 10),
        MathDifficulty::Medium => (10, 100),
        MathDifficulty::Hard => (100, 1000),
    };
    let a: i64 = rng.random_range(min..=max);
    let b: i64 = rng.random_range(min..=max);
    let ans = a + b;
    let dist = generate_distractors(ans, rng);
    (format!("What is {} + {}?", a, b), ans, dist)
}

fn generate_subtraction(diff: MathDifficulty, rng: &mut impl Rng) -> (String, i64, Vec<i64>) {
    let (min, max) = match diff {
        MathDifficulty::Easy => (1, 10),
        MathDifficulty::Medium => (10, 100),
        MathDifficulty::Hard => (100, 1000),
    };
    let a: i64 = rng.random_range(min..=max);
    let b: i64 = rng.random_range(min..=max);
    // Ensure positive results to keep it simple, or allow negative
    let (x, y) = if a > b { (a, b) } else { (b, a) };
    let ans = x - y;
    let dist = generate_distractors(ans, rng);
    (format!("What is {} - {}?", x, y), ans, dist)
}

fn generate_multiplication(diff: MathDifficulty, rng: &mut impl Rng) -> (String, i64, Vec<i64>) {
    let (min1, max1, min2, max2) = match diff {
        MathDifficulty::Easy => (2, 10, 2, 10),
        MathDifficulty::Medium => (5, 20, 5, 20),
        MathDifficulty::Hard => (10, 50, 10, 50),
    };
    let a: i64 = rng.random_range(min1..=max1);
    let b: i64 = rng.random_range(min2..=max2);
    let ans = a * b;
    let dist = generate_distractors(ans, rng);
    (format!("What is {} * {}?", a, b), ans, dist)
}

fn generate_division(diff: MathDifficulty, rng: &mut impl Rng) -> (String, i64, Vec<i64>) {
    // Generate division that results in a clean integer
    let (min_divisor, max_divisor, min_ans, max_ans) = match diff {
        MathDifficulty::Easy => (2, 10, 1, 10),
        MathDifficulty::Medium => (3, 20, 5, 20),
        MathDifficulty::Hard => (5, 30, 10, 30),
    };
    let divisor: i64 = rng.random_range(min_divisor..=max_divisor);
    let ans: i64 = rng.random_range(min_ans..=max_ans);
    let dividend = divisor * ans;
    let dist = generate_distractors(ans, rng);
    (format!("What is {} / {}?", dividend, divisor), ans, dist)
}

fn generate_algebra(diff: MathDifficulty, rng: &mut impl Rng) -> (String, i64, Vec<i64>) {
    let (min, max) = match diff {
        MathDifficulty::Easy => (1, 10),
        MathDifficulty::Medium => (5, 20),
        MathDifficulty::Hard => (10, 50),
    };
    let x: i64 = rng.random_range(min..=max);
    
    // Choose operation type
    let op = rng.random_range(0..3);
    let (text, ans) = match op {
        0 => { // a * x = b
            let a: i64 = rng.random_range(2..10);
            let b = a * x;
            (format!("Solve for x: {}x = {}", a, b), x)
        }
        1 => { // x + a = b
            let a: i64 = rng.random_range(min..=max);
            let b = x + a;
            (format!("Solve for x: x + {} = {}", a, b), x)
        }
        _ => { // a * x + b = c
            let a: i64 = rng.random_range(2..10);
            let b: i64 = rng.random_range(1..20);
            let c = a * x + b;
            (format!("Solve for x: {}x + {} = {}", a, b, c), x)
        }
    };
    let dist = generate_distractors(ans, rng);
    (text, ans, dist)
}

/// Generates 3 plausible wrong answers (distractors) that are unique.
fn generate_distractors(correct: i64, rng: &mut impl Rng) -> Vec<i64> {
    let mut distractors = Vec::new();
    while distractors.len() < 3 {
        // Vary by small offsets, or multiples
        let offset = rng.random_range(1..=10);
        let sign = if rng.random_bool(0.5) { 1 } else { -1 };
        
        let candidate = if rng.random_bool(0.1) && correct > 0 {
            correct * 10
        } else {
            correct + (offset * sign)
        };

        if candidate != correct && !distractors.contains(&candidate) {
            distractors.push(candidate);
        }
    }
    distractors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_generation() {
        let q = generate_question(MathCategory::Addition, MathDifficulty::Easy);
        assert_eq!(q.options.len(), 4);
        assert!(q.is_valid());
    }

    #[test]
    fn test_quiz_generation() {
        let quiz = generate_quiz(MathCategory::Multiplication, MathDifficulty::Medium, 5);
        assert_eq!(quiz.questions.len(), 5);
        for q in quiz.questions {
            assert!(q.is_valid());
        }
    }

    #[test]
    fn test_distractor_uniqueness() {
        let mut rng = rand::rng();
        let dist = generate_distractors(42, &mut rng);
        assert_eq!(dist.len(), 3);
        assert!(!dist.contains(&42));
        assert_ne!(dist[0], dist[1]);
        assert_ne!(dist[0], dist[2]);
        assert_ne!(dist[1], dist[2]);
    }

    #[test]
    fn test_from_str_category() {
        assert_eq!("add".parse::<MathCategory>(), Ok(MathCategory::Addition));
        assert_eq!("Mixed".parse::<MathCategory>(), Ok(MathCategory::Mixed));
        assert!("invalid".parse::<MathCategory>().is_err());
    }

    #[test]
    fn test_from_str_difficulty() {
        assert_eq!("easy".parse::<MathDifficulty>(), Ok(MathDifficulty::Easy));
        assert_eq!("HARD".parse::<MathDifficulty>(), Ok(MathDifficulty::Hard));
        assert!("invalid".parse::<MathDifficulty>().is_err());
    }
}
