use clap::{Parser, Subcommand};
use crate::validators::{OutputFormat, ValidatorCollection};
use eyre::Result;

#[derive(Debug, Parser)]
pub struct Options {
    /// Specify the output format
    #[arg(long, short, default_value = "table")]
    pub format: Option<OutputFormat>,

    /// Whether to include the details field
    #[arg(short = 'd', long, action = clap::ArgAction::SetTrue)]
    pub details: bool,
}

#[derive(Debug, Parser)]
//#[command(group(ArgGroup::new("output_options").args(&["format", "include_details"])))]
pub struct NumberOptions {
    /// Number of validators to show
    #[arg(value_parser = clap::value_parser!(usize), required = true)]
    pub number: usize,

    /// Specify the output format
    #[arg(long, short, default_value = "table")]
    pub format: Option<OutputFormat>,

    /// Whether to include the details field
    #[arg(short = 'd', long, action = clap::ArgAction::SetTrue)]
    pub details: bool,
}

#[derive(Debug, Parser)]
//#[command(group(ArgGroup::new("output_options").args(&["format", "include_details"])))]
pub struct SkipOptions {
    /// Number of validators to skip
    #[arg(value_parser = clap::value_parser!(usize), required = true)]
    pub number: usize,

    /// Specify the output format
    #[arg(long, short, default_value = "table")]
    pub format: Option<OutputFormat>,

    /// Whether to include the details field
    #[arg(short = 'd', long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    pub details: bool,
}


/// Defines the CLI structure for the `validators` command.
#[derive(Parser)]
#[command(about = "Print validators")]
pub struct Cli {
    /// Specify the output format
    #[command(flatten)]
    pub options: Options,

    /// Subcommands for the validators command
    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

/// Subcommands for the `validators` command
#[derive(Subcommand)]
pub enum CliCommand {
    /// Show the top N validators
    Top {
        #[command(flatten)]
        options: NumberOptions,
    },
    /// Show the bottom N validators
    Bottom {
        #[command(flatten)]
        options: NumberOptions,
    },
    /// Skip the top N validators
    SkipTop {
        #[command(flatten)]
        options: SkipOptions,
    },
    /// Skip the bottom N validators
    SkipBottom {
        #[command(flatten)]
        options: SkipOptions,
    },
    /// Show a specified number of random validators outside a specified top percentage
    Random {
        /// Number of random validators to show
        #[arg(short = 'c', long, value_parser = clap::value_parser!(usize), required = true)]
        count: usize,

        #[arg(short = 't', long, value_parser = clap::value_parser!(usize), default_value_t = 20)]
        skip_top: usize,

        #[arg(short = 'b', long, value_parser = clap::value_parser!(usize), default_value_t = 10)]
        skip_bottom: usize,

        #[command(flatten)]
        options: Options,
    },
    /// Search for validators by moniker
    Moniker {
        /// Search for validators by moniker
        #[arg(value_parser = clap::value_parser!(String), required = true)]
        moniker: String,

        #[command(flatten)]
        options: Options,
    },
    /// Search for a validator by address
    Address {
        /// Search for a validator by its address
        #[arg(value_parser = clap::value_parser!(String), required = true)]
        address: String,

        #[command(flatten)]
        options: Options,
    },

    /// Search for a validator by its address
    Search {
        /// Search for a validator by its address
        #[arg(value_parser = clap::value_parser!(String), required_unless_present = "searches")]
        search: Option<String>,

        /// Search for validators by their addresses or monikers
        #[arg(
            long,
            short = 't',
            value_parser = clap::value_parser!(String),
            num_args = 1..,
            required_unless_present = "search",
        )]
        searches: Option<Vec<String>>,

        #[command(flatten)]
        options: Options,
    },

}

