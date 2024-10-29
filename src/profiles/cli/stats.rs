

use clap::Args;
use crate::profiles::ProfileCollection;
use eyre::Result;

#[derive(Debug, Args)]
#[command( about = "Display Statistics")]
pub struct Command {
    #[arg()]
    profile: Option<String>,
}

impl Command {
    pub fn run(&self) -> Result<()> {
        let profile = ProfileCollection::new()?
            .profile_by_name_or_address_or_home_or_default(self.profile.as_deref())?;
        Ok(println!("{}", profile.stats()?))
    }
}
