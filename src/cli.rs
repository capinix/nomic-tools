use clap::{Arg, Command};

pub fn build_cli() -> Command {
	Command::new("nomic-tools")
		.version("0.1.0")
		.author("Your Name <your.email@example.com>")
		.about("Tools for working with Nomic")
		.subcommand(
			Command::new("validators")
				.about("Handle validators information")
				.arg(
					Arg::new("format")
						.short('f')
						.long("format")
						.value_name("FORMAT")
						.num_args(1)
						.default_value("json-pretty")
						.value_parser(["detail", "json", "json-pretty", "raw", "table", "tuple"])
						.help("Output format"),
				)
				.arg(
					Arg::new("validator")
						.short('v')
						.long("validator")
						.value_name("ADDRESS")
						.help("Filter output to a specific validator address")
						.value_parser(clap::value_parser!(String)),
				)
                .subcommand(
                    Command::new("search")
                        .about("Search for validators by moniker")
                        .arg(
                            Arg::new("moniker")
                                .short('m')
                                .long("moniker")
                                .value_name("MONIKER")
                                .help("Search for validators by moniker")
                                .required(true)
                                .value_parser(clap::value_parser!(String)),
                        ),
                ),

		)
}

