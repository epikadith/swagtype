pub mod diff;
pub mod renderer;
pub mod sentences;
pub mod session;

pub use diff::{compute_diff, CharStatus, DiffResult};
pub use session::{run_typing_test, TypingConfig, TypingResult};
