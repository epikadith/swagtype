//! Interactive ratatui-based quiz renderer with arrow-key navigation.
//!
//! This module handles all terminal I/O for the MCQ quiz experience:
//! question display, option highlighting, timer, and end-screen summary.

use crate::question::{AnswerRecord, QuizConfig, QuizResult};
use crate::scoring::calculate_results;
use core::timer::Timer;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    DefaultTerminal, Frame,
};
use std::time::Duration;

/// The action selected on the results end screen.
#[derive(Debug, Clone, PartialEq)]
pub enum QuizEndAction {
    NewQuiz,
    History,
    Quit,
}

const END_BUTTONS: &[(&str, QuizEndAction)] = &[
    ("New Quiz", QuizEndAction::NewQuiz),
    ("History", QuizEndAction::History),
    ("Quit", QuizEndAction::Quit),
];

/// Internal state machine for the quiz UI.
enum Phase {
    /// Actively answering questions.
    Active,
    /// Quiz complete, showing results.
    Results,
}

/// The ratatui application state.
struct QuizApp {
    config: QuizConfig,
    /// Index order of questions (supports shuffle).
    order: Vec<usize>,
    /// Current position in `order`.
    current: usize,
    /// Currently highlighted option (arrow-key controlled).
    selected_option: usize,
    /// Collected answers.
    answers: Vec<AnswerRecord>,
    /// Overall quiz timer.
    quiz_timer: Timer,
    /// Per-question timer.
    question_timer: Timer,
    /// Current UI phase.
    phase: Phase,
    /// Computed results (populated on completion).
    result: Option<QuizResult>,
    /// Scroll offset for results screen
    results_scroll: usize,
    /// Selected end-screen button
    selected_end_button: usize,
}

impl QuizApp {
    fn new(config: QuizConfig) -> Self {
        let mut order: Vec<usize> = (0..config.questions.len()).collect();
        if config.shuffle {
            // Simple Fisher-Yates using std::time for seed (good enough for a quiz).
            use std::time::SystemTime;
            let seed = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos() as usize;
            let len = order.len();
            for i in (1..len).rev() {
                let j = (seed.wrapping_mul(i + 1).wrapping_add(7)) % (i + 1);
                order.swap(i, j);
            }
        }

        let now = Timer::start();
        Self {
            config,
            order,
            current: 0,
            selected_option: 0,
            answers: Vec::new(),
            quiz_timer: now.clone(),
            question_timer: now,
            phase: Phase::Active,
            result: None,
            results_scroll: 0,
            selected_end_button: 0,
        }
    }

    fn current_question_index(&self) -> usize {
        self.order[self.current]
    }

    fn total_questions(&self) -> usize {
        self.order.len()
    }

    fn handle_key(&mut self, key: KeyCode) {
        match self.phase {
            Phase::Active => self.handle_active_key(key),
            Phase::Results => self.handle_results_key(key),
        }
    }

    fn handle_active_key(&mut self, key: KeyCode) {
        let num_options = self.config.questions[self.current_question_index()]
            .options
            .len();

        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_option > 0 {
                    self.selected_option -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_option + 1 < num_options {
                    self.selected_option += 1;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.submit_answer();
            }
            _ => {}
        }
    }

