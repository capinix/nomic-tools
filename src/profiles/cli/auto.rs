
use clap::Args;
use crate::profiles::ProfileCollection;
use eyre::Result;

#[derive(Debug, Args)]
#[command(about = "Auto delegate all profiles")]
pub struct Command;

impl Command {
    pub fn run(&self) -> Result<()> {
        ProfileCollection::load()?.auto_delegate()
    }
}
