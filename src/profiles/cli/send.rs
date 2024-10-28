use clap::Args;
use crate::functions::validate_positive;
use crate::profiles::ProfileCollection;
use eyre::Result;

#[derive(Debug, Args)]
#[command(about = "Send")]
pub struct Command {
    /// Profile
    #[arg(required = true)]
    profile: String,

    /// The validator to delegate to
    #[arg(
        // short, long,
        help = "destination address"
    )]
    destination: Option<String>,

    /// The quantity to send
    #[arg(
        // short, long, 
        help = "Quantity to send", 
        value_parser = validate_positive::<f64>,
    )]
    quantity: Option<f64>,
}

impl Command {
    pub fn run(&self) -> Result<()> {
        ProfileCollection::new()?.send(
            Some(&self.profile),
            self.destination.as_deref(),
            self.quantity
        )
    }
}
