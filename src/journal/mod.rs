mod journal;
mod journalctl;
pub mod cli;

pub use journal::Journal;
pub use journal::OutputFormat;
pub use journalctl::tail;
pub use journalctl::summary;

