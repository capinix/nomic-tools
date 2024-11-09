
use chrono::{Utc, DateTime, Local};
use crate::global::CONFIG;
use crate::validators::ValidatorCollection;
use eyre::{eyre, Result};
use indexmap::IndexMap;
use once_cell::sync::OnceCell;
use std::fmt;
use std::path::Path;
use std::process::Command;
use tabled::builder::Builder;
use tabled::settings::{Alignment, Border, Color, Modify, Span, Style, object::{Cell, Columns, Rows}};
use crate::functions::TableColumns;
use crate::functions::NumberDisplay;
use log::warn;

#[derive(Clone, Debug)]
pub struct Delegation {
    pub staked: u64,
    pub liquid: u64,
    pub nbtc: u64,
}

impl Delegation {
    /// Creates a new Delegation instance.
    pub fn new(staked: u64, liquid: u64, nbtc: u64) -> Self {
        Delegation { staked, liquid, nbtc }
    }
}
#[derive(Clone)]
pub struct Delegations {
    pub address:     String,
    pub timestamp:   DateTime<Utc>,
    pub delegations: IndexMap<String, Delegation>,
    pub total:       OnceCell<Delegation>,
    pub validators:  OnceCell<ValidatorCollection>,
}

impl fmt::Debug for Delegations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Create a formatted string for the delegations without the total field
        // Create a formatted string for the delegations without the total field and the word "delegations"
        f.debug_map()
            .entries(self.delegations.iter().map(|(k, v)| (k, v)))
            .finish()
    }
}

impl Delegations {

    /// Creates a new Delegations instance.
    pub fn new<S: AsRef<str>>(
        address: S,
        timestamp: Option<DateTime<Utc>>,
        validators: Option<ValidatorCollection>,
    ) -> Self {
        let address: String = address.as_ref().to_string();
        Delegations {
            address,
            timestamp:   timestamp.unwrap_or(Utc::now()),
            delegations: IndexMap::new(),
            total:       OnceCell::new(),
            validators:  ValidatorCollection::initialize_oncecell(validators),
        }
    }

    /// Adds a delegation to the collection.
    pub fn add_delegation<S: AsRef<str>>(&mut self, id: S, delegation: Delegation) {
        let id: String = id.as_ref().to_string();
        self.delegations.insert(id, delegation);
    }

    /// Retrieves the total delegation, initializing it if necessary.
    pub fn total(&self) -> &Delegation {
        self.total.get_or_init(|| {
            let mut total_staked = 0;
            let mut total_liquid = 0;
            let mut total_nbtc = 0;

            for delegation in self.delegations.values() {
                total_staked += delegation.staked;
                total_liquid += delegation.liquid;
                total_nbtc   += delegation.nbtc;
            }

            Delegation::new(total_staked, total_liquid, total_nbtc)
        })
    }

