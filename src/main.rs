mod cli;
mod globals;
mod nomic { pub mod validators; }
mod subcommand;
mod tools { pub mod columnizer; }

use subcommand::dispatch;

fn main() {

	// get command line arguments
    let matches = cli::build_cli().get_matches();

    // Delegate the subcommand handling to the `dispatch` function
    if let Err(e) = dispatch(&matches) { eprintln!("Error: {}", e); std::process::exit(1); }

}
