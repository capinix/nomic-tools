use clap::Args;
use crate::functions::validate_positive;
use crate::profiles::ProfileCollection;
use eyre::Result;

#[derive(Debug, Args)]
#[command( about = "Delegate")]
pub struct Command {
    profile: String,

    /// The validator address or moniker
    validator: Option<String>,

    /// The amount to delegate
    #[arg( value_parser = validate_positive::<f64>)]
    quantity: Option<f64>,
}

impl Command {
    pub fn run(&self) -> Result<()> {
        ProfileCollection::new()?
            .profile_by_name_or_address_or_home_or_default(Some(&self.profile))?
            .nomic_delegate(self.validator.clone(), self.quantity, false)
    }
}
