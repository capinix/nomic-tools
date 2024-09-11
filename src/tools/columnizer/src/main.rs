mod format_columns;
use clap::{Arg, ArgAction, Command};
use format_columns::format_columns;
use std::io::{self, Read};

fn main() {
    let matches = Command::new("columnizer")
        .version("1.0")
        .about("Formats text into columns with customizable options.")
        .long_about(
            "The `columnizer` tool takes input text and formats it into a neatly aligned columnar view. \
            You can specify the number of header rows, a divider line, and separators for fields in both \
            the input and output.",
        )
        .arg(
            Arg::new("input")
                .help("Input text to be formatted. If not provided, reads from stdin.")
                .index(1)
                .required(false)
                .value_parser(clap::value_parser!(String))
                .long_help(
                    "Provide input text directly to be formatted. \
                    If not provided, the tool will read input from standard input (stdin).",
                ),
        )
        .arg(
            Arg::new("ifs")
                .short('i')
                .long("ifs")
                .value_parser(clap::value_parser!(String))
                .default_value(" ")
                .help("Input field separator.")
                .long_help(
                    "Specify the character or string used to separate fields in the input text. \
                    The default separator is a space.",
                ),
        )
        .arg(
            Arg::new("ofs")
                .short('o')
                .long("ofs")
                .value_parser(clap::value_parser!(String))
                .default_value(" ")
                .help("Output field separator.")
                .long_help(
                    "Specify the character or string used to separate fields in the output text. \
                    The default separator is a space.",
                ),
        )
        .arg(
            Arg::new("header_row")
                .short('r')
                .long("header-row")
                .value_parser(clap::value_parser!(usize))
                .default_value("0")
                .help("The row number of the header or 0 for no header.")
                .long_help(
                    "Specify the row number that should be treated as the header. \
                    If you don't want a header, set this to 0. \
                    The header row will not be formatted with column alignment.",
                ),
        )
        .arg(
            Arg::new("max_width_row")
                .short('w')
                .long("max-width-row")
                .value_parser(clap::value_parser!(usize))
                .default_value("0")
                .help("The row number that contains the maximum width for each column.")
                .long_help(
                    "Specify the row number where each column's maximum width is defined. \
                    This row will be used to determine the column widths for formatting.",
                ),
        )
        .arg(
            Arg::new("format_string_row")
                .short('s')
                .long("format-string-row")
                .value_parser(clap::value_parser!(usize))
                .default_value("0")
                .help("The row number that contains a Rust format string for each column.")
                .long_help(
                    "Specify the row number where each column's format string is defined. \
                    This row will be used to determine the columns' format string for formatting.",
                ),
        )
        .arg(
            Arg::new("add_divider")
                .short('d')
                .long("add-divider")
                .action(ArgAction::SetTrue)
                .help("Whether to add a divider line after a possible header before any lines.")
                .long_help("If set, adds a divider line after the header row and before any other lines."),
        )
        .arg(
            Arg::new("divider_char")
                .short('c')
                .long("divider-char")
                .value_parser(clap::value_parser!(char))
                .default_value("-")
                .help("Character used for the divider line between columns.")
                .long_help(
                    "Set the character that will be used to draw the divider line between columns. \
                    The default character is a dash ('-').",
                ),
        )
        .arg(
            Arg::new("max_text_width")
                .long("max-text-width")
                .value_parser(clap::value_parser!(usize))
                .default_value("40")
                .help("Text fields will be trimmed to a max of this length")
                .long_help(
                    "Specify the maximum length of text fields. \
                    Text fields will be trimmed to fit this",
                ),
        )
        .arg(
            Arg::new("pad_decimal_digits")
                .short('p')
                .long("pad-decimal-digits")
                .action(ArgAction::SetTrue)
                .help("Whether to pad decimals in numeric columns.")
                .long_help(
                    "Set this flag to true if you want to pad the decimals for numeric columns. \
                    By default, this is set to false.",
                ),
        )
        .arg(
            Arg::new("max_decimal_digits")
                .short('m')
                .long("max-decimal-digits")
                .value_parser(clap::value_parser!(usize))
                .default_value("2")
                .help("Maximum number of decimal places to display for numeric columns.")
                .long_help(
                    "Specify the maximum number of decimal places to display for numeric columns. \
                    The default value is 2.",
                ),
        )
        .arg(
            Arg::new("decimal_separator")
                .short('e')
                .long("decimal-separator")
                .value_parser(clap::value_parser!(char))
                .default_value(".")
                .help("Decimal separator (e.g., '.' for English or ',' for French).")
                .long_help(
                    "Specify the character used as a decimal separator. \
                    The default value is '.'.",
                ),
        )
        .arg(
            Arg::new("add_thousand_separator")
                .short('t')
                .long("add-thousand-separator")
                .action(ArgAction::SetTrue)
                .help("If set, adds a thousands separator to numbers.")
                .long_help("If set, adds a thousands separator to numbers."),
        )
        .arg(
            Arg::new("thousand_separator")
                .short('u')
                .long("thousand-separator")
                .value_parser(clap::value_parser!(char))
                .default_value(",")
                .help("Thousands separator (e.g., ',' for English or '.' for French).")
                .long_help(
                    "Specify the character used as a thousands separator. \
                    The default value is ','.",
                ),
        )
        .get_matches();

    // Argument retrievals are directly used since default values are already set by Clap.
    let ifs = matches.get_one::<String>("ifs").expect("default value is set").to_string();
    let ofs = matches.get_one::<String>("ofs").expect("default value is set").to_string();
    let header_row = *matches.get_one::<usize>("header_row").expect("default value is set");
    let max_width_row = *matches.get_one::<usize>("max_width_row").expect("default value is set");
    let format_string_row = *matches.get_one::<usize>("format_string_row").expect("default value is set");
    let add_divider = *matches.get_one::<bool>("add_divider").expect("default value is set");
    let divider_char = *matches.get_one::<char>("divider_char").expect("default value is set");
    let max_text_width = *matches.get_one::<usize>("max_text_width").expect("default value is set");
    let pad_decimal_digits = *matches.get_one::<bool>("pad_decimal_digits").expect("default value is set");
    let max_decimal_digits = *matches.get_one::<usize>("max_decimal_digits").expect("default value is set"); // Corrected
    let decimal_separator = *matches.get_one::<char>("decimal_separator").expect("default value is set");
    let add_thousand_separator = *matches.get_one::<bool>("add_thousand_separator").expect("default value is set");
    let thousand_separator = *matches.get_one::<char>("thousand_separator").expect("default value is set");

    // Read input text or stdin
    let input = if let Some(input_text) = matches.get_one::<String>("input") {
        input_text.to_string()
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).expect("Failed to read from stdin");
        buf
    };

    let formatted_output = format_columns(
        &input,
        &ifs,
        &ofs,
        header_row,
        max_width_row,
        format_string_row,
        add_divider,
        divider_char,
        max_text_width,
        pad_decimal_digits,
        max_decimal_digits,
        decimal_separator,
        add_thousand_separator,
        thousand_separator,
    );

    // Handle the formatted output, e.g., print it
    println!("{}", formatted_output);
}
