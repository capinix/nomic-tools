use clap::{Parser, Subcommand};
use eyre::Result;
use crate::global::CONFIG;
use crate::global::GroupBy;

/// Defines the CLI structure for the `validators` command.
#[derive(Parser)]
#[command(name = "GlobalConfig", about = "Global Configuration Settings")]
pub struct Cli {
    /// Subcommands for the validators command
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Journalctl {
        #[command(subcommand)]
        command: Journalctl,
    },
}

#[derive(Debug, Subcommand)]
pub enum Journalctl {
    #[command(
        about = "Configure journalctl column widths",
        visible_alias = "w", aliases = ["wi", "wid", "widt", "width"],
    )]
    Widths {
        #[arg(required = false)]
        column: Option<usize>,

        #[arg(required = false)]
        width: Option<usize>,
    },
    #[command(
        about = "journalctl summaries",
        visible_alias = "s", aliases = ["su", "sum", "summ", "summa", "summar"],
    )]
    Summary {
        #[command(subcommand)]
        command: Option<JournalctlSummary>,
    },
}

#[derive(Debug, Subcommand)]
pub enum JournalctlSummary {
    Profile {
        #[command(subcommand)]
        command: Option<JournalctlSummaryProfile>,
    },
    Moniker {
        #[command(subcommand)]
        command: Option<JournalctlSummaryMoniker>,
    },
}

#[derive(Debug, Subcommand)]
pub enum JournalctlSummaryProfile {
    #[command(
        about = "Configure journalctl column widths for profile summary",
        visible_alias = "w", aliases = ["wi", "wid", "widt", "width"],
    )]
    Widths {
        #[arg(required = false)]
        column: Option<usize>,

        #[arg(required = false)]
        width: Option<usize>,
    },
}

#[derive(Debug, Subcommand)]
pub enum JournalctlSummaryMoniker {
    #[command(
        about = "Configure journalctl column widths for moniker summary",
        visible_alias = "w", aliases = ["wi", "wid", "widt", "width"],
    )]
    Widths {
        #[arg(required = false)]
        column: Option<usize>,

        #[arg(required = false)]
        width: Option<usize>,
    },
}

impl Cli {
    // Method to run the CLI commands
    pub fn run(&self) -> Result<()> {
        match &self.command {
            Command::Journalctl { command } => {
                let mut config = CONFIG.clone();
                match command {
                    Journalctl::Widths { column, width } => {
                        match (column, width) {
                            (None, None) => {
                                println!("{:?}", config.journalctl.tail.column_widths);
                            }
                            (Some(col), None) => {
                                if *col <= config.journalctl.tail.column_widths.len() {
                                    println!("{:?}", config.journalctl.tail.column_widths[*col - 1]);
                                } else {
                                    eprintln!("Column index out of bounds: {}", col);
                                }
                            }
                            (Some(col), Some(w)) => {
                                if *col <= config.journalctl.tail.column_widths.len() {
                                    config.set_journalctl_tail_column_width(*col - 1, *w)?;
                                    println!("{:?}", config.journalctl.tail.column_widths);
                                    config.save()?;
                                } else {
                                    eprintln!("Column index out of bounds: {}", col);
                                }
                            }
                            _ => {
                                    eprintln!("Invalid input: Width provided without a column.");
                            }
                        }
                        Ok(())
                    }
                    Journalctl::Summary { command } => match command {
                        Some(JournalctlSummary::Profile { command }) => match command {
                            Some(JournalctlSummaryProfile::Widths { column, width }) => {
                                match (column, width) {
                                    (None, None) => {
                                        println!("{:?}", config.journalctl.summary.profile.column_widths);
                                    }
                                    (Some(col), None) => {
                                        if *col <= config.journalctl.summary.profile.column_widths.len() {
                                            println!("{:?}", config.journalctl.summary.profile.column_widths[*col - 1]);
                                        } else {
                                            eprintln!("Column index out of bounds: {}", col);
                                        }
                                    }
                                    (Some(col), Some(w)) => {
                                        if *col <= config.journalctl.summary.profile.column_widths.len() {
                                            config.set_journalctl_summary_column_width(GroupBy::Profile, *col - 1, *w)?;
                                            println!("{:?}", config.journalctl.summary.profile.column_widths);
                                            config.save()?;
                                        } else {
                                            eprintln!("Column index out of bounds: {}", col);
                                        }
                                    }
                                    _ => {
                                            eprintln!("Invalid input: Width provided without a column.");
                                    }
                                }
                                Ok(())
                            }
                            None => Ok(()),
                        },
                        Some(JournalctlSummary::Moniker { command }) => match command {
                            Some(JournalctlSummaryMoniker::Widths { column, width }) => {
                                match (column, width) {
                                    (None, None) => {
                                        println!("{:?}", config.journalctl.summary.moniker.column_widths);
                                    }
                                    (Some(col), None) => {
                                        if *col <= config.journalctl.summary.moniker.column_widths.len() {
                                            println!("{:?}", config.journalctl.summary.moniker.column_widths[*col - 1]);
                                        } else {
                                            eprintln!("Column index out of bounds: {}", col);
                                        }
                                    }
                                    (Some(col), Some(w)) => {
                                        if *col <= config.journalctl.summary.moniker.column_widths.len() {
                                            config.set_journalctl_summary_column_width(GroupBy::Moniker, *col - 1, *w)?;
                                            println!("{:?}", config.journalctl.summary.moniker.column_widths);
                                            config.save()?;
                                        } else {
                                            eprintln!("Column index out of bounds: {}", col);
                                        }
                                    }
                                    _ => {
                                            eprintln!("Invalid input: Width provided without a column.");
                                    }
                                }
                                Ok(())
                            }
                            None => Ok(()),
                        },
                        None => Ok(()),
                    },
                }
            }
        }
    }
}
