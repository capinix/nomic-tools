use clap::Parser;
use clap::Subcommand;
use crate::validators::OutputFormat;
use crate::validators::ValidatorCollection;

/// Defines the CLI structure for the `validators` command.
#[derive(Parser)]
#[command(name = "validators", about = "Print validators")]
    pub struct Cli {
    /// Specify the output format
    #[arg(long, short)]
    pub format: Option<OutputFormat>,

    /// Wether to include the details field
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

        /// Wether to include the details field
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

        /// Wether to include the details field
        #[arg(short, long)]
        include_details: Option<bool>,

        /// Column widths for table view
        #[arg(short, long)]
        column_widths: Option<Vec<usize>>,
    },
    /// Skip the first N validators
    Skip {
        /// Number of validators to skip
        #[arg(value_parser = clap::value_parser!(usize), required = true)]
        number: usize,

        /// Specify the output format
        #[arg(long, short)]
        format: Option<OutputFormat>,

        /// Wether to include the details field
        #[arg(short, long)]
        include_details: Option<bool>,

        /// Column widths for table view
        #[arg(short, long)]
        column_widths: Option<Vec<usize>>,
    },
    /// Show a specified number of random validators outside a specified top percentage
    Random {
        /// Number of random validators to show
        #[arg(short, long, value_parser = clap::value_parser!(usize), required = true)]
        count: usize,
        
        /// Percentage of validators to consider for randomness
        #[arg(short, long, value_parser = clap::value_parser!(u8), required = true)]
        percent: u8,

        /// Specify the output format
        #[arg(long, short)]
        format: Option<OutputFormat>,

        /// Wether to include the details field
        #[arg(short, long)]
        include_details: Option<bool>,

        /// Column widths for table view
        #[arg(short, long)]
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

        /// Wether to include the details field
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

        /// Wether to include the details field
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
        let collection = ValidatorCollection::init().context("Failed to initialize validator collection")?;

        // Handle subcommands
        match &self.command {
            // Handle address subcommand
            Some(CliCommand::Address { address, format, include_details, column_widths }) => {
                let format = format.clone().or_else(|| self.format.clone());
                let include_details = include_details.or_else(|| self.include_details);
                let column_widths = column_widths.clone().or_else(|| self.column_widths.clone());

                if !address.is_empty() {
                    let filtered = collection.search_by_address(address);
                    if filtered.is_empty() {
                        eprintln!("No validators found with the address: {}", address);
                    } else {
                        filtered.print(format, include_details, column_widths.clone());
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
                    let filtered = collection.search_by_moniker(moniker);
                    if filtered.is_empty() {
                        eprintln!("No validators found with the moniker: {}", moniker);
                    } else {
                        filtered.print(format, include_details, column_widths.clone());
                    }
                } else {
                    eprintln!("Validator moniker is empty.");
                }
            },

            // Handle top subcommand
            Some(CliCommand::Top { number, format, include_details, column_widths }) => {
                let format = format.clone().or_else(|| self.format.clone());
                let include_details = include_details.or_else(|| self.include_details);
                let column_widths = column_widths.clone().or_else(|| self.column_widths.clone());

                let filtered = collection.top(*number);
                if filtered.is_empty() {
                    eprintln!("No validators found in the top {}", number);
                } else {
                    filtered.print(format, include_details, column_widths.clone());
                }
            },

            // Handle bottom subcommand
            Some(CliCommand::Bottom { number, format, include_details, column_widths }) => {
                let format = format.clone().or_else(|| self.format.clone());
                let include_details = include_details.or_else(|| self.include_details);
                let column_widths = column_widths.clone().or_else(|| self.column_widths.clone());

                let filtered = collection.bottom(*number);
                if filtered.is_empty() {
                    eprintln!("No validators found in the bottom {}", number);
                } else {
                    filtered.print(format, include_details, column_widths.clone());
                }
            },

            // Handle skip subcommand
            Some(CliCommand::Skip { number, format, include_details, column_widths }) => {
                let format = format.clone().or_else(|| self.format.clone());
                let include_details = include_details.or_else(|| self.include_details);
                let column_widths = column_widths.clone().or_else(|| self.column_widths.clone());

                let filtered = collection.skip(*number);
                if filtered.is_empty() {
                    eprintln!("No validators found after skipping {}", number);
                } else {
                    filtered.print(format, include_details, column_widths.clone());
                }
            },

            // Handle random subcommand
            Some(CliCommand::Random { count, percent, format, include_details, column_widths }) => {
                let format = format.clone().or_else(|| self.format.clone());
                let include_details = include_details.or_else(|| self.include_details);
                let column_widths = column_widths.clone().or_else(|| self.column_widths.clone());

                let filtered = collection.random(*count, *percent);
                if filtered.is_empty() {
                    eprintln!("No random validators found");
                } else {
                    filtered.print(format, include_details, column_widths.clone());
                }
            },

            // Default case when no subcommand is provided
            None => {
                collection.print(self.format.clone(), self.include_details, self.column_widths.clone());
            },
        }
        Ok(()) // Return Ok if everything executes successfully
    }
}
