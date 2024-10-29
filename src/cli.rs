use clap::Parser;
use clap::Subcommand;
use crate::global;
use crate::journal;
use crate::nonce;
use crate::privkey;
use crate::profiles;
use crate::validators;
use crate::z;
use eyre::Result;
use fmt;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {

    #[command(visible_alias = "ad", aliases = ["add", "addr", "addre", "addres"])]
    Address(profiles::cli::address::Command),

    #[command( visible_alias = "au", aliases = ["aut", "autod"])]
    AutoDelegate(profiles::cli::auto::Command),

    /// Display the Balance for a profile
    #[command( visible_alias = "b", aliases = ["ba", "bal", "bala", "balan", "balanc"])]
    Balance(profiles::cli::balance::Command),

    #[command(visible_alias = "cl", aliases = ["cla", "clai"])]
    Claim(profiles::cli::claim::Command),

    #[command( visible_alias = "co", aliases = ["con", "conf", "confi"])]
    Config(profiles::cli::config::Cli),

    #[command(visible_alias = "de")]
    Delegate(profiles::cli::delegate::Command),

    #[command(visible_alias = "ds", aliases = ["dn", "delegati", "delegatio", "delegation"])]
    Delegations(profiles::cli::delegations::Command),

    #[command( visible_alias = "ex", aliases = ["exp", "expo", "expor"])]
    Export(profiles::cli::export::Command),

    Fmt(fmt::cli::Cli),

    #[command(visible_alias = "g", aliases = ["global"])]
    GlobalConfig(global::Cli),

    #[command(visible_alias = "i", aliases = ["im", "imp", "impo", "impor"])]
    Import(profiles::cli::import::Command),

    #[command(visible_alias = "jo", aliases = ["jou", "jour", "journ", "journa"])]
    Journal(journal::cli::Journal),

    #[command(visible_alias = "log")]
    Journalctl(journal::cli::Journalctl),

    #[command(visible_alias = "k",)]
    Key(privkey::Cli),

    #[command(visible_alias = "last", aliases = ["lj", "lastj"])]
    LastJournal(journal::cli::LastJournal),

    #[command(visible_alias = "n", aliases = ["nom", "nomi"])]
    Nomic(profiles::cli::nomic::Command),

    #[command(aliases = ["non", "nonc"])]
    Nonce(nonce::Cli),

    #[command(visible_alias = "p", aliases = ["pr", "pro", "prof", "profi", "profil", "profile"])]
    Profiles(profiles::cli::profiles::Command),

    #[command(visible_alias = "r",
        aliases = ["re", "red", "rede", "redel", "redele", "redeleg", "redelega", "redelegat"]
    )]
    Redelegate(profiles::cli::redelegate::Command),

    #[command(visible_alias = "se", aliases = ["sen"])]
    Send(profiles::cli::send::Command),

    #[command(visible_alias = "st", aliases = ["sta", "stat"])]
    Stats(profiles::cli::stats::Command),

    #[command(visible_alias = "v", 
        aliases = ["va", "val", "vali", "valid", "valida", "validat", "validato", "validator"]
    )]
    Validators(validators::Cli),

    Z(z::Cli),
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            Commands::Address(cmd)      => cmd.run(),
            Commands::AutoDelegate(cmd) => cmd.run(),
            Commands::Balance(cmd)      => cmd.run(),
            Commands::Claim(cmd)        => cmd.run(),
            Commands::Config(cli)       => cli.run(),
            Commands::Delegate(cmd)     => cmd.run(),
            Commands::Delegations(cmd)  => cmd.run(),
            Commands::Export(cmd)       => cmd.run(),
            Commands::Fmt(cli)          => cli.run(),
            Commands::GlobalConfig(cli) => cli.run(),
            Commands::Import(cmd)       => cmd.run(),
            Commands::Journal(cmd)      => cmd.run(),
            Commands::Key(cli)          => cli.run(),
            Commands::LastJournal(cmd)  => cmd.run(),
            Commands::Journalctl(cmd)   => cmd.run(),
            Commands::Nomic(cmd)        => cmd.run(),
            Commands::Nonce(cli)        => cli.run(),
            Commands::Profiles(cmd)     => cmd.run(),
            Commands::Redelegate(cmd)   => cmd.run(),
            Commands::Send(cmd)         => cmd.run(),
            Commands::Stats(cmd)        => cmd.run(),
            Commands::Validators(cli)   => cli.run(),
            Commands::Z(cli)            => cli.run(),
        }
    }
}
