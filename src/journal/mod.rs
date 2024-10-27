mod journal;
mod journalctl;

pub use journal::Journal;
pub use journal::OutputFormat;
pub use journalctl::tail;
pub use journalctl::summary;

