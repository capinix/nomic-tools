

/// Function to clean a field by removing non-graphic characters and whitespace
fn clean_field(field: &str) -> String {
	field.chars()
		 .filter(|&c| c.is_ascii_graphic() || c.is_whitespace())
		 .collect::<String>()
		 .trim()
		 .to_string()
}
