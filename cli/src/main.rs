use clap::{Parser, Subcommand};
use comfy_table::Table;
use swagtype_core::config::ensure_data_dir;
use swagtype_core::stats::StatsDb;
use std::path::PathBuf;
use std::io::{self, Write};

use customwiz::run_customwiz;
use mathwiz::{MathCategory, MathDifficulty, run_mathwiz};
use sciencewiz::{run_sciencewiz, ScienceCategory};
use quiz_engine::QuizEndAction;
use typer::TypingConfig;

#[derive(Parser)]
#[command(name = "swagtype", version, about = "A terminal suite for typing and learning", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// View your past run history
    History {
        #[arg(short, long, help = "Clear all history from the device")]
        clear: bool,
    },
    /// Run a typing test
    Typer {
        #[arg(short, long, help = "Optional custom sentence to type")]
        sentence: Option<String>,
        #[arg(short, long, default_value = "medium", help = "Length of random sentence (short, medium, long)")]
        length: typer::sentences::SentenceLength,
    },
    /// Run a math quiz
    Mathwiz {
        #[arg(short, long, default_value = "mixed", help = "Category (add, sub, mul, div, algebra, mixed)")]
        category: MathCategory,
        #[arg(short, long, default_value = "medium", help = "Difficulty (easy, medium, hard)")]
        difficulty: MathDifficulty,
        #[arg(short = 'n', long, default_value = "5", help = "Number of questions")]
        count: usize,
    },
    /// Run a science quiz
    Sciencewiz {
        #[arg(short, long, help = "Category (physics, chemistry, biology, earth, general)")]
        category: Option<ScienceCategory>,
        #[arg(short = 'n', long, default_value = "5", help = "Number of questions")]
        count: usize,
    },
    /// Run a custom quiz from a JSON file
    Customwiz {
        #[arg(help = "Path to the custom quiz JSON file")]
        file: PathBuf,
    },
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::History { clear } => {
            if clear {
                clear_history();
            } else {
                print_history();
            }
            Ok(())
        }
        Commands::Typer { sentence, length } => {
            let mut current_sentence = sentence;
            loop {
                let config = TypingConfig { sentence: current_sentence.clone(), length };
                let result = typer::run_typing_test(config)?;
                
                let mut current_action = result.action;
                let current_stats = result.stats;
                let last_sentence = result.sentence;

                if let Some(r) = &current_stats {
                    let run = swagtype_core::stats::QuizRun {
                        id: None,
                        subtool: "typer".to_string(),
                        score: r.wpm.round() as usize,
                        total: r.sentence_length,
                        accuracy: r.accuracy,
                        time_secs: r.time_secs,
                        category: None,
                        timestamp: chrono::Utc::now(),
                    };
                    if let Ok(dir) = ensure_data_dir() 
                        && let Ok(db) = StatsDb::open(&dir.join("stats.db")) 
                    {
                        let _ = db.record_quiz_run(&run);
                    }
                }

                loop {
                    match current_action {
                        typer::session::TyperEndAction::Quit => return Ok(()),
                        typer::session::TyperEndAction::History => {
                            print_history();
                            println!("\nPress Enter to return...");
                            let mut buf = String::new();
                            let _ = std::io::stdin().read_line(&mut buf);
                            
                            if let Some(stats) = &current_stats {
                                current_action = typer::session::run_end_screen(&last_sentence, stats)?;
                            } else {
                                break;
                            }
                        }
                        typer::session::TyperEndAction::New => {
                            current_sentence = None;
                            break;
                        }
                        typer::session::TyperEndAction::Repeat(s) => {
                            current_sentence = Some(s);
                            break;
                        }
                    }
                }
            }
        }
        Commands::Mathwiz { category, difficulty, count } => {
            loop {
                let action = run_mathwiz(mathwiz::MathwizConfig { category, difficulty, count })?;
                match action {
                    Some(QuizEndAction::History) => {
                        print_history();
                        if post_history_menu()? { continue; } else { break; }
                    }
                    Some(QuizEndAction::NewQuiz) => continue,
                    _ => break,
                }
            }
            Ok(())
        }
        Commands::Sciencewiz { category, count } => {
            loop {
                let action = run_sciencewiz(sciencewiz::SciencewizConfig { category, count })?;
                match action {
                    Some(QuizEndAction::History) => {
                        print_history();
                        if post_history_menu()? { continue; } else { break; }
                    }
                    Some(QuizEndAction::NewQuiz) => continue,
                    _ => break,
                }
            }
            Ok(())
        }
        Commands::Customwiz { file } => {
            let mut current_file = file;
            loop {
                let action = run_customwiz(customwiz::CustomwizConfig { file_path: current_file.clone() })?;
                match action {
                    Some(QuizEndAction::History) => {
                        print_history();
                        if post_history_menu()? { continue; } else { break; }
                    }
                    Some(QuizEndAction::NewQuiz) => {
                        match pick_json_file() {
                            Some(path) => { current_file = path; }
                            None => break,
                        }
                    }
                    _ => break,
                }
            }
            Ok(())
        }
    }
}

fn clear_history() {
    if let Ok(dir) = ensure_data_dir() {
        let db_path = dir.join("stats.db");
        if let Ok(db) = StatsDb::open(&db_path) {
            if db.clear_history().is_ok() {
                println!("History successfully cleared.");
            } else {
                eprintln!("Failed to clear history from database.");
            }
        } else {
            eprintln!("Failed to open stats database.");
        }
    }
}