    /// Fetches the delegations from the command output and returns a new Delegations instance.
    pub fn fetch<P: AsRef<Path>, S: AsRef<str>>(address: S, home: Option<P>) -> Result<Self> {

        let timestamp = Some(Utc::now());

        // Create and configure the Command
        let mut cmd = Command::new(CONFIG.nomic()?); // Replace with the actual command string

        // Set environment variables
        if let Some(ref version) = CONFIG.nomic_legacy_version {
            cmd.env("NOMIC_LEGACY_VERSION", version);
        }

        cmd.arg("delegations");

        if let Some(home_path) = home {
            if let Some(home_str) = home_path.as_ref().to_str() {
                cmd.env("HOME", home_str);
            }
        }

        // Execute the command and collect the output
        let output = cmd.output().map_err(|e| eyre!("Failed to execute command: {}", e))?;

        // Check if the command was successful
        if !output.status.success() {
            let error_msg = format!(
                "Command `{}` failed with output: {:?}",
                CONFIG.nomic()?, // Replace with the actual command string
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(eyre!(error_msg));
        }

        // Convert the output to a string and split it into lines
        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();

        let address: &str = address.as_ref();
        let mut delegations = Delegations::new(address, timestamp, None);

        // Iterate over the lines starting with "- nomic"
        for line in lines.iter().filter(|line| line.trim().starts_with("- nomic")) {
            let line = line.trim_start_matches('-').trim(); // Remove the `-` marker

            if let Some((address, rest)) = line.split_once(':') {
                let rest = rest.trim();
                let parts: Vec<&str> = rest.split(',').collect();

                // Ensure we have the right number of parts
                if parts.len() == 3 {
                    // Parse staked value
                    let staked = parts[0]
                        .split('=')
                        .nth(1) // Get the value after '='
                        .ok_or_else(|| eyre!("Failed to find staked value in: {}", parts[0]))?
                        .trim()
                        .replace(" NOM", "")
                        .parse::<u64>()?;

                    // Parse liquid value
                    let liquid = parts[1]
                        .split('=')
                        .nth(1) // Get the value after '='
                        .ok_or_else(|| eyre!("Failed to find liquid value in: {}", parts[1]))?
                        .trim()
                        .replace(" NOM", "")
                        .parse::<u64>()?;

                    // Parse nbtc value
                    let nbtc = parts[2]
                        .trim() // Trim spaces
                        .replace(" NBTC", "") // Remove the " NBTC" suffix
                        .parse::<u64>()?; // Parse to u64

                    // Create a new Delegation instance
                    let delegation = Delegation::new(staked, liquid, nbtc);
                    delegations.add_delegation(address.trim().to_string(), delegation);
                } else {
                    println!("Unexpected parts length: {}", parts.len());
                }
            } else {
                println!("Could not split line: {}", line);
            }
        }

        // Return the delegations instance wrapped in a Result
        Ok(delegations)
    }

    pub fn validators(&self) -> eyre::Result<&ValidatorCollection> {
        self.validators.get_or_try_init(|| {
            ValidatorCollection::fetch()
        })
    }

    pub fn find<S: AsRef<str>>(&self, address: S) -> Result<&Delegation> {
        let address: &str = address.as_ref();
        self.delegations.get(address)
            .ok_or_else(|| eyre!("Delegation not found for address: {}", address))
    }

    pub fn moniker<S: AsRef<str>>(&self, address: S) -> Result<String> {
        let address: &str = address.as_ref();
        Ok(self.validators()?.validator(address)?.moniker().to_string())
    }

    pub fn rank<S: AsRef<str>>(&self, address: S) -> Result<String> {
        let address: &str = address.as_ref();
        Ok(self.validators()?.validator(address)?.rank().to_string())
    }

    pub fn table(&self) -> String {

        let mut rows: Vec<TableColumns> = Vec::new();

        // want to convert DateTime<Utc> to DateTime<Local>
        let timestamp_local: DateTime<Local> = self.timestamp.with_timezone(&Local);

        rows.push(TableColumns::new(vec![
            &format!("Delegations for \x1b[32m{}\x1b[0m as at \x1b[32m{}\x1b[0m",
                self.address,
                timestamp_local.format("%Y-%m-%d %H:%M"),
            ),
        ]));

        rows.push(TableColumns::new(vec![
            "Rank",
            "Validator Address",
            "Moniker",
            "Staked",
            "Liquid",
            "NBTC",
        ]));

        let mut data_rows: Vec<TableColumns> = Vec::new();
        // Iterate over the `delegations` field in `Delegations`
        for (address, delegation) in self.delegations.iter() {
        // Attempt to retrieve validators and handle errors if they occur
            data_rows.push(TableColumns::new(vec![
                &self.rank(address).unwrap_or_else(|e| { warn!("Unable to get rank: {}", e); "N/A".to_string() }),
                address,
                &self.moniker(address).unwrap_or_else(|e| { warn!("Unable to get moniker: {}", e); "N/A".to_string() }),
                &NumberDisplay::new(delegation.staked).scale(6).decimal_places(6).trim(true).format(),
                &NumberDisplay::new(delegation.liquid).scale(6).decimal_places(6).format(),
                &NumberDisplay::new(delegation.nbtc).scale(8).decimal_places(8).trim(false).format(),
            ]));
        }

        if data_rows.is_empty() {
            warn!("No data.");
            return String::new();
        }

        // Sort rows descending by `cell0`, converting each `cell0` to `usize`
        data_rows.sort_by(|a, b| {
            a.cell0.parse::<usize>().unwrap_or(0).cmp(&b.cell0.parse::<usize>().unwrap_or(0))
        });

        // Append `data_rows` to `rows`
        rows.extend(data_rows);

        // Create totals row
        rows.push(TableColumns::new(vec![
            "",
            "",
            "",
            &NumberDisplay::new(self.total().staked).scale(6).decimal_places(6).trim(true).format(),
            &NumberDisplay::new(self.total().liquid).scale(6).decimal_places(6).trim(false).format(),
            &NumberDisplay::new(self.total().nbtc).scale(8).decimal_places(8).trim(false).format(),
        ]));

        rows.push(TableColumns::new(vec![
            "Total staked and liquid",
            "",
            &NumberDisplay::new(self.total().staked.saturating_add(self.total().liquid)).scale(6).decimal_places(6).trim(true).format(),
        ]));

        // Initialize Builder without headers
        let mut builder = Builder::default();
        for row in &rows {
            builder.push_record([
                row.cell0.clone(),
                row.cell1.clone(),
                row.cell2.clone(),
                row.cell3.clone(),
                row.cell4.clone(),
                row.cell5.clone(),
            ]);
        }

        let mut table = builder.build();

        table
            .with(Style::blank())
            .with(Modify::new(Cell::new(0, 0)).with(Span::column(6)).with(Alignment::left()))
            .with(Modify::new(Cell::new(rows.len() -1, 0)).with(Span::column(2)).with(Alignment::right()))
            .with(Modify::new(Cell::new(rows.len() -2, 0)).with(Span::column(3)).with(Alignment::right()))
            .with(Modify::new(Cell::new(rows.len() -1, 2)).with(Border::new()).with(Color::FG_YELLOW))
            .with(Modify::new(Cell::new(rows.len() -2, 3)).with(Border::new().set_top('-')).with(Color::FG_GREEN))
            .with(Modify::new(Cell::new(rows.len() -2, 4)).with(Border::new().set_top('-')).with(Color::FG_GREEN))
            .with(Modify::new(Cell::new(rows.len() -2, 5)).with(Border::new().set_top('-')).with(Color::FG_GREEN))
            .with(Modify::new(Columns::single(0)).with(Alignment::right()))
            .with(Modify::new(Columns::new(3..)).with(Alignment::right()))
            .with(Modify::new(Rows::single(1)).with(Border::new().set_bottom('-')))
            .with(Modify::new(Rows::single(rows.len() - 2)).with(Alignment::right()))
            ;
        table.to_string()
    }
}

impl fmt::Display for Delegations {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n{}", self.table())
    }
}

