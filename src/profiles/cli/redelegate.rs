// src/redelegate.rs
use clap::Args;
use crate::functions::validate_positive;
use crate::profiles::ProfileCollection;
use eyre::Result;

#[derive(Debug, Args)]
#[command( about = "Redelegate")]
pub struct Command {
    /// Profile
    #[arg(required = true)]
    profile: String,

    /// The validator to redelegate from
    #[arg(help = "redelegate from ..")]
    from: String,

    /// The validator to redelegate to
    #[arg(help = "redelegate to ..")]
    to: String,

    /// The quantity to redelegate in nom
    #[arg(
        help = "Quantity to Redelegate (NOM)", 
        value_parser = validate_positive::<f64>,
    )]
    quantity: f64,
}

impl Command {
    pub fn run(&self) -> Result<()> {
        ProfileCollection::new()?
            .profile_by_name_or_address_or_home_or_default(Some(&self.profile))?
            .redelegate(&self.from, &self.to, self.quantity)
    }
}