fn print_history() {
    if let Ok(dir) = ensure_data_dir() {
        let db_path = dir.join("stats.db");
        if let Ok(db) = StatsDb::open(&db_path) {
            if let Ok(runs) = db.get_quiz_history(None, 20) {
                if runs.is_empty() {
                    println!("No history found.");
                    return;
                }
                
                let mut table = Table::new();
                table.set_header(vec!["Time", "Subtool", "Category", "Score", "Total", "Accuracy", "Time (s)"]);
                
                for run in runs {
                    let cat_str = run.category.unwrap_or_else(|| "-".to_string());
                    table.add_row(vec![
                        run.timestamp.format("%Y-%m-%d %H:%M").to_string(),
                        run.subtool,
                        cat_str,
                        run.score.to_string(),
                        run.total.to_string(),
                        format!("{:.1}%", run.accuracy),
                        format!("{:.1}s", run.time_secs),
                    ]);
                }
                println!("{}", table);
            } else {
                eprintln!("Failed to read history from database.");
            }
        } else {
            eprintln!("Failed to open stats database.");
        }
    }
}

/// After viewing history, ask the user if they want a new quiz or to quit.
/// Returns true for new quiz, false for quit.
fn post_history_menu() -> io::Result<bool> {
    use crossterm::{
        cursor::MoveTo,
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
        terminal::{disable_raw_mode, enable_raw_mode},
    };

    let mut stdout = io::stdout();
    let mut selected = 0usize;
    let options = ["New Quiz", "Quit"];

    let _ = enable_raw_mode();

    loop {
        // Render the menu on the line after the history table
        let _ = execute!(stdout, Print("\r\n"));
        for (i, label) in options.iter().enumerate() {
            if i == selected {
                let _ = execute!(
                    stdout,
                    SetBackgroundColor(Color::White),
                    SetForegroundColor(Color::Black),
                    Print(format!("  {}  ", label)),
                    ResetColor
                );
            } else {
                let _ = execute!(
                    stdout,
                    SetForegroundColor(Color::DarkGrey),
                    Print(format!("  {}  ", label)),
                    ResetColor
                );
            }
            let _ = execute!(stdout, Print("  "));
        }
        let _ = stdout.flush();

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press { continue; }
            match key.code {
                KeyCode::Left => { selected = selected.saturating_sub(1); }
                KeyCode::Right => { if selected + 1 < options.len() { selected += 1; } }
                KeyCode::Enter => {
                    let _ = disable_raw_mode();
                    println!();
                    return Ok(selected == 0); // 0 = New Quiz
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    let _ = disable_raw_mode();
                    println!();
                    return Ok(true);
                }
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    let _ = disable_raw_mode();
                    println!();
                    return Ok(false);
                }
                _ => {}
            }
            // Erase the menu line before re-drawing
            let _ = execute!(stdout, MoveTo(0, crossterm::cursor::position()?.1), crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine));
        }
    }
}

/// Show an interactive picker for JSON files in the current directory.
/// Returns the selected path, or None if the user presses Esc/q.
fn pick_json_file() -> Option<PathBuf> {
    use crossterm::{
        cursor::MoveTo,
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
        terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
    };

    // Collect JSON files from the current directory (non-recursive)
    let mut json_files: Vec<PathBuf> = std::fs::read_dir(".")
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map(|ext| ext == "json").unwrap_or(false))
                .collect()
        })
        .unwrap_or_default();
    json_files.sort();

    if json_files.is_empty() {
        println!("No JSON files found in the current directory.");
        return None;
    }

    let mut stdout = io::stdout();
    let _ = enable_raw_mode();
    let _ = execute!(stdout, Clear(ClearType::All), MoveTo(0, 0));

    let mut selected = 0usize;

    loop {
        let _ = execute!(stdout, Clear(ClearType::All), MoveTo(0, 0));
        let _ = execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print("Select a quiz file (↑↓ navigate, Enter select, Esc cancel):\r\n\n"),
            ResetColor
        );

        for (i, path) in json_files.iter().enumerate() {
            let name = path.display().to_string();
            let _ = execute!(stdout, MoveTo(2, (i + 2) as u16));
            if i == selected {
                let _ = execute!(
                    stdout,
                    SetBackgroundColor(Color::White),
                    SetForegroundColor(Color::Black),
                    Print(format!(" {} ", name)),
                    ResetColor
                );
            } else {
                let _ = execute!(stdout, Print(format!(" {} ", name)));
            }
        }
        let _ = stdout.flush();

        if let Ok(Event::Key(key)) = event::read() {
            if key.kind != KeyEventKind::Press { continue; }
            match key.code {
                KeyCode::Up => { selected = selected.saturating_sub(1); }
                KeyCode::Down => { if selected + 1 < json_files.len() { selected += 1; } }
                KeyCode::Enter => {
                    let _ = disable_raw_mode();
                    let _ = execute!(stdout, Clear(ClearType::All), MoveTo(0, 0));
                    return Some(json_files.swap_remove(selected));
                }
                KeyCode::Esc | KeyCode::Char('q') => break,
                _ => {}
            }
        }
    }

    let _ = disable_raw_mode();
    let _ = execute!(stdout, Clear(ClearType::All), MoveTo(0, 0));
    None
}
