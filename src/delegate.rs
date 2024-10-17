
//use clap::Parser;
use clap::Subcommand;
use crate::profiles::ProfileCollection;
use eyre::Result;

// Defines the CLI structure for the `delegate` command.
//#[derive(Parser)]
//#[command(name = "Delegate", about = "Delegate to Validators")]
//pub struct Cli {
//    #[command(subcommand)]
//    pub command: Commands,
//}

//#[derive(Parser)]
#[derive(Subcommand)]
pub enum Cli {
    Delegate {
        /// The validator to delegate to
        #[arg()]
        profile: String,

        /// The validator to delegate to
        #[arg()]
        validator: String,

        /// The quantity to delegate
        #[arg()]
        quantity: f64,
    },
}


impl Cli {
    pub fn run(&self) -> Result<()> {
        match self.command {
            Commands::Delegate { ref profile, ref validator, quantity } => {
                ProfileCollection::new()?
                    .profile_by_name_or_address(profile)?
                    .nomic_delegate(Some(validator.to_string()), Some(quantity))?;
                Ok(())
            }
        }
    }
}
