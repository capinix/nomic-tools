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

    pub fn separator(mut self, separator: &'a str) -> Self {
        self.separator = separator;
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
    let empty_string = String::new();
    let headers: Vec<String> = rows.get(0..header_count).unwrap_or(&[]).to_vec();
    let first_row = rows.get(header_count).unwrap_or(&empty_string);
    let num_cols = first_row.split(separator).count();

    // Determine the max width of each column
    let mut max_widths: Vec<usize> = vec![0; num_cols];
    let combined_rows: Vec<String> = rows.iter().chain(headers.iter()).map(|s| s.to_string()).collect();
    for row in &combined_rows {
        let fields: Vec<&str> = row.split(separator).collect();
        for (i, field) in fields.iter().enumerate() {
            max_widths[i] = max_widths[i].max(field.len());
        }
    }

    let mut output = String::new();

    // Initialize divider_line with an empty string
    let mut divider_line = String::new();

    // Add the header divider if needed
    if header_count > 0 && divider {
        divider_line = max_widths
            .iter()
            .map(|&w| divider_char.to_string().repeat(w + 2)) // Add padding
            .collect::<Vec<String>>()
            .join(&divider_char.to_string());

        output.push_str(&divider_line);
        output.push('\n');
    }

    // Add the headers
    for row in &headers {
        let fields: Vec<&str> = row.split(separator).collect();
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
    if let Some(rows_slice) = rows.get(header_count..) {
        for row in rows_slice {
            let fields: Vec<&str> = row.split(separator).collect();
            let formatted_fields: Vec<String> = fields
                .iter()
                .enumerate()
                .map(|(i, &field)| format!("{:<width$}", field, width = max_widths[i]))
                .collect();
            output.push_str(&formatted_fields.join(separator));
            output.push('\n');
        }
    }

    output
}

