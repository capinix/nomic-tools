use clap::Parser;
use clap::Subcommand;
use crate::validators::OutputFormat;
use crate::validators::ValidatorCollection;
use eyre::Result;

/// Defines the CLI structure for the `validators` command.
#[derive(Parser)]
#[command(name = "validators", about = "Print validators", visible_alias = "v")]
pub struct Cli {
    /// Specify the output format
    #[arg(long, short)]
    pub format: Option<OutputFormat>,

    /// Whether to include the details field
    #[arg(short, long)]
    pub include_details: Option<bool>,

    /// Column widths for table view
    #[arg(short, long)]
    pub column_widths: Option<Vec<usize>>,

    /// Subcommands for the validators command
    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

/// Subcommands for the `validators` command
#[derive(Subcommand)]
pub enum CliCommand {
    /// Show the top N validators
    Top {
        /// Number of top validators to show
        #[arg(value_parser = clap::value_parser!(usize), required = true)]
        number: usize,

        /// Specify the output format
        #[arg(default_value = "json-pretty", long, short)]
        format: Option<OutputFormat>,

        /// Whether to include the details field
        #[arg(default_value = "false", short, long)]
        include_details: Option<bool>,

        /// Column widths for table view
        #[arg(short, long)]
        column_widths: Option<Vec<usize>>,
    },
    /// Show the bottom N validators
    Bottom {
        /// Number of bottom validators to show
        #[arg(value_parser = clap::value_parser!(usize), required = true)]
        number: usize,

        /// Specify the output format
        #[arg(long, short)]
        format: Option<OutputFormat>,

        /// Whether to include the details field
        #[arg(short, long)]
        include_details: Option<bool>,

        /// Column widths for table view
        #[arg(short, long)]
        column_widths: Option<Vec<usize>>,
    },
    /// Skip the top N validators
    SkipTop {
        /// Number of validators to skip
        #[arg(value_parser = clap::value_parser!(usize), required = true)]
        number: usize,

        /// Specify the output format
        #[arg(long, short)]
        format: Option<OutputFormat>,

        /// Whether to include the details field
        #[arg(short, long)]
        include_details: Option<bool>,

        /// Column widths for table view
        #[arg(short, long)]
        column_widths: Option<Vec<usize>>,
    },
    /// Skip the bottom N validators
    SkipBottom {
        /// Number of validators to skip
        #[arg(value_parser = clap::value_parser!(usize), required = true)]
        number: usize,

        /// Specify the output format
        #[arg(long, short)]
        format: Option<OutputFormat>,

        /// Whether to include the details field
        #[arg(short, long)]
        include_details: Option<bool>,

        /// Column widths for table view
        #[arg(short, long)]
        column_widths: Option<Vec<usize>>,
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

        /// Specify the output format
        #[arg(long, short = 'f')]
        format: Option<OutputFormat>,

        /// Whether to include the details field
        #[arg(short = 'd', long)]
        include_details: Option<bool>,

        /// Column widths for table view
        #[arg(short = 'w', long)]
        column_widths: Option<Vec<usize>>,
    },
    /// Search for validators by moniker
    Moniker {
        /// Search for validators by moniker
        #[arg(value_parser = clap::value_parser!(String), required = true)]
        moniker: String,

        /// Specify the output format
        #[arg(long, short)]
        format: Option<OutputFormat>,

        /// Whether to include the details field
        #[arg(default_value = "true", short, long)]
        include_details: Option<bool>,

        /// Column widths for table view
        #[arg(short, long)]
        column_widths: Option<Vec<usize>>,
    },
    /// Search for a validator by address
    Address {
        /// Search for a validator by its address
        #[arg(value_parser = clap::value_parser!(String), required = true)]
        address: String,

        /// Specify the output format
        #[arg(long, short)]
        format: Option<OutputFormat>,

        /// Whether to include the details field
        #[arg(default_value = "true", short, long)]
        include_details: Option<bool>,

        /// Column widths for table view
        #[arg(short, long)]
        column_widths: Option<Vec<usize>>,
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

        /// Specify the output format
        #[arg(long, short)]
        format: Option<OutputFormat>,

        /// Whether to include the details field
        #[arg(default_value = "true", short, long)]
        include_details: Option<bool>,

        /// Column widths for table view
        #[arg(short, long)]
        column_widths: Option<Vec<usize>>,
    },

}

impl Cli {
    // Change the function to be a method of Cli
    pub fn run(&self) -> Result<()> {
        let collection = ValidatorCollection::fetch()?;

        // Handle subcommands
        match &self.command {
            // Handle address subcommand
            Some(CliCommand::Address { address, format, include_details, column_widths }) => {
                let format = format.clone().or_else(|| self.format.clone());
                let include_details = include_details.or_else(|| self.include_details);
                let column_widths = column_widths.clone().or_else(|| self.column_widths.clone());

                if !address.is_empty() {
                    let filtered = collection.filter_address(address)?;
                    if filtered.is_empty() {
                        eprintln!("No validators found with the address: {}", address);
                    } else {
                        filtered.print(format, include_details, column_widths.clone())?;
                    }
                } else {
                    eprintln!("Validator address is empty.");
                }
            },

            // Handle moniker subcommand
            Some(CliCommand::Moniker { moniker, format, include_details, column_widths }) => {
                let format = format.clone().or_else(|| self.format.clone());
                let include_details = include_details.or_else(|| self.include_details);
                let column_widths = column_widths.clone().or_else(|| self.column_widths.clone());

                if !moniker.is_empty() {
                    let filtered = collection.filter_moniker(moniker)?;
                    if filtered.is_empty() {
                        eprintln!("No validators found with the moniker: {}", moniker);
                    } else {
                        filtered.print(format, include_details, column_widths.clone())?;
                    }
                } else {
                    eprintln!("Validator moniker is empty.");
                }
            },

            // Handle search subcommand
            Some(CliCommand::Search { search, searches, format, include_details, column_widths }) => {
                let format = format.clone().or_else(|| self.format.clone());
                let include_details = include_details.or_else(|| self.include_details);
                let column_widths = column_widths.clone().or_else(|| self.column_widths.clone());

                let mut any_found = false; // Declare the flag before checking searches

                // Check for a single search term
                if let Some(single_search) = search {
                    let filtered = collection.search(&single_search)?;
                    if !filtered.is_empty() {
                        any_found = true; // Found at least one match
                        filtered.print(format.clone(), include_details, column_widths.clone())?;
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
                        filtered.print(format.clone(), include_details, column_widths.clone())?;
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
            Some(CliCommand::Top { number, format, include_details, column_widths }) => {
                let format = format.clone().or_else(|| self.format.clone());
                let include_details = include_details.or_else(|| self.include_details);
                let column_widths = column_widths.clone().or_else(|| self.column_widths.clone());

                let filtered = collection.top(Some(*number))?;
                if filtered.is_empty() {
                    eprintln!("No validators found in the top {}", number);
                } else {
                    filtered.print(format, include_details, column_widths.clone())?;
                }
            },

            // Handle bottom subcommand
            Some(CliCommand::Bottom { number, format, include_details, column_widths }) => {
                let format = format.clone().or_else(|| self.format.clone());
                let include_details = include_details.or_else(|| self.include_details);
                let column_widths = column_widths.clone().or_else(|| self.column_widths.clone());

                let filtered = collection.bottom(Some(*number))?;
                if filtered.is_empty() {
                    eprintln!("No validators found in the bottom {}", number);
                } else {
                    filtered.print(format, include_details, column_widths.clone())?;
                }
            },

            // Handle skip top subcommand
            Some(CliCommand::SkipTop { number, format, include_details, column_widths }) => {
                let format = format.clone().or_else(|| self.format.clone());
                let include_details = include_details.or_else(|| self.include_details);
                let column_widths = column_widths.clone().or_else(|| self.column_widths.clone());

                let filtered = collection.skip_top(Some(*number))?;
                if filtered.is_empty() {
                    eprintln!("No validators found after skipping {}", number);
                } else {
                    filtered.print(format, include_details, column_widths.clone())?;
                }
            },

            // Handle skip bottom subcommand
            Some(CliCommand::SkipBottom { number, format, include_details, column_widths }) => {
                let format = format.clone().or_else(|| self.format.clone());
                let include_details = include_details.or_else(|| self.include_details);
                let column_widths = column_widths.clone().or_else(|| self.column_widths.clone());

                let filtered = collection.skip_bottom(Some(*number))?;
                if filtered.is_empty() {
                    eprintln!("No validators found after skipping {}", number);
                } else {
                    filtered.print(format, include_details, column_widths.clone())?;
                }
            },

            // Handle random subcommand
            Some(CliCommand::Random { count, skip_top, skip_bottom, format, include_details, column_widths }) => {
                let format = format.clone().or_else(|| self.format.clone());
                let include_details = include_details.or_else(|| self.include_details);
                let column_widths = column_widths.clone().or_else(|| self.column_widths.clone());

                let filtered = collection.random(Some(*count), Some(*skip_top), Some(*skip_bottom))?;
                if filtered.is_empty() {
                    eprintln!("No random validators found");
                } else {
                    filtered.print(format, include_details, column_widths.clone())?;
                }
            },

            // Default case when no subcommand is provided
            None => {
                collection.print(self.format.clone(), self.include_details, self.column_widths.clone())?;
            },
        }
        Ok(()) // Return Ok if everything executes successfully
    }
}
