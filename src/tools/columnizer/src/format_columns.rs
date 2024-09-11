
/// Truncates a string to a maximum number of characters and appends an ellipsis if needed.
///
/// # Arguments
///
/// * `text` - The string to be truncated.
/// * `max_text_width` - The maximum number of characters allowed.
///
/// # Returns
///
/// A truncated string with an ellipsis if the original string is longer than `max_text_width`.

fn truncate_string(text: &str, max_text_width: usize) -> String {
	if max_text_width <= 3 {
	    return text.chars().take(max_text_width).collect::<String>();
	}
	if text.chars().count() > max_text_width {
	    let truncated = &text.chars().take(max_text_width - 3).collect::<String>(); // -3 for ellipsis
	    format!("{}...", truncated)
	} else {
	    text.to_string()
	}
}

/// Formats a numeric cell value and indicates whether it is numeric. 
/// Adds padding, thousands separators, and custom decimal separators as specified.
///
/// # Arguments
///
/// * `input` - The cell value to format.
/// * `max_text_width` - The maximum width allowed for the formatted value.
/// * `pad_decimal_digits` - A boolean indicating whether to pad decimal digits.
/// * `max_decimal_digits` - The maximum number of decimal digits (only used if `pad_decimal_digits` is true).
/// * `decimal_separator` - The character used as the decimal separator.
/// * `add_thousand_separator` - A boolean indicating whether to add a thousand separator.
/// * `thousand_separator` - The character used as the thousand separator.
///
/// # Returns
///
/// A tuple containing:
/// * A boolean indicating if the cell value is numeric.
/// * The formatted or truncated cell value as a string.

use std::num::ParseFloatError;

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

use std::collections::HashSet;

