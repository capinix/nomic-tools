// src/nomic.rs
use eyre::Result;
use crate::profiles::ProfileCollection;
use crate::profiles::nomic;
use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Run Nomic commands as Profile")]
pub struct Command {
    pub profile: String,
    pub args: Vec<String>,
}

impl Command {
    pub fn run(&self) -> Result<()> {
        let collection = ProfileCollection::new()?;
        let home_path = collection.home(Some(&self.profile))?;
        nomic(&home_path, Some(String::new()), self.args.clone())
    }
}
