use clap::Parser;
use clap::Subcommand;
use eyre::Result;

#[derive(Parser)]
#[command(name = "z")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

#[derive(Subcommand)]
pub enum CliCommand {
    Test,
}

impl Cli {
    // Change the function to be a method of Cli
    pub fn run(&self) -> Result<()> {
        // Handle subcommands
        match &self.command {
            // Handle address subcommand
            Some(CliCommand::Test) => {


                return Ok(());
            },

            // Default case when no subcommand is provided
            None => {
                println!("No command selected");
            },
        }
        Ok(()) // Return Ok if everything executes successfully
    }
}
