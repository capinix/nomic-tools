use eyre::Result;
use eyre::WrapErr;
use std::env;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use crate::journal::Journal;
use crate::global::GroupBy;
use std::collections::HashMap;
use chrono::Utc;
use chrono::DateTime;
use log::warn;

// Common function to process journalctl output
fn process_journal_lines<F>(grep_expr: &str, line_processor: F) -> Result<()>
where
    F: Fn(String) -> Result<()>,
{
    // Get the path of the current executable
    let exe_path = env::current_exe().wrap_err("Failed to get the current executable path")?;
    let exe_path_str = exe_path.to_string_lossy();

    // Start the journalctl command
    let mut child = Command::new("journalctl")
        .args(&[
            &format!("_EXE={}", exe_path_str),
            &format!("--grep={}", grep_expr),
            "--output=cat",
            "--no-pager",
            "--follow",
            "--lines=200",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .wrap_err("Failed to start journalctl")?;

    // Read stdout from journalctl
    let reader = BufReader::new(child.stdout.take().unwrap());

    // Process each line using the provided line processor
    for line in reader.lines() {
        let line = line.wrap_err("Failed to read line from journalctl")?;
        line_processor(line)?;
    }

    // Wait for the child process to finish
    let status = child.wait().wrap_err("Failed to wait for journalctl process")?;
    if !status.success() {
        return Err(eyre::eyre!("journalctl process exited with a non-zero status"));
    }

    Ok(())
}

pub fn tail(staked_or_not: Option<bool>) -> Result<()> {
    // Define the grep expression based on the staked_or_not parameter
    let grep_expr = match staked_or_not {
        Some(true) => r#"{[^}]*"staked"[[:space:]]*:[[:space:]]*"✅"[^}]*}"#,
        Some(false) => r#"{[^}]*"staked"[[:space:]]*:[[:space:]]*"❌"[^}]*}"#,
        None => r#"{[^}]*"staked"[[:space:]]*:[[:space:]]*"(✅|❌)"[^}]*}"#,
    };

    // Call the common function with the grep expression and the specific line processing
    process_journal_lines(grep_expr, |line| {
        let journal = Journal::from_json_str(&line)?;
        println!("{}", journal.log());
        Ok(())
    })
}

pub struct DailyTotals {
    pub day: String,
    pub totals: HashMap<String, u64>,
    pub column_widths: [usize; 3],
}

impl DailyTotals {
    pub fn new() -> Self {
        Self {
            day: String::new(), // Start with an empty string
            totals: HashMap::new(),
            column_widths: [5, 10, 10], // Default widths for day, name, and total columns
        }
    }

    pub fn add(&mut self, timestamp: DateTime<Utc>, name: String, quantity: u64) {
        // Convert timestamp to MM-DD format
        let day_str = timestamp.format("%m-%d").to_string();

        // If the day does not match, flush and update to the new day
        if self.day != day_str {
            self.flush();
            self.day = day_str;
        }

        // Add quantity to the existing total for this name, or insert a new one if it doesn't exist
        *self.totals.entry(name).or_insert(0) += quantity;
    }

    fn flush(&mut self) {
        let (width_day, width_name, width_total) = (
            self.column_widths[0],
            self.column_widths[1],
            self.column_widths[2],
        );

        // Collect entries into a vector and sort by name
        let mut sorted_totals: Vec<_> = self.totals.iter().collect();
        sorted_totals.sort_by_key(|(name, _)| name.to_owned()); // Sort by name

        if !sorted_totals.is_empty() { // Only print if there are totals to show
            for (name, total) in sorted_totals {
                println!("{:width_day$} {:width_name$} {:>width_total$}", self.day, name, total);
            }
        }

        // Clear totals after printing
        self.totals.clear();
    }
}

pub fn summary(group_by: GroupBy) -> Result<()> {
    // Define the grep expression for staked status
    let grep_expr = r#"{[^}]*"staked"[[:space:]]*:[[:space:]]*"✅"[^}]*}"#;

    // Call the common function with the grep expression and specific line processing
    process_journal_lines(grep_expr, |line| {
        let journal = Journal::from_json_str(&line)?;
        let timestamp = journal.get::<DateTime<Utc>>("timestamp");

        // Determine the group based on the GroupBy enum
        let group = match group_by {
            GroupBy::Profile => journal.get::<String>("profile"),
            GroupBy::Moniker => journal.get::<String>("moniker"),
        };

        let quantity = journal.get::<u64>("quantity");

        let mut totals = DailyTotals::new();

        match (timestamp, group, quantity) {
            (Some(timestamp), Some(group), Some(quantity)) => {
                totals.add(timestamp, group, quantity);
            }
            _ => {
                warn!("Skipping line due to missing data: {:?}", line);
            }
        }
        Ok(())
    })
}

