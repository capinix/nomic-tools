use std::num::ParseFloatError;
use unicode_width::UnicodeWidthStr;
use unicode_width::UnicodeWidthChar;

/// Truncates a string to a maximum number of characters and appends an ellipsis if needed.
///
/// This function ensures that the resulting string does not exceed the specified maximum width.
/// If the original string is longer than `max_text_width`, it is truncated and an ellipsis ("...") is appended.
///
/// # Arguments
///
/// * `text` - The string to be truncated. This is the text that will be shortened if necessary.
/// * `max_text_width` - The maximum number of characters allowed in the output string. 
/// If `max_text_width` is less than or equal to 3, the function will truncate to that length without appending an ellipsis.
///
/// # Returns
///
/// A `String` that contains the truncated version of `text`, appended with "..."
/// if the original string was longer than `max_text_width`. 
/// If `text` is shorter than or equal to `max_text_width`, it is returned unchanged.
fn truncate_string(text: &str, max_text_width: usize) -> String {
    if max_text_width <= 3 {
        // Ensure that truncation length is not negative and avoid slicing invalid indices
        return text.chars().take(max_text_width).collect::<String>();
    }

    let mut width = 0;
    let mut chars = text.chars().peekable();
    let mut truncated = String::new();

    while let Some(c) = chars.next() {
        width += c.width().unwrap_or(0);
        if width > max_text_width - 3 {
            break;
        }
        truncated.push(c);
    }

    if width > max_text_width {
        // Add ellipsis and trim the resulting string
        truncated = format!("{}...", truncated.trim());
    } else {
        truncated = truncated.trim().to_string();
    }

    truncated
}

/// Formats a numeric cell value and indicates whether it is numeric. 
/// This function adds padding, thousands separators, and custom decimal separators as specified, 
/// and returns the formatted value along with a boolean indicating if it was successfully parsed as a number.
///
/// # Arguments
///
/// * `input` - The cell value to format, which is expected to be a string representation of a number.
/// * `max_text_width` - The maximum width allowed for the formatted value. This determines if the value should be truncated.
/// * `pad_decimal_digits` - A boolean indicating whether to pad decimal digits to ensure a consistent number of decimal places.
/// * `max_decimal_digits` - The maximum number of decimal digits to display, used only if `pad_decimal_digits` is true.
/// * `decimal_separator` - The character to use as the decimal separator (e.g., '.' or ',').
/// * `add_thousand_separator` - A boolean indicating whether to include a thousand separator in the integer part of the number.
/// * `thousand_separator` - The character to use as the thousand separator (e.g., ',' or ' ').
///
/// # Returns
///
/// A tuple containing:
/// * A boolean indicating whether the cell value was successfully parsed as a numeric value.
/// * The formatted value as a `String`. If parsing fails, the original `input` string is returned 
///   truncated to `max_text_width`, or as-is if `max_text_width` is zero.
fn format_content(
	input: &str,
	max_text_width: usize,
	pad_decimal_digits: bool,
	max_decimal_digits: usize,
	decimal_separator: char,
	add_thousand_separator: bool,
	thousand_separator: char
) -> (bool, String) {
	// Normalize input by replacing custom separators
	let normalized_input = input
	    .replace(thousand_separator, ",") // Remove custom thousands separators
	    .replace(decimal_separator, "."); // Convert custom decimal separator to standard '.'

	// Attempt to parse the normalized input as a number
	let result: Result<f64, ParseFloatError> = normalized_input.parse();

	match result {
	    Ok(number) => {
	        // Format the number with native Rust formatting
	        let formatted = if pad_decimal_digits {
	            format!(
	                "{:.*}",
	                max_decimal_digits,
	                number
	            )
	        } else {
	            format!("{}", number)
	        };

	        // Split formatted number into integer and fractional parts
	        let parts: Vec<&str> = formatted.split('.').collect();
	        let integer_part = parts[0];
	        let fractional_part = if parts.len() > 1 { parts[1] } else { "" };

            // Apply thousands separators if needed
            let integer_with_thousands = if add_thousand_separator {
                let integer_chars: Vec<char> = integer_part.chars().rev().collect();
                let mut result_chars = Vec::new();
                for (i, ch) in integer_chars.iter().enumerate() {
                    if i > 0 && i % 3 == 0 {
                        result_chars.push(thousand_separator);
                    }
                    result_chars.push(*ch);
                }
                result_chars.reverse();
                result_chars.iter().collect::<String>()
            } else {
                integer_part.to_string()
            };

	        // Combine integer and fractional parts and replace decimal separator
	        let formatted_number = if !fractional_part.is_empty() {
	            format!("{}.{}", integer_with_thousands, fractional_part)
	        } else {
	            integer_with_thousands
	        };

	        // Replace decimal separator with custom one
	        let final_formatted_number = formatted_number.replace('.', &decimal_separator.to_string());

	        (true, final_formatted_number) // Return true and formatted string
	    }
        Err(_) => {
            if max_text_width > 0 {
                (false, truncate_string(input, max_text_width))
            } else {
                (false, input.to_string())
            }
        }
	}
}

