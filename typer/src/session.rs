//! Orchestrates the typing test session.

use crate::diff::calculate_wpm;
use crate::renderer::Renderer;
use crate::sentences::{get_sentence, SentenceLength};
use core::terminal::RawModeGuard;
use core::timer::Timer;
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::io::{self, Write};

/// Configuration for a typing test session.
#[derive(Debug, Clone)]
pub struct TypingConfig {
    /// If provided, uses this specific sentence. Otherwise uses a random one based on length.
    pub sentence: Option<String>,
    /// Target length if random sentence is chosen.
    pub length: SentenceLength,
}

/// Result of a completed typing test.
#[derive(Debug, Clone)]
pub struct TypingResult {
    pub wpm: f64,
    pub accuracy: f64,
    pub time_secs: f64,
    pub sentence_length: usize,
}

pub enum TyperEndAction {
    Repeat(String),
    New,
    History,
    Quit,
}

pub struct TyperSessionResult {
    pub stats: Option<TypingResult>,
    pub action: TyperEndAction,
    pub sentence: String,
}

pub fn run_typing_test(config: TypingConfig) -> io::Result<TyperSessionResult> {
    let mut stdout = io::stdout();
    let renderer = Renderer::new();

    let sentence = config
        .sentence
        .clone()
        .unwrap_or_else(|| get_sentence(config.length).to_string());

    run_single_session(&mut stdout, &renderer, &sentence)
}

pub fn run_end_screen(
    sentence: &str,
    stats: &TypingResult,
) -> io::Result<TyperEndAction> {
    let mut stdout = io::stdout();
    let renderer = Renderer::new();
    let _guard = RawModeGuard::new(true)?;
    show_end_screen(&mut stdout, &renderer, sentence, sentence, stats.wpm, stats.accuracy, stats.time_secs)
}

fn run_single_session(
    stdout: &mut io::Stdout,
    renderer: &Renderer,
    sentence: &str,
) -> io::Result<TyperSessionResult> {
    // Take over the terminal
    let _guard = RawModeGuard::new(true)?;

    let mut typed = String::new();
    let mut timer: Option<Timer> = None;
    let expected_len = sentence.chars().count();
    let mut total_keystrokes = 0;
    let mut total_errors = 0;

    renderer.clear(stdout)?;
    renderer.render(stdout, sentence, &typed, 0.0, 100.0, true)?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Esc => {
                    return Ok(TyperSessionResult {
                        stats: None,
                        action: TyperEndAction::Quit,
                        sentence: sentence.to_string(),
                    });
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(TyperSessionResult {
                        stats: None,
                        action: TyperEndAction::Quit,
                        sentence: sentence.to_string(),
                    });
                }
                KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(TyperSessionResult {
                        stats: None,
                        action: TyperEndAction::Repeat(sentence.to_string()),
                        sentence: sentence.to_string(),
                    });
                }
                KeyCode::Backspace => {
                    if !typed.is_empty() {
                        typed.pop();
                    }
                }
                KeyCode::Char(c) => {
                    if timer.is_none() {
                        timer = Some(Timer::start());
                    }

                    if typed.chars().count() < expected_len {
                        let expected_char = sentence.chars().nth(typed.chars().count()).unwrap();
                        total_keystrokes += 1;
                        if c != expected_char {
                            total_errors += 1;
                        }
                        typed.push(c);
                    }
                }
                _ => {}
            }

            let (wpm, accuracy) = if let Some(t) = &timer {
                let elapsed = t.elapsed_secs();
                let acc = if total_keystrokes > 0 {
                    let a = ((total_keystrokes - total_errors) as f64 / total_keystrokes as f64) * 100.0;
                    a.max(0.0)
                } else {
                    100.0
                };
                let wpm = calculate_wpm(typed.chars().count(), elapsed);
                (wpm, acc)
            } else {
                (0.0, 100.0)
            };

            renderer.clear(stdout)?;
            renderer.render(stdout, sentence, &typed, wpm, accuracy, true)?;

            if typed.chars().count() == expected_len {
                let t = timer.unwrap();
                let time_taken = t.elapsed_secs();
                
                let stats = TypingResult {
                    wpm,
                    accuracy,
                    time_secs: time_taken,
                    sentence_length: expected_len,
                };
                
                let action = show_end_screen(stdout, renderer, sentence, &typed, wpm, accuracy, time_taken)?;
                
                return Ok(TyperSessionResult {
                    stats: Some(stats),
                    action,
                    sentence: sentence.to_string(),
                });
            }
        }
    }
}

fn show_end_screen(
    stdout: &mut io::Stdout,
    renderer: &Renderer,
    sentence: &str,
    typed: &str,
    wpm: f64,
    accuracy: f64,
    time_taken: f64,
) -> io::Result<TyperEndAction> {
    let options = [
        ("Repeat sentence", TyperEndAction::Repeat(sentence.to_string())),
        ("New sentence", TyperEndAction::New),
        ("History", TyperEndAction::History),
        ("Quit", TyperEndAction::Quit),
    ];
    let mut selected = 0;

    loop {
        renderer.clear(stdout)?;
        let end_y = renderer.render(stdout, sentence, typed, wpm, accuracy, false)?;
        let mut y = end_y + 2;

        queue!(
            stdout,
            MoveTo(0, y),
            SetForegroundColor(Color::Cyan),
            Print(format!("Time Taken: {:.1}s", time_taken)),
            ResetColor
        )?;
        
        y += 2;
        queue!(stdout, MoveTo(0, y))?;

        for (i, (label, _)) in options.iter().enumerate() {
            if i == selected {
                queue!(
                    stdout, 
                    SetForegroundColor(Color::Black), 
                    crossterm::style::SetBackgroundColor(Color::White)
                )?;
            }
            queue!(stdout, Print(format!(" [{}]{} ", label.chars().next().unwrap(), &label[1..])))?;
            if i == selected {
                queue!(stdout, crossterm::style::ResetColor)?;
            }
            queue!(stdout, Print("  "))?;
        }
        stdout.flush()?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press { continue; }
            match key.code {
                KeyCode::Left => { selected = selected.saturating_sub(1); }
                KeyCode::Right => { if selected < options.len() - 1 { selected += 1; } }
                KeyCode::Enter => {
                    return Ok(match selected {
                        0 => TyperEndAction::Repeat(sentence.to_string()),
                        1 => TyperEndAction::New,
                        2 => TyperEndAction::History,
                        _ => TyperEndAction::Quit,
                    });
                }
                KeyCode::Char('r') | KeyCode::Char('R') => return Ok(TyperEndAction::Repeat(sentence.to_string())),
                KeyCode::Char('n') | KeyCode::Char('N') => return Ok(TyperEndAction::New),
                KeyCode::Char('h') | KeyCode::Char('H') => return Ok(TyperEndAction::History),
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(TyperEndAction::Quit),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(TyperEndAction::Quit),
                _ => {}
            }
        }
    }
}
