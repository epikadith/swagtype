# swagtype

A Rust CLI suite of terminal-native typing and quiz games, distributed as a single
binary with optional feature-gated subtools.

## Concept

`swagtype` is a multi-tool CLI: one binary, multiple subcommands, each a self-contained
"game." Users can compile in only the subtools they want. Shared logic (timer, stats,
terminal handling, quiz navigation) lives in common crates so each subtool stays thin.

## Subtools

- **typer** — Typing test. Given a sentence/paragraph (built-in or user-supplied),
  measures time, WPM, and accuracy. Highlights mistakes live in red as the user types.
  Configurable sequence length. QOL features expected: live WPM, backspace/correction
  handling, restart shortcut, history of past runs.
- **mathwiz** — Math MCQs (multiplication tables, algebra, arithmetic, etc.), mostly
  procedurally generated. Arrow-key navigation between options, Enter to select. Timer
  + accuracy metrics at the end. Future: text-input questions, graph-based questions.
- **sciencewiz** — Same engine as mathwiz, but science questions. Content is curated/
  bundled rather than procedurally generated (harder to generate proceduraly than math).
- **customwiz** — Users supply their own MCQ questions via a simple, LLM-friendly text
  format (documented separately, versioned so future question types don't break old
  files). Tool parses a user's `.txt` file and runs it through the shared quiz engine.
- **other** — Not yet decided. Candidates: reaction/reflex tester, sequence/memory
  game, vim-motions trainer, morse code trainer. Anything timed + scored + terminal-
  native fits the pattern.

## Architecture

Cargo workspace, resolver v3. `cli` is the entrypoint binary; each subtool is an
optional dependency gated behind a matching Cargo feature, so users can
`cargo build --release --no-default-features --features mathwiz` for a slim binary.

```
swagtype/
├── Cargo.toml              # [workspace], resolver = "3"
├── cli/                    # binary entrypoint, subcommand dispatch via clap
├── core/                   # terminal setup/teardown guard, timer, stats storage,
│                            # config dir handling (via `directories` crate)
├── quiz-engine/             # shared: MCQ rendering, arrow-key nav, scoring —
│                            # used by mathwiz, sciencewiz, customwiz
├── typer/                  # typing test logic + live diff rendering
├── mathwiz/                 # procedural math question generators
├── sciencewiz/               # curated/bundled science question bank
└── customwiz/                # parser for user-supplied question format
```

Key design decisions:
- Keep pure logic (scoring, diffing, parsing) separate from I/O/rendering for
  testability — interactive TTY code is hard to unit test directly.
- `ratatui` (on top of `crossterm`) for the MCQ arrow-key UI rather than hand-rolled
  ANSI; `crossterm`/`colored`/`owo-colors` territory for the typer's red-highlight
  mistake rendering.
- Terminal raw-mode state must be restored via a `Drop`-guard struct so a panic or
  Ctrl+C never leaves the user's terminal broken.
- Respect `NO_COLOR` env var and detect non-TTY output (piping) before emitting ANSI.
- Persistent local stats (JSON or `rusqlite`) from day one — retrofitting history
  tracking later is painful and users expect progress tracking in typing/quiz tools.
- `customwiz` format: flat, line-based, LLM-generatable, versioned via a header so
  future question types (text input, graph-click) don't break existing files.

## Dependencies (indicative)

`clap` (derive), `crossterm`, `ratatui`, `serde`, `directories`, `colored` or
`owo-colors`. `crossterm`/`ratatui` are pre-1.0 — review changelogs on version bumps
rather than blindly auto-updating.

## Build/release

- `cargo build --release` per target; GitHub Actions matrix build across OS/arch
  (Linux gnu/musl, macOS x86_64/aarch64, Windows msvc) on tag push, using `cross` or
  `cargo-zigbuild` for the harder cross targets, uploading binaries to GitHub Releases.
- `Cargo.lock` is committed (this is a binary, not a library); CI uses
  `cargo build --locked`.
- MSRV pinned per-crate via `rust-version`; CI has a dedicated MSRV check job.