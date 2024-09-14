/// Provides utilities for formatting text into columns with customizable options.
pub mod columnizer;

pub mod builder {
	#[allow(dead_code)]
	/// Builder for configuring and formatting text into columns.
	///
	/// This struct allows setting various options to control the formatting of text into columns,
	/// including field separators, header rows, divider lines, and text width.
	pub struct Builder<'a> {
		input: &'a str,			       // The text to be formatted
		ifs: &'a str,		       	   // Input Field Separator
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

	#[allow(dead_code)]
	impl<'a> Builder<'a> {
		/// Creates a new `Builder` instance with default settings.
		///
		/// # Arguments
		///
		/// * `input` - The input text to be formatted.
		///
		/// # Returns
		///
		/// A `Builder` instance with default values for all options.
		pub fn new(input: &'a str) -> Self {
			Builder {
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

		/// Sets the input field separator.
		///
		/// # Arguments
		///
		/// * `ifs` - The separator for fields in the input text.
		///
		/// # Returns
		///
		/// The `Builder` instance with the specified input field separator.
		pub fn ifs(mut self, ifs: &'a str) -> Self {
			self.ifs = ifs;
			self
		}

		/// Sets the output field separator.
		///
		/// # Arguments
		///
		/// * `ofs` - The separator for fields in the output text.
		///
		/// # Returns
		///
		/// The `Builder` instance with the specified output field separator.
		pub fn ofs(mut self, ofs: &'a str) -> Self {
			self.ofs = ofs;
			self
		}

		/// Sets the row number for the header.
		///
		/// # Arguments
		///
		/// * `row` - The row index that contains the header or 0 for no header.
		///
		/// # Returns
		///
		/// The `Builder` instance with the specified header row.
		pub fn header_row(mut self, row: usize) -> Self {
			self.header_row = row;
			self
		}

		/// Sets the row number containing maximum widths for columns.
		///
		/// # Arguments
		///
		/// * `row` - The row index containing the maximum width of each column or 0 to ignore.
		///
		/// # Returns
		///
		/// The `Builder` instance with the specified max width row.
		pub fn max_width_row(mut self, row: usize) -> Self {
			self.max_width_row = row;
			self
		}

		/// Sets the row number containing format strings for columns.
		///
		/// # Arguments
		///
		/// * `row` - The row index containing the format string for each column.
		///
		/// # Returns
		///
		/// The `Builder` instance with the specified format string row.
		pub fn format_string_row(mut self, row: usize) -> Self {
			self.format_string_row = row;
			self
		}

		/// Specifies whether to add a divider line after the header row.
		///
		/// # Arguments
		///
		/// * `add` - A boolean indicating whether to include a divider line.
		///
		/// # Returns
		///
		/// The `Builder` instance with the specified divider option.
		pub fn add_divider(mut self, add: bool) -> Self {
			self.add_divider = add;
			self
		}

		/// Sets the character used for the divider line.
		///
		/// # Arguments
		///
		/// * `divider_char` - The character to use as the divider.
		///
		/// # Returns
		///
		/// The `Builder` instance with the specified divider character.
		pub fn divider_char(mut self, divider_char: char) -> Self {
			self.divider_char = divider_char;
			self
		}

		/// Sets the maximum width of text columns.
		///
		/// # Arguments
		///
		/// * `max_text_width` - The maximum width of text fields.
		///
		/// # Returns
		///
		/// The `Builder` instance with the specified text width.
		pub fn max_text_width(mut self, max_text_width: usize) -> Self {
			self.max_text_width = max_text_width;
			self
		}

		/// Specifies whether to pad decimal digits in numeric columns.
		///
		/// # Arguments
		///
		/// * `pad_decimal_digits` - A boolean indicating whether to pad decimal digits.
		///
		/// # Returns
		///
		/// The `Builder` instance with the specified decimal padding option.
		pub fn pad_decimal_digits(mut self, pad_decimal_digits: bool) -> Self {
			self.pad_decimal_digits = pad_decimal_digits;
			self
		}

		/// Sets the maximum number of decimal places for numeric columns.
		///
		/// # Arguments
		///
		/// * `max_decimal_digits` - The maximum number of decimal places.
		///
		/// # Returns
		///
		/// The `Builder` instance with the specified decimal digits limit.
		pub fn max_decimal_digits(mut self, max_decimal_digits: usize) -> Self {
			self.max_decimal_digits = max_decimal_digits;
			self
		}

		/// Sets the character used as the decimal separator.
		///
		/// # Arguments
		///
		/// * `decimal_separator` - The character to use as the decimal separator.
		///
		/// # Returns
		///
		/// The `Builder` instance with the specified decimal separator.
		pub fn decimal_separator(mut self, decimal_separator: char) -> Self {
			self.decimal_separator = decimal_separator;
			self
		}

		/// Specifies whether to add a thousands separator to numbers.
		///
		/// # Arguments
		///
		/// * `add_thousand_separator` - A boolean indicating whether to add a thousands separator.
		///
		/// # Returns
		///
		/// The `Builder` instance with the specified thousands separator option.
		pub fn add_thousand_separator(mut self, add_thousand_separator: bool) -> Self {
			self.add_thousand_separator = add_thousand_separator;
			self
		}

		/// Sets the character used for the thousands separator.
		///
		/// # Arguments
		///
		/// * `thousand_separator` - The character to use as the thousands separator.
		///
		/// # Returns
		///
		/// The `Builder` instance with the specified thousands separator.
		pub fn thousand_separator(mut self, thousand_separator: char) -> Self {
			self.thousand_separator = thousand_separator;
			self
		}

		/// Formats the input text into columns based on the specified options.
		///
		/// # Returns
		///
		/// A `String` containing the formatted output.
		pub fn format(self) -> String {
			crate::columnizer::run (
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
}
