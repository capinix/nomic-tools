// src/cli.rs
use clap::{Arg, Command};

pub fn parse_args() -> clap::ArgMatches {
    Command::new("nomic-tools")
        .version("1.0")
        .author("capinix")
        .about("My nomic utilities")
        .subcommand(
            Command::new("status")
                .about("Executes command1")
                .arg(
                    Arg::new("option1")
                        .long("option1")
                        .takes_value(true)
                        .help("Specifies the value for option1"),
                )
                .arg(
                    Arg::new("option2")
                        .long("option2")
                        .help("A boolean flag for option2"),
                ),
        )
        .subcommand(
            Command::new("journald")
                .about("Executes command2")
                .arg(
                    Arg::new("param1")
                        .long("param1")
                        .takes_value(true)
                        .help("Specifies the value for param1"),
                )
                .arg(
                    Arg::new("param2")
                        .long("param2")
                        .takes_value(true)
                        .help("Specifies the value for param2"),
                ),
        )
        .get_matches()
}

pub fn handle_command1(matches: &clap::ArgMatches) {
    let option1 = matches.get_one::<String>("option1").unwrap_or(&"".to_string());
    let option2 = matches.is_present("option2");

    println!("Executing command1 with options:");
    println!("Option1: {}", option1);
    println!("Option2: {}", option2);
}

pub fn handle_command2(matches: &clap::ArgMatches) {
    let param1 = matches.get_one::<String>("param1").unwrap_or(&"".to_string());
    let param2 = matches.get_one::<String>("param2").unwrap_or(&"".to_string());

    println!("Executing command2 with parameters:");
    println!("Param1: {}", param1);
    println!("Param2: {}", param2);
}
