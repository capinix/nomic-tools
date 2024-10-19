
// use clap::Args;
use clap::Parser;
use clap::Subcommand;
use crate::nonce::Nonce;
use eyre::Result;

#[derive(Parser)]
#[command(
    name = "Nonce", 
    about = "Manage Nonce File",
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

#[derive(Parser, Debug)]
pub struct StdinArgs {
    /// Read decimal or binary from stdin
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
    /// decimal, Profile, Home or File
    pub input: Option<String>,

    /// Read decimal or binary from stdin
    #[command(flatten)]
    pub stdin_args: StdinArgs,

}

/// Subcommands for the `nonce` command
#[derive(Subcommand)]
pub enum CliCommand {
    #[command(
        name = "read", 
        about = "Display decimal value of the contents of nonce file",
        visible_alias = "r",
    )]
    Read {
        #[command(flatten)]
        input_args: InputArgs,
    },

    #[command(
        name = "write", 
        about = "Write decimal value to nonce file",
        visible_alias = "w",
    )]
    Write {

        /// decimal, Profile, Home or File
        #[arg(long, short,)]
        input: Option<String>,

        /// Read decimal or binary from stdin
        #[command(flatten)]
        stdin_args: StdinArgs,

        /// Profile, Home or File
        #[arg()]
        output: Option<String>,

        /// Don't overwrite, if file exists.
        #[arg(long = "dont-overwrite", short = 'D',)]
        dont_overwrite: bool,

    },
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            Some(CliCommand::Read { input_args }) => {
                let nonce = if input_args.stdin_args.stdin {
                    Nonce::from_stdin(
                        input_args.stdin_args.max_attempts, 
                        input_args.stdin_args.timeout,
                    )?
                } else {
                    Nonce::from_input(input_args.input.as_deref(), None)?
                };
                println!("{}", nonce.decimal());
                Ok(())
            },
            Some(CliCommand::Write { input, stdin_args, output, dont_overwrite }) => {
                let nonce = if stdin_args.stdin {
                    Nonce::from_stdin(
                        stdin_args.max_attempts, 
                        stdin_args.timeout,
                    )?
                } else {
                    Nonce::from_input(input.as_deref(), None)?
                };

                nonce.to_output(output.as_deref(), *dont_overwrite)?;
                Ok(())
            },
            None => {
                Err(eyre::eyre!("No command provided."))
            }
        }
    }
}
