
use clap::{Parser, Subcommand};
use crate::privkey::PrivKey;
use eyre::{eyre, Result};
use std::io::{self, Write};
use std::path::PathBuf;

/// Defines the CLI structure for the `privkey` command.
#[derive(Parser)]
#[command(about = "Manage PrivKey File")]
pub struct Cli {
    /// Subcommands for the nonce command
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Parser, Debug)]
pub struct StdinArgs {
    /// Read hex or binary from stdin
    #[arg(long, short)]
    pub stdin: bool,

    /// Maximum attempts to read from stdin (default: 5)
    #[arg(long, default_value = "5", requires = "stdin")]
    pub max_attempts: usize,

    /// Timeout for reading from stdin, in milliseconds (default: 500 ms)
    #[arg(long, default_value = "500", requires = "stdin")]
    pub timeout: u64,
}

#[derive(Parser, Debug)]
pub struct InputArgs {
    /// Hex key, Profile, Home or File
    pub input: Option<String>,

    /// Read hex or binary from stdin
    #[command(flatten)]
    pub stdin_args: StdinArgs,

}

/// Subcommands for the `privkey` command
#[derive(Subcommand)]
pub enum Command {
    /// Show the public address (AccountID)
    #[command(visible_alias = "ad", aliases = ["a", "add", "addr", "addre", "addres"])]
    Address {
        #[command(flatten)]
        input_args: InputArgs,
    },

    /// Export Private key caution
    Export {
        #[command(flatten)]
        input_args: InputArgs,
    },

    /// Save Private key to file
    #[command(visible_alias = "wr", aliases = ["w", "wri", "writ"])]
    Write {
        /// Hex key, Profile, Home or File
        #[arg(long, short, conflicts_with = "stdin")]
        input: Option<String>,

        /// Read hex or binary from stdin
        #[command(flatten)]
        stdin_args: StdinArgs,

        /// Profile, Home path or File path
        #[arg(conflicts_with = "stdout")]
        output: Option<PathBuf>,

        ///// Output to stdout
        //#[arg(long, short = 't', conflicts_with = "output")]
        //stdout: bool,

        /// Force overwrite
        #[arg(long, short = 'f')]
        force: bool,
    },
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        match &self.command {

            Some(Command::Address { input_args }) => {
                let privkey = if input_args.stdin_args.stdin {
                    PrivKey::stdin(
                        input_args.stdin_args.max_attempts, 
                        input_args.stdin_args.timeout,
                    )?
                } else {
                    match input_args.input.as_deref() {
                        Some(data) => {
                            PrivKey::import(data)?
                        },
                        None => {
                            PrivKey::new(None)
                        }
                    }
                };
                Ok(println!("{}", privkey.address()?))
            },

            Some(Command::Export { input_args }) => {
                let privkey = if input_args.stdin_args.stdin {
                    PrivKey::stdin(
                        input_args.stdin_args.max_attempts, 
                        input_args.stdin_args.timeout,
                    )?
                } else {
                    match input_args.input.as_deref() {
                        Some(data) => {
                            PrivKey::import(data)?
                        },
                        None => {
                            PrivKey::new(None)
                        }
                    }
                };
                Ok(println!("{}", privkey.export()))
            },

            Some(Command::Write { input, stdin_args, output, force }) => {
                let privkey = if stdin_args.stdin {
                    PrivKey::stdin(
                        stdin_args.max_attempts, 
                        stdin_args.timeout,
                    )?
                } else {
                    match input.as_deref() {
                        Some(data) => {
                            PrivKey::import(data)?
                        },
                        None => {
                            PrivKey::new(None)
                        }
                    }
                };

                match output.as_deref() {
                    Some(file) => {
                        return Ok(privkey.save(file, *force)?);
                    },
                    None => {
                        return Ok(io::stdout().write_all(privkey.bytes())?);
                    },
                }

            },

            None => Err(eyre!("No command provided")),
        }
    }
}