/// Processes the input data to create vectors for rows and determine numeric columns,
/// formatting each numeric cell based on the given parameters.
///
/// This function parses the input text into rows and columns, extracts special rows (header, max width, and format strings),
/// and formats each cell according to its type (numeric or text) using the provided formatting options.
///
/// # Arguments
///
/// * `input` - The input text to be processed, which should be formatted as rows of columns separated by `ifs`.
/// * `ifs` - The input field separator used to split columns in the input text.
/// * `header_row` - The row number of the header (1-based index) or 0 if there is no header.
/// * `max_width_row` - The row number (1-based index) that contains the maximum width for each column or 0 if not applicable.
/// * `format_string_row` - The row number (1-based index) that contains format strings for each column or 0 if not applicable.
/// * `max_text_width` - The maximum width allowed for text cells before truncating.
/// * `pad_decimal_digits` - A boolean indicating whether to pad decimal digits to a consistent length.
/// * `max_decimal_digits` - The maximum number of decimal digits to display, used only if `pad_decimal_digits` is true.
/// * `decimal_separator` - The character used as the decimal separator (e.g., '.' or ',').
/// * `add_thousand_separator` - A boolean indicating whether to include a thousand separator in numeric values.
/// * `thousand_separator` - The character to use as the thousand separator (e.g., ',' or ' ').
///
/// # Returns
///
/// A tuple containing:
/// * A vector of vectors representing the formatted data rows.
/// * A vector representing the header row.
/// * A vector representing the maximum width row.
/// * A vector representing the format string row.
/// * A vector of booleans indicating which columns are numeric.
fn process_data(
    input: &str,
	ifs: &str,
    header_row: usize,
    max_width_row: usize,
    format_string_row: usize,
	max_text_width: usize,
    pad_decimal_digits: bool,
    max_decimal_digits: usize,
    decimal_separator: char,
    add_thousand_separator: bool,
    thousand_separator: char,
) -> (
    Vec<Vec<String>>, // Formatted data rows
    Vec<String>,      // Header row
    Vec<usize>,       // Max width row
    Vec<String>,      // Format string row
    Vec<bool>         // Numeric columns
) {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut num_columns = 0;
	let trimmed: &str = input.trim_start().trim_end();

    // Split input into lines
    let lines: Vec<&str> = trimmed.lines().collect();

    // Process each line into columns using the custom field separator
    for line in lines {
        let columns: Vec<String> = line.split(ifs).map(|s| s.to_string()).collect();
        num_columns = num_columns.max(columns.len());
        rows.push(columns);
    }
    let mut numeric_columns = vec![true; num_columns];

    // Extract special rows if they exist
    let header_row_data = if header_row > 0 && header_row <= rows.len() {
        rows[header_row - 1].clone()
    } else {
        vec!["".to_string(); num_columns]
    };

    let max_width_row_data = if max_width_row > 0 && max_width_row <= rows.len() {
        rows[max_width_row - 1]
            .iter()
            .map(|s| s.parse::<usize>().unwrap_or(0)) // Parse widths as usize
            .collect()
    } else {
        vec![0; num_columns]
    };

    let format_string_row_data = if format_string_row > 0 && format_string_row <= rows.len() {
        rows[format_string_row - 1].clone()
    } else {
        vec!["".to_string(); num_columns]
    };

// 	// Debug print to check special rows
// 	println!("Header Row Data: {:?}", header_row_data);
// 	println!("Max Width Row Data: {:?}", max_width_row_data);
// 	println!("Format String Row Data: {:?}", format_string_row_data);

    // Filter out special rows from data rows
    let mut data_rows: Vec<Vec<String>> = rows.into_iter()
        .enumerate()
        .filter_map(|(i, row)| {
			// Check if the row index matches any of the special rows
			let is_special_row = (header_row != 0 && i == header_row - 1) ||
								 (max_width_row != 0 && i == max_width_row - 1) ||
								 (format_string_row != 0 && i == format_string_row - 1);

			// Only include rows that are not special rows
			if !is_special_row {
				Some(row)
			} else {
				None
			}
        })
        .collect();

// 	 // Debug print to check special rows
// 	 println!("Data rows: {:?}", data_rows);

    // Determine numeric columns and format data rows
    for (_i, row) in data_rows.iter_mut().enumerate() {
        for (col_index, col_value) in row.iter_mut().enumerate() {
			let min_max_text_width = if max_width_row_data[col_index] > 0 {
				max_text_width.min(max_width_row_data[col_index])
			} else {
				max_text_width
			};
            let (is_numeric, formatted_value) = format_content(
                col_value,
				min_max_text_width,
                pad_decimal_digits,
                max_decimal_digits,
                decimal_separator,
                add_thousand_separator,
                thousand_separator
            );

            if !is_numeric {
                numeric_columns[col_index] = false;
            }
            
            *col_value = formatted_value;
        }
    }

    (data_rows, header_row_data, max_width_row_data, format_string_row_data, numeric_columns)
}

