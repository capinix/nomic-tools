use chrono::DateTime;
use chrono::Utc;
use crate::functions::format_to_millions;
use crate::functions::TableColumns;
use crate::global::GroupBy;
use crate::journal::Journal;
use eyre::Result;
use eyre::WrapErr;
use log::warn;
use std::collections::HashMap;
use std::env;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use tabled::builder::Builder;
use tabled::settings::{Alignment, Color, Modify, Style, Padding};
use tabled::settings::object::{Columns, Rows};

// Define ANSI color codes for a small palette of contrasting colors
const COLORS: [&str; 4] = ["\x1b[31m", "\x1b[34m", "\x1b[33m", "\x1b[32m"]; // Red, Yellow, Blue, Green
const RESET: &str = "\x1b[0m";

// Common function to process journalctl output
fn process_journal_lines<F>(grep_expr: &str, follow: bool, mut line_processor: F) -> Result<()>
where
    F: FnMut(String) -> Result<()>,
{
    // Get the path of the current executable
    let exe_path = env::current_exe().wrap_err("Failed to get the current executable path")?;
    let exe_path_str = exe_path.to_string_lossy();

    // Start the journalctl command
    let mut cmd = Command::new("journalctl");
    cmd.args(&[
        &format!("_EXE={}", exe_path_str),
        &format!("--grep={}", grep_expr),
        "--no-tail",
        "--no-pager",
        "--output=cat",
    ]);
    if follow {
        cmd.arg("--follow");
    }

    let mut child = cmd
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

pub fn tail(staked_or_not: Option<bool>, follow: bool) -> Result<()> {
    // Define the grep expression based on the staked_or_not parameter
    let grep_expr = match staked_or_not {
        Some(true) => r#"{[^}]*"staked"[[:space:]]*:[[:space:]]*"✅"[^}]*}"#,
        Some(false) => r#"{[^}]*"staked"[[:space:]]*:[[:space:]]*"❌"[^}]*}"#,
        None => r#"{[^}]*"staked"[[:space:]]*:[[:space:]]*"(✅|❌)"[^}]*}"#,
    };

    // Call the common function with the grep expression and the specific line processing
    process_journal_lines(grep_expr, follow, |line| {
        let journal = Journal::from_json_str(&line)?;
        println!("{}", journal.log());
        Ok(())
    })
}

pub struct DailyTotals {
    pub day: String,
    pub totals: HashMap<String, u64>,
    color_index: usize,
}




impl DailyTotals {
    pub fn new() -> Self {
        Self {
            day: String::new(),
            totals: HashMap::new(),
            color_index: 0,
        }
    }

    pub fn add(&mut self, timestamp: DateTime<Utc>, name: String, quantity: u64) {
        // Convert timestamp to MM-DD format
        let day_str = timestamp.format("%m-%d").to_string();

        // If the day does not match, flush and update to the new day
        if self.day != day_str {
            self.flush();
            self.day = day_str.to_string();
        }

        // Add quantity to the existing total for this name, or insert a new one if it doesn't exist
        *self.totals.entry(name).or_insert(0) += quantity;
    }

    fn flush(&mut self) {

        // Collect entries into a vector and sort by name
        let mut sorted_totals: Vec<_> = self.totals.iter().collect();
        sorted_totals.sort_by_key(|(name, _)| name.to_owned()); // Sort by name

        self.color_index = (self.color_index + 1) % COLORS.len(); // Increment and wrap around

        println!("{}", self);

        // Clear totals after printing
        self.totals.clear();
    }

    fn print(&self) -> String {
        let mut rows: Vec<TableColumns> = Vec::new();
        for (name, total) in self.totals.iter() {
            rows.push(TableColumns::new(vec![
                &self.day,
                name,
                &format_to_millions(*total, None),
            ]));
        }
        rows.sort_by_key(|row| row.cell1.clone());

        // Initialize Builder without headers
        let mut builder = Builder::default();
        for row in &rows {
            builder.push_record([
                row.cell0.clone(), 
                row.cell1.clone(), 
                row.cell2.clone(), 
            ]);
        }

        let color = COLORS[self.color_index % COLORS.len()];

        let mut table = builder.build();
        table
            .with(Style::blank())
            .with(Modify::new(Columns::single(0)).with(Padding::new(0,0,0,0)))
            .with(Modify::new(Columns::single(1)).with(Padding::new(0,1,0,0)))
            .with(Modify::new(Columns::single(2)).with(Padding::new(0,0,0,0)))
            .with(Modify::new(Columns::single(2)).with(Alignment::right()))
            .with(Modify::new(Rows::new(0..)).with(Color::new(color, RESET)))
            ;

        table.to_string()
    }
}

impl std::fmt::Display for DailyTotals { // Fully qualified fmt
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.print())
    }
}

pub fn summary(group_by: GroupBy, follow: bool) -> Result<()> {
    // Define the grep expression for staked status
    let grep_expr = r#"{[^}]*"staked"[[:space:]]*:[[:space:]]*"✅"[^}]*}"#;

    let mut totals = DailyTotals::new();

    // Call the common function with the grep expression and specific line processing
    process_journal_lines(grep_expr, follow, |line| {
        let journal = Journal::from_json_str(&line)?;
        let timestamp = journal.get::<DateTime<Utc>>("timestamp");

        // Determine the group based on the GroupBy enum
        let group = match group_by {
            GroupBy::Profile => journal.get::<String>("profile"),
            GroupBy::Moniker => journal.get::<String>("moniker"),
        };

        let quantity = journal.get::<u64>("quantity");

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