/// Processes the input data to create vectors for rows and determine numeric columns,
/// formatting each numeric cell based on the given parameters.
///
/// # Arguments
///
/// * `input` - The input text to be processed.
/// * `ifs` - The input field separator used to split columns.
/// * `header_row` - The row number of the header or 0 if there is no header.
/// * `max_width_row` - The row number that contains the maximum width for each column or 0 if not applicable.
/// * `format_string_row` - The row number that contains format strings for each column or 0 if not applicable.
/// * `max_text_width` - The maximum width allowed for text cells.
/// * `pad_decimal_digits` - A boolean indicating whether to pad decimal digits.
/// * `max_decimal_digits` - The maximum number of decimal digits (used only if `pad_decimal_digits` is true).
/// * `decimal_separator` - The character used as the decimal separator.
/// * `add_thousand_separator` - A boolean indicating whether to add a thousand separator.
/// * `thousand_separator` - The character used as the thousand separator.
///
/// # Returns
///
/// A tuple containing:
/// * A vector of vectors representing the formatted data rows.
/// * A vector representing the header row.
/// * A vector representing the maximum width row.
/// * A vector representing the format string row.
/// * A HashSet containing the indices of numeric columns.

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
    HashSet<usize>    // Numeric columns
) {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut numeric_columns = HashSet::new();
    let mut num_columns = 0;

    // Split input into lines
    let lines: Vec<&str> = input.lines().collect();

    // Process each line into columns using the custom field separator
    for line in lines {
        let columns: Vec<String> = line.split(ifs).map(|s| s.to_string()).collect();
        num_columns = num_columns.max(columns.len());
        rows.push(columns);
    }

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
            let (is_numeric, formatted_value) = format_content(
                col_value,
				max_text_width,
                pad_decimal_digits,
                max_decimal_digits,
                decimal_separator,
                add_thousand_separator,
                thousand_separator
            );

            if is_numeric {
                numeric_columns.insert(col_index);
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
    data: &[Vec<String>]
) -> Vec<usize> {
    // Determine the number of columns (assuming all rows have the same number of columns)
    let num_columns = header_data.len().max(data.iter().map(|row| row.len()).max().unwrap_or(0));

    // Initialize max widths for each column
    let mut max_widths = vec![0; num_columns];
    
    // Update max widths with header data
    for (i, header) in header_data.iter().enumerate() {
        if i < num_columns {
            max_widths[i] = header.chars().count();
        }
    }

    // Process data rows and update max widths
    for row in data {
        for (i, cell) in row.iter().enumerate() {
            // Apply truncation based on max_width_data if available
            let truncated_cell = if i < max_width_data.len() && max_width_data[i] > 0 {
                truncate_string(cell, max_width_data[i])
            } else {
                cell.clone()
            };
            let cell_width = truncated_cell.chars().count();
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
    numeric_columns: &HashSet<usize>,
    data: &[Vec<String>]
) -> String {

	println!("numeric_columns: {:?}", numeric_columns);

    let mut output = String::new();

// 	// Helper function to determine if a column is numeric
//	let is_numeric_column = |index: usize| numeric_columns.contains(&index);

    // Helper function to format a cell based on its type
    let format_cell = |cell: &str, width: usize, is_numeric: bool, format_string: &str| -> String {
        // Clean up the cell content

        let cell_cleaned = cell.trim();

		if !format_string.is_empty() {
			format_string.replace("{}", cell_cleaned)  // Use the user-provided format string if available
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
			let is_numeric = numeric_columns.contains(&i);  // Use HashSet's contains method
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
			let is_numeric = numeric_columns.contains(&i);  // Use HashSet's contains method
			let format_str = format_string_data.get(i).unwrap_or(&default_format_str);  // Safely access format string
            format_cell(cell, width, is_numeric, format_str)
        }).collect();
        output.push_str(&formatted_row.join(ofs));
        output.push('\n');
    }

    output
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

pub fn format_columns(
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

    let column_widths = calculate_max_column_widths(
		&header_data,
		&max_width_data,
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_truncate_string() {
	    assert_eq!(truncate_string("Hello, world!", 5), "He...");
	    assert_eq!(truncate_string("Hello", 5), "Hello");
	    assert_eq!(truncate_string("Hi", 5), "Hi");
	    assert_eq!(truncate_string("Hello, world!", 15), "Hello, world!");
	    assert_eq!(truncate_string("Hello", 2), "He");
	}

	#[test]
	fn test_format_content() {
	    // Test with numeric string and thousand separators
	    assert_eq!(
	        format_content("1234.56", 1, true, 2, '.', true, ','),
	        (true, "1,234.56".to_string())
	    );

		// Test with numeric string, no thousand separator, and different decimal separator
		let result = format_content("1234.56", 10, true, 2, ',', false, '.');
		let expected = (true, "1234,56".to_string());
		assert_eq!(
			result, 
			expected,
			"Failed on test with input '1234.56', max_width 10, pad_decimal_digits true, \
				max_decimal_digits 2, decimal_separator ',', add_thousand_separator false, \
				thousand_separator '.'. Got: {:?}, Expected: {:?}",
			result,
			expected
		);

        // Test with non-numeric string that requires truncation
        assert_eq!(
            format_content("Hello, world!", 5, true, 2, '.', true, ','),
            (false, "He...".to_string())
        );

        // Test with non-numeric string that fits within the limit
        assert_eq!(
            format_content("Hi", 5, false, 0, '.', false, ','),
            (false, "Hi".to_string())
        );

        // Test padding of decimal digits
        assert_eq!(
            format_content("1234.5", 10, true, 3, '.', true, ','),
            (true, "1,234.500".to_string())
        );
    }

    #[test]
    fn test_process_data() {
		let input = r#"
col1 col2 col3 col4
20.5674,sugar,50,babies
1,biscuit,200,kilimangaro
"#;
        let ifs = " ";
        let (_rows, header_data, _max_width_data, _format_string_data, _numeric_columns) = process_data(
            input,
            ifs,
            1,
            0,
            0,
            8,
            true,
            2,
            '.',
            true,
            ',',
        );

		// Print debug information
		use std::io::Write;
		println!("header_data: {:?}", header_data);
		println!("Expected: {:?}", vec!["col1".to_string(), "col2".to_string(), "col3".to_string(), "col4".to_string()]);
		std::io::stdout().flush().unwrap();


        assert_eq!(header_data, vec!["col1".to_string(), "col2".to_string(), "col3".to_string(), "col4".to_string()]);
//         assert_eq!(rows[0], vec!["20.56".to_string(), "sugar".to_string(),   "50.00".to_string(),  "babies".to_string()]);
//         assert_eq!(rows[1], vec![ "1.56".to_string(), "biscuit".to_string(), "200.00".to_string(), "kilimang".to_string()]);
//         assert_eq!(rows[2], vec![ "4,444.00".to_string(), "training".to_string(), "6,546,757.30".to_string(), "twenty".to_string()]);
//         assert!(numeric_columns.contains(&1)); // 'Value' column is numeric
// 
//         // Test with non-numeric columns only
//         let input = "Name,Description\nA,Test\nB,Example";
//         let (rows, header, _, _, numeric_columns) = process_data(
//             input,
//             ifs,
//             1,
//             0,
//             0,
//             10,
//             true,
//             2,
//             '.',
//             true,
//             ',',
//         );
// 
//         assert!(numeric_columns.is_empty());
    }
// 
//     #[test]
//     fn test_calculate_max_column_widths() {
//         let header_data = vec!["Name".to_string(), "Value".to_string()];
//         let max_width_data = vec![5, 8];
//         let data = vec![
//             vec!["A".to_string(), "1234.56".to_string()],
//             vec!["B".to_string(), "7890.12".to_string()],
//         ];
// 
//         let max_widths = calculate_max_column_widths(&header_data, &max_width_data, &data);
//         assert_eq!(max_widths, vec![4, 8]); // 'Name' has max length of 4, 'Value' has 8
//     }
// 
//     #[test]
//     fn test_format_columns() {
//         let input = "Name,Value\nA,1234.56\nB,7890.12";
//         let ifs = ",";
//         let ofs = " | ";
//         let formatted = format_columns(
//             input,
//             ifs,
//             ofs,
//             1,
//             0,
//             0,
//             true,
//             '-',
//             10,
//             true,
//             2,
//             '.',
//             true,
//             ',',
//         );
// 
//         let expected = "Name | Value\n------------\nA    | 1,234.56\nB    | 7,890.12";
//         assert_eq!(formatted.trim(), expected.trim());
// 
//         // Test without divider
//         let formatted_no_divider = format_columns(
//             input,
//             ifs,
//             ofs,
//             1,
//             0,
//             0,
//             false,
//             '-',
//             10,
//             true,
//             2,
//             '.',
//             true,
//             ',',
//         );
// 
//         let expected_no_divider = "Name | Value\nA    | 1,234.56\nB    | 7,890.12";
//         assert_eq!(formatted_no_divider.trim(), expected_no_divider.trim());
//     }
}
