
pub struct ColumnFormatter<'a> {
    input: &'a str,
    header_count: usize,
    separator: &'a str,
    divider: bool,
    divider_char: char,
}

impl<'a> ColumnFormatter<'a> {
    pub fn new(input: &'a str) -> Self {
        ColumnFormatter {
            input,
            header_count: 0,
            separator: " ",
            divider: false,
            divider_char: '-',
        }
    }

    pub fn header_count(mut self, count: usize) -> Self {
        self.header_count = count;
        self
    }

    pub fn separator(mut self, separator: &'a str) -> Self {
        self.separator = separator;
        self
    }

    pub fn divider(mut self, divider: bool) -> Self {
        self.divider = divider;
        self
    }

    pub fn divider_char(mut self, char: char) -> Self {
        self.divider_char = char;
        self
    }

    pub fn format(self) -> String {
        format_columns(self.input, self.header_count, self.separator, self.divider, self.divider_char)
    }
}

fn format_columns(
    input: &str,
    header_count: usize,
    separator: &str,
    divider: bool,
    divider_char: char
) -> String {
    let rows: Vec<String> = input.lines().map(|s| s.to_string()).collect();
    let num_rows = rows.len();
    if num_rows == 0 {
        return String::new();
    }

    // Determine the number of columns based on the first row
    let headers: Vec<String> = rows.get(0..header_count).unwrap_or(&[]).to_vec();
    let first_row = rows.get(header_count).unwrap_or(&String::new());
    let num_cols = first_row.split(separator).count();

    // Determine the max width of each column
    let mut max_widths: Vec<usize> = vec![0; num_cols];
    let combined_rows: Vec<String> = rows.iter().chain(headers.iter()).map(|s| s.to_string()).collect();
    for row in &combined_rows {
        let fields: Vec<&str> = row.split(separator).collect();
        for (i, field) in fields.iter().enumerate() {
            let is_number = field.chars().all(|c| c.is_numeric() || c == ',' || c == '.');
            if is_number {
                max_widths[i] = max_widths[i].max(field.len());
            } else {
                max_widths[i] = max_widths[i].max(field.len());
            }
        }
    }

    let mut output = String::new();

	// Initialize divider_line with an empty string
	let mut divider_line = String::new();

    // Add the header divider if needed
    if header_count > 0 && divider {
    let divider_str = divider_char.to_string(); // Convert divider_char to String once

        divider_line = max_widths
            .iter()
            .map(|&w| divider_char.to_string().repeat(w + 2)) // Add padding
            .collect::<Vec<String>>()
        .join(&divider_str); // Use &divider_str here

        output.push_str(&divider_line);
        output.push('\n');
	}

    // Add the headers
    for row in &headers {
		let fields: Vec<&str> = row.as_str().split(separator).collect(); // Convert `row` to `&str`
// 		let fields: Vec<&str> = row.split(separator).collect();
        let formatted_fields: Vec<String> = fields
            .iter()
            .enumerate()
            .map(|(i, &field)| format!("{:<width$}", field, width = max_widths[i]))
            .collect();
        output.push_str(&formatted_fields.join(separator));
        output.push('\n');
    }

    // Add the header divider line if needed
    if header_count > 0 && divider {
        output.push_str(&divider_line);
        output.push('\n');
    }

    // Add the rest of the rows
    for row in rows.get(header_count..) {
        let fields: Vec<&str> = row.split(separator).collect();
        let formatted_fields: Vec<String> = fields
            .iter()
            .enumerate()
            .map(|(i, &field)| format!("{:<width$}", field, width = max_widths[i]))
            .collect();
        output.push_str(&formatted_fields.join(separator));
        output.push('\n');
    }

    output
}

fn get_max_column_widths(rows: Vec<&str>, separator: &str) -> Vec<usize> {
    let mut max_widths: Vec<usize> = Vec::new();

    for row in rows {
        let columns: Vec<&str> = row.split(separator).collect();
        for (i, column) in columns.iter().enumerate() {
            let width = column.len();
            if i >= max_widths.len() {
                max_widths.push(width);
            } else if width > max_widths[i] {
                max_widths[i] = width;
            }
        }
    }

    max_widths
}

fn format_row(row: &str, widths: &[usize], separator: &str) -> String {
    row.split(separator)
        .enumerate()
        .map(|(i, field)| format!("{:width$}", field, width = widths.get(i).unwrap_or(&0)))
        .collect::<Vec<_>>()
        .join(separator)
}
