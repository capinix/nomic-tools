use clap::{Arg, Command};

pub fn cli() -> Command {
    Command::new("validators")
        .about("Manage validators")
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .default_value("json-pretty")
                .value_parser(["raw", "table", "json", "json-pretty"])
                .help("Specify the output format")
        )
        .subcommand(
            Command::new("top")
                .about("Show the top N validators")
				.arg(
					Arg::new("format")
						.short('f')
						.long("format")
						.value_name("FORMAT")
						.value_parser(["raw", "table", "json", "json-pretty"])
						.help("Specify the output format")
				)
                .arg(
                    Arg::new("number")
                        .value_name("N")
                        .value_parser(clap::value_parser!(usize))
                        .help("Number of top validators to show")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("bottom")
                .about("Show the bottom N validators")
				.arg(
					Arg::new("format")
						.short('f')
						.long("format")
						.value_name("FORMAT")
						.value_parser(["raw", "table", "json", "json-pretty"])
						.help("Specify the output format")
				)
                .arg(
                    Arg::new("number")
                        .value_name("N")
                        .value_parser(clap::value_parser!(usize))
                        .help("Number of bottom validators to show")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("skip")
                .about("Skip the first N validators")
				.arg(
					Arg::new("format")
						.short('f')
						.long("format")
						.value_name("FORMAT")
						.value_parser(["raw", "table", "json", "json-pretty"])
						.help("Specify the output format")
				)
                .arg(
                    Arg::new("number")
                        .value_name("N")
                        .value_parser(clap::value_parser!(usize))
                        .help("Number of validators to skip")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("random")
                .about("Show a specified number of random validators outside a specified top percentage")
				.arg(
					Arg::new("format")
						.short('f')
						.long("format")
						.value_name("FORMAT")
						.value_parser(["raw", "table", "json", "json-pretty"])
						.help("Specify the output format")
				)
                .arg(
                    Arg::new("count")
						.short('c')
						.long("count")
                        .value_name("COUNT")
                        .value_parser(clap::value_parser!(usize))
                        .help("Number of random validators to show")
                        .required(true),
                )
                .arg(
                    Arg::new("percent")
						.short('p')
						.long("percent")
                        .value_name("PERCENT")
                        .value_parser(clap::value_parser!(u8))
                        .help("Percentage of validators to consider for randomness")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("moniker")
                .about("Search for validators by moniker")
				.arg(
					Arg::new("format")
						.short('f')
						.long("format")
						.value_name("FORMAT")
						.value_parser(["raw", "table", "json", "json-pretty"])
						.help("Specify the output format")
				)
                .arg(
                    Arg::new("moniker")
                        .value_name("MONIKER")
                        .help("Search for validators by moniker")
                        .required(true)
                        .value_parser(clap::value_parser!(String)),
                ),
        )
        .subcommand(
            Command::new("address")
                .about("Search for a validator by address")
				.arg(
					Arg::new("format")
						.short('f')
						.long("format")
						.value_name("FORMAT")
						.value_parser(["raw", "table", "json", "json-pretty"])
						.help("Specify the output format")
				)
                .arg(
                    Arg::new("address")
                        .value_name("ADDRESS")
                        .help("Search for a validator by its address")
                        .required(true)
                        .value_parser(clap::value_parser!(String)),
                ),
        )
}
