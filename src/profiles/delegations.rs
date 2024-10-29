
use chrono::{Utc, DateTime};
use crate::functions::format_to_millions;
use crate::global::CONFIG;
use crate::validators::ValidatorCollection;
use eyre::{eyre, Result};
use indexmap::IndexMap;
use once_cell::sync::OnceCell;
use std::fmt;
use std::path::Path;
use std::process::Command;
use tabled::{Tabled, Table, settings::{Alignment, Border, Modify, Span, Style, object::{Columns, Rows, Cell}}};

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

    pub fn find(&self, address: &str) -> Result<&Delegation> {
        self.delegations.get(address)
            .ok_or_else(|| eyre!("Delegation not found for address: {}", address))
    }

    /// Creates a new Delegations instance.
    pub fn new(
        timestamp: Option<DateTime<Utc>>,
        validators: Option<ValidatorCollection>,
    ) -> Self {
        Delegations {
            timestamp:   timestamp.unwrap_or(Utc::now()),
            delegations: IndexMap::new(),
            total:       OnceCell::new(),
            validators:  ValidatorCollection::initialize_oncecell(validators),
        }
    }

    /// Adds a delegation to the collection.
    pub fn add_delegation(&mut self, id: String, delegation: Delegation) {
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
    pub fn fetch<P: AsRef<Path>>(home: Option<P>) -> Result<Self> {

        let timestamp = Some(Utc::now());

        // Create and configure the Command
        let mut cmd = Command::new(CONFIG.nomic()?); // Replace with the actual command string

        // Set environment variables
        if let Some(ref version) = CONFIG.nomic_legacy_version {
            cmd.env("NOMIC_LEGACY_VERSION", version);
        }

        cmd.arg("delegations");
        if let Some(home_str) = home.as_ref().map(|p| p.as_ref().to_str()).flatten() {
            cmd.env("HOME", home_str);
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

        let mut delegations = Delegations::new(timestamp, None);

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

    pub fn moniker(&self, address: &str) -> String {
        match self.validators() {
            Ok(validators) => {
                // Attempt to get the moniker, and handle errors gracefully
                validators.validator(address)
                    .map(|validator| validator.moniker()) // Extract the moniker if successful
                    .unwrap_or_else(|_| "N/A").to_string() // Handle the error
            },
            Err(_) => "N/A".to_string(), // Handle error from validators()
        }
    }

    pub fn format_table(&self) -> String {
        // Prepare rows for the table
        let mut rows: Vec<DelegationRow> = Vec::new();

        // Add header row automatically handled by the Tabled derive
        for (address, delegation) in &self.delegations {
            let row = DelegationRow::from_data(
                address.clone(),
                self.moniker(address),
                delegation.staked,
                delegation.liquid,
                delegation.nbtc,
            );
            rows.push(row);
        }

        // Create totals row
        let totals_row = DelegationRow::from_data(
            format!("{} {}",
             self.timestamp.format("%Y-%m-%d %H:%M").to_string(),
             "Total"
            ),
            "".to_string(),
            self.total().staked,
            self.total().liquid,
            self.total().nbtc,
        );
        rows.push(totals_row);

        // Create the table and apply styles and modifications
         let mut table = Table::new(rows.clone()); // Clone the rows here to pass ownership to the table

        table
            .with(Style::empty()) // Use an empty style
            .with(Modify::new(Columns::new(2..)).with(Alignment::right())) // Staked, Liquid, NBTC columns align right
            // Set borders on the first row
            .with(Modify::new(Cell::new(0, 0)).with(Border::new().set_bottom('-')))
            .with(Modify::new(Cell::new(0, 1)).with(Border::new().set_bottom('-')))
            .with(Modify::new(Cell::new(0, 2)).with(Border::new().set_bottom('-')))
            .with(Modify::new(Cell::new(0, 3)).with(Border::new().set_bottom('-')))
            .with(Modify::new(Cell::new(0, 4)).with(Border::new().set_bottom('-')))
            // Apply right alignment to the totals row (last row)
            .with(Modify::new(Rows::single(rows.len())).with(Alignment::right()))
            // Apply a distinct bottom border to the totals row
            .with(Modify::new(Cell::new(rows.len(), 2)).with(Border::new().set_top('-')))
            .with(Modify::new(Cell::new(rows.len(), 3)).with(Border::new().set_top('-')))
            .with(Modify::new(Cell::new(rows.len(), 4)).with(Border::new().set_top('-')))
            // Apply span to the first two cells in the last row
            .with(Modify::new(Cell::new(rows.len(), 0)).with(Span::column(2)));

        // Return the formatted table as a string
        table.to_string()
    }
}

#[derive(Clone, Tabled)]
pub struct DelegationRow {
    #[tabled(rename = "Address")]
    address: String,

    #[tabled(rename = "Moniker")]
    moniker: String,

    #[tabled(rename = "Staked")]
    staked: String,

    #[tabled(rename = "Liquid")]
    liquid: String,

    #[tabled(rename = "NBTC")]
    nbtc: String,
}

impl DelegationRow {
    pub fn from_data(address: String, moniker: String, staked: u64, liquid: u64, nbtc: u64) -> Self {
        DelegationRow {
            address,
            moniker,
            staked: format_to_millions(staked, Some(2)),
            liquid: format_to_millions(liquid, Some(2)),
            nbtc: format_to_millions(nbtc, Some(2)),
        }
    }
}


impl fmt::Display for Delegations {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n{}", self.format_table())
    }
}

