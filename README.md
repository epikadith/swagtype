# swagtype 

`swagtype` is an all-in-one terminal suite for practicing typing, doing math exercises, testing your science knowledge, and running custom quizzes—all completely offline, right in your terminal. 

It keeps a persistent local history of your performance so you can track your progress over time!

## Features

- **Typing Test (`typer`)**: Test your typing speed in raw mode with real-time character-by-character WPM and accuracy tracking. You can type randomly generated sentences or supply your own. Features an interactive menu to quickly retry or view history.
- **Math Quiz (`mathwiz`)**: Procedurally generated math questions. Configurable by difficulty and category (Addition, Subtraction, Multiplication, Division, Algebra, or Mixed).
- **Science Quiz (`sciencewiz`)**: Curated quizzes across various science domains like Physics, Chemistry, Biology, and Earth Science.
- **Custom Quizzes (`customwiz`)**: Load your own custom JSON quizzes into the interactive TUI engine.
- **History Tracker (`history`)**: Automatically records every typing session and quiz score into a local SQLite database and lets you view your performance over time.

## Installation

### Downloading Pre-compiled Executables

Pre-compiled binaries are built automatically via GitHub Actions for Linux, macOS, and Windows.
1. Go to the **Releases** page on the GitHub repository.
2. Download the binary that corresponds to your operating system (`swagtype-linux-x86_64`, `swagtype-macos-x86_64`, `swagtype-macos-aarch64`, or `swagtype-windows-x86_64.exe`).
3. Make the file executable (e.g., `chmod +x swagtype-linux-x86_64`).
4. Move the executable to a directory in your system's `PATH` so it can be run from anywhere:
   - **Linux/macOS**: Move it to `/usr/local/bin/` or `~/.local/bin/` (e.g., `mv swagtype-linux-x86_64 ~/.local/bin/swagtype`).
   - **Windows**: Place `swagtype.exe` in a dedicated folder (e.g., `C:\Program Files\swagtype`) and add that folder to your system's "Environment Variables -> PATH".
### Compiling from Source

If you have Rust and Cargo installed, you can easily compile `swagtype` from source.

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/swagtype.git
   cd swagtype
   ```
2. Build the project in release mode:
   ```bash
   cargo build --release
   ```
3. Run the executable located at `target/release/swagtype`.

Alternatively, you can run commands directly during development using `cargo run`.

## Data Storage

`swagtype` keeps a persistent local history of your performance in a local SQLite database (`stats.db`). This file is safely stored in your system's standard application data directory, so you can update or move the executable without losing your data:

- **Linux**: `~/.local/share/swagtype/stats.db`
- **macOS**: `~/Library/Application Support/swagtype/stats.db`
- **Windows**: `C:\Users\<Username>\AppData\Roaming\swagtype\data\stats.db`

## Usage & Commands

Run `swagtype <COMMAND> --help` to see detailed information for each tool.

### Typing Test
Test your WPM (Words Per Minute) and accuracy.
```bash
# Run a typing test with a random medium-length sentence
swagtype typer

# Specify the length of the random sentence (short, medium, long)
swagtype typer --length short

# Use your own custom sentence
swagtype typer --sentence "The quick brown fox jumps over the lazy dog."
```

### Math Quiz
Answer procedurally generated math problems.
```bash
# Start a mixed math quiz with medium difficulty (default: 5 questions)
swagtype mathwiz

# Start an algebra quiz with hard difficulty, for 10 questions
swagtype mathwiz --category algebra --difficulty hard -n 10
```
**Categories:** `add`, `sub`, `mul`, `div`, `algebra`, `mixed`
**Difficulties:** `easy`, `medium`, `hard`

### Science Quiz
Test your science knowledge.
```bash
# Start a general science quiz (default: 5 questions)
swagtype sciencewiz

# Start a physics quiz for 10 questions
swagtype sciencewiz --category physics -n 10
```
**Categories:** `physics`, `chemistry`, `biology`, `earth`, `general`

### Custom Quiz
Run a custom multiple-choice quiz from a JSON file.
```bash
swagtype customwiz --file /path/to/my_quiz.json
```

To create your own custom quizzes, your JSON file must follow this exact format:
```json
{
  "title": "My Awesome Quiz",
  "shuffle": true,
  "questions": [
    {
      "text": "What is the capital of France?",
      "options": ["Berlin", "London", "Paris", "Madrid"],
      "correct_index": 2
    },
    {
      "text": "Which planet is known as the Red Planet?",
      "options": ["Earth", "Mars", "Jupiter", "Venus"],
      "correct_index": 1
    }
  ]
}
```

**Generate Quizzes with AI**
You can easily generate compatible quizzes using ChatGPT, Claude, or Gemini by copying and pasting the following prompt alongside your topic or document:

```text
Please generate a multiple-choice quiz about [INSERT TOPIC OR PASTE DOCUMENT HERE]. 
Output the quiz strictly as a JSON object matching the following schema. Do not include markdown code blocks or any other text, just the raw JSON:

{
  "title": "<Quiz Title>",
  "shuffle": true,
  "questions": [
    {
      "text": "<Question Text>",
      "options": ["<Option 1>", "<Option 2>", "<Option 3>", "<Option 4>"],
      "correct_index": <0-based index of the correct option>
    }
  ]
}

Ensure there are exactly 4 options for each question.
```

### History
View your past scores and typing speeds.
```bash
# View your history
swagtype history

# Clear all history from the device
swagtype history --clear
```

## Architecture

`swagtype` is designed as a Cargo workspace with multiple distinct crates:
- `core`: Shared infrastructure, raw terminal handling, timers, and the `rusqlite` database.
- `quiz-engine`: The shared interactive `ratatui` TUI used by all quiz subtools.
- `typer`, `mathwiz`, `sciencewiz`, `customwiz`: Independent business logic for each subtool.
- `cli`: The main `clap`-based binary entrypoint that wires everything together.