impl Cli {
    // Change the function to be a method of Cli
    pub fn run(&self) -> Result<()> {
        let collection = ValidatorCollection::fetch()?;

        // Handle subcommands
        match &self.command {
            // Handle address subcommand
            Some(CliCommand::Address { address, options }) => {
                if !address.is_empty() {
                    let format = options.format.as_ref().or(self.options.format.as_ref()).unwrap();
                    let details = options.details || self.options.details;
                    let filtered = collection.filter_address(address)?;
                    if filtered.is_empty() {
                        eprintln!("No validators found with the address: {}", address);
                    } else {
                        println!("");
                        filtered.print(Some(format.clone()), details)?;
                        println!("");
                    }
                } else {
                    eprintln!("Validator address is empty.");
                }
            },

            // Handle moniker subcommand
            Some(CliCommand::Moniker { moniker, options }) => {
                if !moniker.is_empty() {
                    let format = options.format.as_ref().or(self.options.format.as_ref()).unwrap();
                    let details = options.details || self.options.details;
                    let filtered = collection.filter_moniker(moniker)?;
                    if filtered.is_empty() {
                        eprintln!("No validators found with the moniker: {}", moniker);
                    } else {
                        println!("");
                        filtered.print(Some(format.clone()), details)?;
                        println!("");
                    }
                } else {
                    eprintln!("Validator moniker is empty.");
                }
            },

            // Handle search subcommand
            Some(CliCommand::Search { search, searches, options }) => {

                let mut any_found = false; // Declare the flag before checking searches
                let format = options.format.as_ref().or(self.options.format.as_ref()).unwrap();
                let details = options.details || self.options.details;

                // Check for a single search term
                if let Some(single_search) = search {
                    let filtered = collection.search(&single_search)?;
                    if !filtered.is_empty() {
                        any_found = true; // Found at least one match
                        println!("");
                        filtered.print(Some(format.clone()), details)?;
                        println!("");
                    } else {
                        eprintln!("No validators found with the search: {}", single_search);
                    }
                }

                // Check for multiple search terms
                if let Some(multi_search) = searches {
                    // Perform search for all terms at once
                    let filtered = collection.search_multi(multi_search.to_vec())?;

                    // Check if any validators were found
                    if !filtered.is_empty() {
                        any_found = true; // Found at least one match
                        println!("");
                        filtered.print(Some(format.clone()), details)?;
                        println!("");
                    } else {
                        eprintln!("No validators found matching any of the provided searches.");
                    }
                }

                // If no matches were found in any of the searches, notify the user
                if !any_found {
                    eprintln!("No validators found with any of the specified searches.");
                }
            },

            // Handle top subcommand
            Some(CliCommand::Top { options }) => {
                let filtered = collection.top(Some(options.number))?;
                let format = options.format.as_ref().or(self.options.format.as_ref()).unwrap();
                let details = options.details || self.options.details;
                if filtered.is_empty() {
                    eprintln!("No validators found in the top {}", options.number);
                } else {
                    println!("");
                    filtered.print(Some(format.clone()), details)?;
                    println!("");
                }
            },

            // Handle bottom subcommand
            Some(CliCommand::Bottom { options }) => {
                let filtered = collection.bottom(Some(options.number))?;
                let format = options.format.as_ref().or(self.options.format.as_ref()).unwrap();
                let details = options.details || self.options.details;
                if filtered.is_empty() {
                    eprintln!("No validators found in the bottom {}", options.number);
                } else {
                    println!("");
                    filtered.print(Some(format.clone()), details)?;
                    println!("");
                }
            },

            // Handle skip top subcommand
            Some(CliCommand::SkipTop { options }) => {
                let filtered = collection.skip_top(Some(options.number))?;
                let format = options.format.as_ref().or(self.options.format.as_ref()).unwrap();
                let details = options.details || self.options.details;
                if filtered.is_empty() {
                    eprintln!("No validators found after skipping {}", options.number);
                } else {
                    println!("");
                    filtered.print(Some(format.clone()), details)?;
                    println!("");
                }
            },

            // Handle skip bottom subcommand
            Some(CliCommand::SkipBottom { options }) => {
                let filtered = collection.skip_bottom(Some(options.number))?;
                let format = options.format.as_ref().or(self.options.format.as_ref()).unwrap();
                let details = options.details || self.options.details;
                if filtered.is_empty() {
                    eprintln!("No validators found after skipping {}", options.number);
                } else {
                    println!("");
                    filtered.print(Some(format.clone()), details)?;
                    println!("");
                }
            },

            // Handle random subcommand
            Some(CliCommand::Random { count, skip_top, skip_bottom, options}) => {
                let filtered = collection.random(Some(*count), Some(*skip_top), Some(*skip_bottom))?;
                let format = options.format.as_ref().or(self.options.format.as_ref()).unwrap();
                let details = options.details || self.options.details;
                if filtered.is_empty() {
                    eprintln!("No random validators found");
                } else {
                    println!("");
                    filtered.print(Some(format.clone()), details)?;
                    println!("");
                }
            },

            // Default case when no subcommand is provided
            None => {
                println!("");
                collection.print(self.options.format.clone(), self.options.details)?;
                println!("");
            },
        }
        Ok(()) // Return Ok if everything executes successfully
    }
}