/// Calculates the maximum column widths for each column, including the header,
/// and truncates text columns based on `max_width_data`.
///
/// # Arguments
///
/// * `header_data` - A vector of strings representing the header row.
/// * `max_width_data` - A vector of usize values representing the maximum width for each column.
/// * `data` - A vector of vectors, where each vector represents a data row.
///
/// # Returns
///
/// A vector of usize values where each value represents the maximum width of the corresponding column.
fn calculate_max_column_widths(
    header_data: &[String],
    max_width_data: &[usize],
    numeric_columns: &Vec<bool>,
    data: &[Vec<String>]
) -> Vec<usize> {
    // Determine the number of columns (assuming all rows have the same number of columns)
    let num_columns = header_data.len().max(data.iter().map(|row| row.len()).max().unwrap_or(0));

    // Initialize max widths for each column
    let mut max_widths = vec![0; num_columns];
    
    // Update max widths with header data
    for (i, header) in header_data.iter().enumerate() {
        if i < num_columns {
            max_widths[i] = header.width();
// 			if max_width_data[i] > 0 && max_widths[i] > max_width_data[i] {
// 				max_widths[i] = max_width_data[i];
// 			}
        }
    }

	
    // Process data rows and update max widths
    for row in data {
        for (i, cell) in row.iter().enumerate() {
            // Apply truncation based on max_width_data if available
            let truncated_cell = if i < max_width_data.len() && max_width_data[i] > 0 && !numeric_columns[i] {
                truncate_string(cell, max_width_data[i])
            } else {
                cell.clone().trim().to_string()
            };
            let cell_width = truncated_cell.width();
            if i < max_widths.len() && cell_width > max_widths[i] {
                max_widths[i] = cell_width;
            }
        }
    }

    max_widths
}