    fn handle_results_key(&mut self, key: KeyCode) {
        let max_scroll = self.result.as_ref().map(|r| r.answers.len().saturating_sub(1)).unwrap_or(0);
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.results_scroll > 0 {
                    self.results_scroll -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.results_scroll < max_scroll {
                    self.results_scroll += 1;
                }
            }
            KeyCode::Left => {
                if self.selected_end_button > 0 {
                    self.selected_end_button -= 1;
                }
            }
            KeyCode::Right => {
                if self.selected_end_button + 1 < END_BUTTONS.len() {
                    self.selected_end_button += 1;
                }
            }
            _ => {}
        }
    }

    fn submit_answer(&mut self) {
        let qi = self.current_question_index();
        let correct = self.config.questions[qi].correct_index;
        let elapsed_ms = self.question_timer.elapsed().as_millis() as u64;

        self.answers.push(AnswerRecord {
            question_index: qi,
            selected: self.selected_option,
            correct,
            time_ms: elapsed_ms,
        });

        if self.current + 1 < self.total_questions() {
            self.current += 1;
            self.selected_option = 0;
            self.question_timer = Timer::start();
        } else {
            // Quiz complete.
            let total_time = self.quiz_timer.elapsed_secs();
            self.result = Some(calculate_results(
                &self.answers,
                &self.config.questions,
                total_time,
            ));
            self.phase = Phase::Results;
        }
    }

    fn draw(&self, frame: &mut Frame) {
        match self.phase {
            Phase::Active => self.draw_active(frame),
            Phase::Results => self.draw_results(frame),
        }
    }

    fn draw_active(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // header
                Constraint::Min(5),    // question + options
                Constraint::Length(3), // footer / timer
            ])
            .split(frame.area());

        // ── Header ───────────────────────────────────────────────────────
        let progress = format!(
            " {} — Question {}/{}",
            self.config.title,
            self.current + 1,
            self.total_questions()
        );
        let header = Paragraph::new(progress)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::BOTTOM));
        frame.render_widget(header, chunks[0]);

        // ── Question + options ───────────────────────────────────────────
        let qi = self.current_question_index();
        let question = &self.config.questions[qi];

        let q_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // question text
                Constraint::Min(1),    // options
            ])
            .split(chunks[1]);

        let q_text = Paragraph::new(format!("  {}", question.text))
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            .wrap(Wrap { trim: false });
        frame.render_widget(q_text, q_area[0]);

        let items: Vec<ListItem> = question
            .options
            .iter()
            .enumerate()
            .map(|(i, opt)| {
                let prefix = if i == self.selected_option {
                    " ▸ "
                } else {
                    "   "
                };
                let style = if i == self.selected_option {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };
                ListItem::new(format!("{prefix}{opt}")).style(style)
            })
            .collect();

        let options_list =
            List::new(items).block(Block::default().borders(Borders::NONE));
        frame.render_widget(options_list, q_area[1]);

        // ── Footer / timer ───────────────────────────────────────────────
        let elapsed = self.quiz_timer.elapsed_secs();
        let mins = (elapsed / 60.0).floor() as u64;
        let secs = (elapsed % 60.0).floor() as u64;
        let footer_text = format!(" ⏱  {:02}:{:02}  │  ↑↓ navigate  │  Enter select  │  Esc quit", mins, secs);
        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP));
        frame.render_widget(footer, chunks[2]);
    }

    fn draw_results(&self, frame: &mut Frame) {
        let result = self.result.as_ref().unwrap();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // title
                Constraint::Length(5),  // summary stats
                Constraint::Min(5),    // per-question review
                Constraint::Length(3), // buttons
                Constraint::Length(3), // footer
            ])
            .split(frame.area());

        // ── Title ────────────────────────────────────────────────────────
        let title = Paragraph::new(format!(" {} — Results", self.config.title))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::BOTTOM));
        frame.render_widget(title, chunks[0]);

        // ── Summary stats ────────────────────────────────────────────────
        let mins = (result.time_secs / 60.0).floor() as u64;
        let secs = (result.time_secs % 60.0).floor() as u64;
        let score_color = if result.accuracy >= 80.0 {
            Color::Green
        } else if result.accuracy >= 50.0 {
            Color::Yellow
        } else {
            Color::Red
        };

        let summary_lines = vec![
            Line::from(vec![
                Span::styled("  Score: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{}/{}", result.correct, result.total),
                    Style::default().fg(score_color).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Accuracy: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{:.1}%", result.accuracy),
                    Style::default().fg(score_color).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Time: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{:02}:{:02}", mins, secs),
                    Style::default().fg(Color::White),
                ),
            ]),
        ];
        let summary = Paragraph::new(summary_lines);
        frame.render_widget(summary, chunks[1]);

        // ── Per-question review ──────────────────────────────────────────
        self.draw_question_review(frame, result, chunks[2]);

        // ── End action buttons ───────────────────────────────────────────
        let button_spans: Vec<Span> = END_BUTTONS
            .iter()
            .enumerate()
            .flat_map(|(i, (label, _))| {
                let style = if i == self.selected_end_button {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                let btn = Span::styled(format!("  {}  ", label), style);
                if i + 1 < END_BUTTONS.len() {
                    vec![btn, Span::raw("  ")]
                } else {
                    vec![btn]
                }
            })
            .collect();
        let buttons = Paragraph::new(Line::from(button_spans))
            .block(Block::default().borders(Borders::TOP));
        frame.render_widget(buttons, chunks[3]);

        // ── Footer ───────────────────────────────────────────────────────
        let footer = Paragraph::new(" ↑↓ scroll review  │  ←→ select action  │  Enter confirm  │  n/h/q shortcuts")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP));
        frame.render_widget(footer, chunks[4]);
    }

    fn draw_question_review(&self, frame: &mut Frame, result: &QuizResult, area: Rect) {
        let items: Vec<ListItem> = result
            .answers
            .iter()
            .enumerate()
            .map(|(i, ans)| {
                let q = &self.config.questions[ans.question_index];
                let icon = if ans.is_correct() { "✓" } else { "✗" };
                let color = if ans.is_correct() {
                    Color::Green
                } else {
                    Color::Red
                };

                let mut text = format!(" {icon}  Q{}: {}", i + 1, q.text);
                if !ans.is_correct() {
                    text.push_str(&format!(
                        "\n      Your answer: {}  │  Correct: {}",
                        q.options[ans.selected], q.options[ans.correct]
                    ));
                }
                ListItem::new(text).style(Style::default().fg(color))
            })
            .collect();

        // Apply scroll offset
        let visible_items: Vec<ListItem> = items
            .into_iter()
            .skip(self.results_scroll)
            .collect();

        let review = List::new(visible_items).block(
            Block::default()
                .title(" Review ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(review, area);
    }
}

/// Run an interactive MCQ quiz in the terminal.
///
/// This takes over the terminal (alternate screen + raw mode) and returns
/// the quiz results when the user finishes or quits, along with the end action
/// chosen by the user on the results screen.
///
/// Returns `None` if the user quits early (Esc/q during active phase).
pub fn run_quiz(config: QuizConfig) -> std::io::Result<Option<(QuizResult, QuizEndAction)>> {
    if config.questions.is_empty() {
        return Ok(Some((QuizResult {
            correct: 0,
            total: 0,
            accuracy: 0.0,
            time_secs: 0.0,
            answers: vec![],
        }, QuizEndAction::Quit)));
    }

    let mut terminal = ratatui::try_init()?;
    let result = run_quiz_loop(&mut terminal, config);
    ratatui::try_restore()?;
    result
}

fn run_quiz_loop(
    terminal: &mut DefaultTerminal,
    config: QuizConfig,
) -> std::io::Result<Option<(QuizResult, QuizEndAction)>> {
    let mut app = QuizApp::new(config);

    loop {
        terminal.draw(|f| app.draw(f))?;

        // Poll with a timeout so the timer updates even when idle.
        if event::poll(Duration::from_millis(200))?
            && let Event::Key(key) = event::read()?
        {
            // Only handle key press events (not release/repeat).
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    match app.phase {
                        Phase::Active => return Ok(None), // early quit
                        Phase::Results => return Ok(app.result.map(|r| (r, QuizEndAction::Quit))),
                    }
                }
                // Shortcut keys on results screen
                KeyCode::Char('n') => {
                    if matches!(app.phase, Phase::Results) {
                        return Ok(app.result.map(|r| (r, QuizEndAction::NewQuiz)));
                    } else {
                        app.handle_key(key.code);
                    }
                }
                KeyCode::Char('h') => {
                    if matches!(app.phase, Phase::Results) {
                        return Ok(app.result.map(|r| (r, QuizEndAction::History)));
                    } else {
                        app.handle_key(key.code);
                    }
                }
                KeyCode::Enter => {
                    if matches!(app.phase, Phase::Results) {
                        let action = END_BUTTONS[app.selected_end_button].1.clone();
                        return Ok(app.result.map(|r| (r, action)));
                    } else {
                        app.handle_key(key.code);
                    }
                }
                other => app.handle_key(other),
            }
        }
    }
}

// NOTE: The renderer is intentionally not unit-tested — it's an interactive
// TUI component that requires a real terminal to function. Coverage is provided
// indirectly through the quiz-engine integration tests.
