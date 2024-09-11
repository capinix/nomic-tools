use format_columns::format_columns;

pub struct ColumnFormatter<'a> {
	input: &'a str,			       // The text to be formatted
	ifs: &'a str,			       // Input Field Separator
	ofs: &'a str,			       // Output Field Separator
	header_row: usize,  	       // Whiich row is the header or 0 for no header
	max_width_row: usize, 		   // A row containing max widths of each column or 0 not to bother
	format_string_row: usize, 	   // A row containing a rust format string for each column
	add_divider: bool,             // Whether to include a divider line ----
	divider_char: char,            // Divider Character ----, ====, ####
	max_text_width: usize,         // Maximum width of text columns
    pad_decimal_digits: bool,      // Do we align the decimals padding with 0 at the end if neccessay
    max_decimal_digits: usize,     // Limit the number of decimal places
	decimal_separator: char,       // character to display decimals 0.0, 0,0
	add_thousand_separator: bool,  // do we add thousands seperator in output
	thousand_separator: char,      // seperator for thousands, 0,000, 0.000
}

impl<'a> ColumnFormatter<'a> {
	pub fn new(input: &'a str) -> Self {
		ColumnFormatter {
			input,
			ifs:                    " "   , // Default Input Field Separator
			ofs:                    " "   , // Default Output Field Separator
			header_row:             0     ,	// Default header row
			max_width_row:          0     ,	// Default max_width row
			format_string_row:      0     , // Default format string row
			add_divider:            false ,	// No divider by default
			divider_char:           '-'   ,	// Default Divider Character
			max_text_width:         40    , // Maximum width of text fields
			pad_decimal_digits:     false , // No padding
			max_decimal_digits:     2     , // Default 0.00
			decimal_separator:      '.'   , // seperate integers and decimals with dot 0.0
			add_thousand_separator: false , // no parsing 0,000
			thousand_separator:     ','     // seperate thousands with comma 0,000
		}
	}

	#[allow(dead_code)]
	pub fn ifs(mut self, separator: &'a str) -> Self {
		self.ifs = separator;
		self
	}

	#[allow(dead_code)]
	pub fn ofs(mut self, separator: &'a str) -> Self {
		self.ofs = separator;
		self
	}

	#[allow(dead_code)]
	pub fn header_row(mut self, row: usize) -> Self {
		self.header_row = row;
		self
	}

	#[allow(dead_code)]
	pub fn max_width_row(mut self, row: usize) -> Self {
		self.max_width_row = row;
		self
	}

	#[allow(dead_code)]
	pub fn format_string_row(mut self, row: usize) -> Self {
		self.format_string_row = row;
		self
	}

	#[allow(dead_code)]
	pub fn add_divider(mut self, add_divider: bool) -> Self {
		self.divider = add_divider;
		self
	}

	#[allow(dead_code)]
	pub fn divider_char(mut self, divider_char: char) -> Self {
		self.divider_char = divider_char;
		self
	}

	#[allow(dead_code)]
	pub fn max_text_width(mut self, max_text_width: char) -> Self {
		self.max_text_width = max_text_width;
		self
	}

	#[allow(dead_code)]
	pub fn pad_decimal_digits(mut self, pad_decimal_digits: bool) -> Self {
		self.pad_decimal_digits = pad_decimal_digits;
		self
	}

	#[allow(dead_code)]
	pub fn max_decimal_digits(mut self, max_decimal_digits: usize) -> Self {
		self.max_decimal_digits = max_decimal_digits;
		self
	}

	#[allow(dead_code)]
	pub fn decimal_separator(mut self, decimal_separator: char) -> Self {
		self.decimal_separator = decimal_separator;
		self
	}

	#[allow(dead_code)]
	pub fn add_thousand_separator(mut self, add_thousand_separator: bool) -> Self {
		self.add_thousand_separator = add_thousand_separator;
		self
	}

	#[allow(dead_code)]
	pub fn thousand_separator(mut self, thousand_separator: separator) -> Self {
		self.thousand_separator = thousand_separator;
		self
	}

	pub fn format(self) -> String {
		format_columns(
			self.input, 
			self.ifs, 
			self.ofs,
			self.header_row, 
			self.max_width_row, 
			self.format_string_row,
			self.add_divider, 
			self.divider_char, 
			self.max_text_width, 
			self.pad_decimal_digits, 
			self.max_decimal_digits, 
			self.decimal_separator, 
			self.add_thousand_separator, 
			self.thousand_separator, 
		)
	}
}