fn generate_output(
	ofs: &str,
	header_row: &usize,
    add_divider: bool,
    divider_char: char,
    header_data: &[String],
    format_string_data: &[String],
    column_widths: &[usize],
    numeric_columns: &Vec<bool>,
    data: &[Vec<String>]
) -> String {

// 	println!("numeric_columns: {:?}", numeric_columns);

    let mut output = String::new();

    // Helper function to format a cell based on its type
    let format_cell = |cell: &str, width: usize, is_numeric: bool, format_string: &str| -> String {
        // Clean up the cell content

//         let cell_cleaned = cell.trim();
		let cell_cleaned = if !is_numeric {
			truncate_string(cell, width)
		} else {
			cell.trim().to_string()
		};

		if !format_string.is_empty() {
			format_string.replace("{}", &cell_cleaned)  // Use the user-provided format string if available
		} else {
			// Default formatting
			if is_numeric {
				format!("{:>width$}", cell_cleaned, width = width) // Right-align numeric cells
			} else {
				format!("{:<width$}", cell_cleaned, width = width) // Left-align text cells
			}
		}
    };

    // Create a default empty string for the format string
    let default_format_str = String::new();


    // Header
    
	if header_row > &0 {
		let row: Vec<String> = header_data.iter().enumerate().map(|(i, cell)| {
			let width = *column_widths.get(i).unwrap_or(&0);
			let is_numeric = numeric_columns[i];  // Use HashSet's contains method
			let format_str = format_string_data.get(i).unwrap_or(&default_format_str);  // Safely access format string
			format_cell(cell, width, is_numeric, format_str)
		}).collect();
		output.push_str(&row.join(ofs));
		output.push('\n');
	}

    // Add divider if needed
    if add_divider {
        let divider: String = column_widths.iter()
            .map(|width| divider_char.to_string().repeat(*width))
            .collect::<Vec<String>>()
            .join(ofs);
        output.push_str(&divider);
        output.push('\n');
    }


    // Data Rows
    for row in data {
        let formatted_row: Vec<String> = row.iter().enumerate().map(|(i, cell)| {
            let width = *column_widths.get(i).unwrap_or(&0);
			let is_numeric = numeric_columns[i];  // Use HashSet's contains method
			let format_str = format_string_data.get(i).unwrap_or(&default_format_str);  // Safely access format string
            format_cell(cell, width, is_numeric, format_str)
        }).collect();
        output.push_str(&formatted_row.join(ofs));
        output.push('\n');
    }

    output.trim_end().to_string()
}

/// Formats a block of text into aligned columns based on various formatting parameters.
///
/// # Arguments
///
/// * `input` - The text to be formatted.
/// * `ifs` - Input Field Separator, the character or string used to separate input fields.
/// * `ofs` - Output Field Separator, the character or string used to separate output fields.
/// * `header_row` - The row number of the header or 0 for no header.
/// * `max_width_row` - A row containing the maximum widths for each column or 0 if not provided.
/// * `format_string_row` - A row containing format strings for each column or 0 if not provided.
/// * `add_divider` - A boolean indicating whether to include a divider line before the data.
/// * `divider_char` - The character used for the divider line.
/// * `max_text_width` - The maximum width for text columns before truncating.
/// * `pad_decimal_digits` - A boolean indicating whether to pad decimal digits with zeros.
/// * `max_decimal_digits` - The maximum number of decimal places to display.
/// * `decimal_separator` - The character used to separate decimal places (default is '.').
/// * `add_thousand_separator` - A boolean indicating whether to add a thousand separator to numeric values.
/// * `thousand_separator` - The character used as the thousand separator (default is ',').
///
/// # Returns
///
/// A formatted string with the input data aligned into columns according to the specified parameters.

pub fn run(
	input:                   &str,  // The text to be formatted                                                  
	ifs:                     &str,  // Input Field Separator                                                                       
	ofs:                     &str,  // Output Field Separator                                                                       
	header_row:              usize, // Which row is the header or 0 for no header           
	max_width_row:           usize, // A row containing max widths of each column or 0 not to bother
	format_string_row:       usize, // A row containing rust format string  of each column or 0 not to bother
	add_divider:             bool,  // Whether to include a divider line before the data                   
	divider_char:            char,  // Divider Character                                                                             
	max_text_width:          usize, // Maximum width for text columns                                                                           
	pad_decimal_digits:      bool,  // Do we align the decimals padding with 0 at the end if necessary
	max_decimal_digits:      usize, // Limit the number of decimal places                                          
	decimal_separator:       char,  // character seperating decimals, defauts to "."                                             
	add_thousand_separator:  bool,  // Add thousands seperator if set                                                        
	thousand_separator:      char   // character seperating thousands, defauts to ","                                             
) -> String {

    // Call process_data to get the rows and numeric columns
    let (data, header_data, max_width_data, format_string_data, numeric_columns) = process_data(
        input, 
		ifs,
		header_row,
		max_width_row,
		format_string_row,
		max_text_width,
		pad_decimal_digits,
		max_decimal_digits,
		decimal_separator,
		add_thousand_separator,
		thousand_separator
	);

// 	println!("numeric_columns: {:?}", numeric_columns);

    let column_widths = calculate_max_column_widths(
		&header_data,
		&max_width_data,
		&numeric_columns,
		&data);

    let output = generate_output(
		&ofs,
		&header_row,
		add_divider,
		divider_char,
		&header_data,
		&format_string_data,
		&column_widths,
		&numeric_columns,
		&data
	);

    output
}


