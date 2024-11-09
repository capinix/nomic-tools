use chrono::{DateTime, Local, Utc};
use crate::functions::NumberDisplay;
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
use tabled::settings::{Alignment, Color, Modify, Span, Style, Padding};
use tabled::settings::object::{Cell, Columns, Rows};

// Define ANSI color codes for a small palette of contrasting colors
const COLORS: [&str; 5] = [
    "\x1b[31m",  // Red
    "\x1b[34m",  // Blue
    "\x1b[33m",  // Yellow
    "\x1b[38;2;255;165;0m",  // Orange (RGB: 255, 165, 0)
    "\x1b[32m",  // Green
];
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

pub struct Summary {
    pub period: String,
    pub totals: HashMap<String, u64>,
    color_index: usize,
}




impl Summary {
    pub fn new() -> Self {
        Self {
            period: String::new(),
            totals: HashMap::new(),
            color_index: 0,
        }
    }

    pub fn add_day(&mut self, timestamp: DateTime<Utc>, name: String, quantity: u64) {
        // Convert timestamp to MM-DD format
        let dt_local = timestamp.with_timezone(&Local);
        let day = dt_local.format("%Y-%m-%d %a").to_string();

        // If the day does not match, flush and update to the new day
        if self.period != day {
            self.flush();
            self.period = day.to_string();
        }

        // Add quantity to the existing total for this name, or insert a new one if it doesn't exist
        *self.totals.entry(name).or_insert(0) += quantity;
    }

    pub fn add_week(&mut self, timestamp: DateTime<Utc>, name: String, quantity: u64) {
        // Convert timestamp to MM-DD format
        let dt_local = timestamp.with_timezone(&Local);
        let week = dt_local.format("%G-W%V").to_string();

        // If the day does not match, flush and update to the new day
        if self.period != week {
            self.flush();
            self.period = week.to_string();
        }

        // Add quantity to the existing total for this name, or insert a new one if it doesn't exist
        *self.totals.entry(name).or_insert(0) += quantity;
    }

    pub fn add_month(&mut self, timestamp: DateTime<Utc>, name: String, quantity: u64) {
        // Convert timestamp to MM-DD format
        let dt_local = timestamp.with_timezone(&Local);
        let month = dt_local.format("%Y %B").to_string();

        // If the day does not match, flush and update to the new day
        if self.period != month {
            self.flush();
            self.period = month.to_string();
        }

        // Add quantity to the existing total for this name, or insert a new one if it doesn't exist
        *self.totals.entry(name).or_insert(0) += quantity;
    }


    fn flush(&mut self) {

        // Collect entries into a vector and sort by name
        let mut sorted_totals: Vec<_> = self.totals.iter().collect();
        sorted_totals.sort_by_key(|(name, _)| name.to_owned()); // Sort by name

        self.color_index = (self.color_index + 1) % COLORS.len(); // Increment and wrap around

        if self.totals.len() > 0 {
            println!("\n{}", self);
        };

        // Clear totals after printing
        self.totals.clear();
    }

    fn print(&self) -> String {
        let mut rows: Vec<TableColumns> = Vec::new();
        for (name, total) in self.totals.iter() {
            rows.push(TableColumns::new(vec![
                name,
                &NumberDisplay::new(*total).decimal_places(2).format(),
            ]));
        }

        if rows.len() == 0 {
            return String::new()
        };

        // Sort the rows by the first column.
        rows.sort_by_key(|row| row.cell0.clone());

        // Insert the day row at the beginning.
        rows.insert(0, TableColumns::new(vec![&self.period]));

        // Calculate the grand total.
        let grand_total = self.totals.values().sum();

        // Add the totals row at the end.
        rows.push(TableColumns::new(vec![
            "",
            &NumberDisplay::new(grand_total).decimal_places(2).format(),
        ]));

        // Initialize Builder without headers
        let mut builder = Builder::default();
        for row in &rows {
            builder.push_record([
                row.cell0.clone(), 
                row.cell1.clone(), 
            ]);
        }

        let color = COLORS[self.color_index % COLORS.len()];
        let color_q = COLORS[(self.color_index + 2) % COLORS.len()];

        let mut table = builder.build();
        table
            .with(Style::blank())
            .with(Modify::new(Columns::single(0)).with(Padding::new(0,0,0,0)))
            .with(Modify::new(Columns::single(1)).with(Padding::new(0,0,0,0)))
            .with(Modify::new(Columns::single(1)).with(Alignment::right()))
            .with(Modify::new(Columns::single(0)).with(Color::new(color, RESET)))
            .with(Modify::new(Columns::single(1)).with(Color::new(color, RESET)))
            .with(Modify::new(Cell::new(0, 0)).with(Span::column(2)).with(Alignment::left()))
            //.with(Modify::new(Rows::single(0)).with(Border::new().set_bottom('-')))
            .with(Modify::new(Rows::single(0)).with(Color::new(color_q, RESET)))
            .with(Modify::new(Rows::single(rows.len() - 1)).with(Color::new(color_q, RESET)))
            //.with(Modify::new(Cell::new(rows.len() - 1, 1)).with(Border::new().set_top('-')))
            ;

        table.to_string()
    }
}

impl std::fmt::Display for Summary { // Fully qualified fmt
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.print())
    }
}

pub fn summary(group_by: GroupBy, follow: bool) -> Result<()> {
    // Define the grep expression for staked status
    let grep_expr = r#"{[^}]*"staked"[[:space:]]*:[[:space:]]*"✅"[^}]*}"#;

    let mut daily = Summary::new();
    let mut weekly = Summary::new();
    let mut monthly = Summary::new();

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
                daily.add_day(timestamp, group.clone(), quantity);
                weekly.add_week(timestamp, group.clone(), quantity);
                monthly.add_month(timestamp, group.clone(), quantity);
            }
            _ => {
                warn!("Skipping line due to missing data: {:?}", line);
            }
        }
        Ok(())
    })
}

