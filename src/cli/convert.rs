/// Defines the CLI structure for the `convert` command.
///
/// This module sets up the command-line interface for the `convert` command, which is responsible
/// for handling format conversions. The `convert` command includes subcommands for converting
/// between binary, decimal, and hex formats. Each subcommand routes to the appropriate handler
/// functions defined in the `handlers` module.
///
/// # Subcommands
///
/// - `binary`: Convert binary to/from other formats
///   - `to-decimal`: Convert binary to decimal
///   - `to-hex`: Convert binary to hex
///
/// - `decimal`: Convert decimal to/from other formats
///   - `to-binary`: Convert decimal to binary
///   - `to-hex`: Convert decimal to hex
///
/// - `hex`: Convert hex to/from other formats
///   - `to-binary`: Convert hex to binary
///   - `to-decimal`: Convert hex to decimal
///
/// Each subcommand is routed to the corresponding handler function in the `handlers` module.
use clap::Command;

pub fn cli() -> Command {
	Command::new("convert")
		.about("Convert between different formats")
		.subcommand(
			Command::new("binary")
				.about("Convert binary to/from other formats")
				.subcommand(
					Command::new("to-decimal")
						.about("Convert binary to decimal")
				)
				.subcommand(
					Command::new("to-hex")
						.about("Convert binary to hex")
				)
		)
		.subcommand(
			Command::new("decimal")
				.about("Convert decimal to/from other formats")
				.subcommand(
					Command::new("to-binary")
						.about("Convert decimal to binary")
				)
				.subcommand(
					Command::new("to-hex")
						.about("Convert decimal to hex")
				)
		)
		.subcommand(
			Command::new("hex")
				.about("Convert hex to/from other formats")
				.subcommand(
					Command::new("to-binary")
						.about("Convert hex to binary")
				)
				.subcommand(
					Command::new("to-decimal")
						.about("Convert hex to decimal")
				)
		)
}
