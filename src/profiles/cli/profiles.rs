// src/profiles.rs
use eyre::Result;
use crate::profiles::ProfileCollection;
use crate::profiles::CollectionOutputFormat;
use clap::Args;

#[derive(Debug, Args)]
#[command(about = "List Profiles")]
pub struct Command {
    pub format: Option<CollectionOutputFormat>,
}

impl Command {
    pub fn run(&self) -> Result<()> {
        ProfileCollection::new()?.print(self.format.clone())
    }
}
