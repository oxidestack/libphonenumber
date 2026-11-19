//! `Phonenumber.PhoneNumber` and `NumberParseException`.

use std::fmt;
use std::hash::{Hash, Hasher};

/// `PhoneNumber.CountryCodeSource`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CountryCodeSource {
    /// The calling code came from a number written with a leading `+`.
    FromNumberWithPlusSign,
    /// The calling code came after the region's international dialling prefix.
    FromNumberWithIdd,
    /// The calling code was at the start of the number, with no `+` or prefix.
    FromNumberWithoutPlusSign,
    /// The calling code is the default region's: the number was national.
    FromDefaultCountry,
    /// Not set.
    #[default]
    Unspecified,
}

impl CountryCodeSource {
    /// The Java constant's name.
    pub fn name(self) -> &'static str {
        match self {
            CountryCodeSource::FromNumberWithPlusSign => "FROM_NUMBER_WITH_PLUS_SIGN",
            CountryCodeSource::FromNumberWithIdd => "FROM_NUMBER_WITH_IDD",
            CountryCodeSource::FromNumberWithoutPlusSign => "FROM_NUMBER_WITHOUT_PLUS_SIGN",
            CountryCodeSource::FromDefaultCountry => "FROM_DEFAULT_COUNTRY",
            CountryCodeSource::Unspecified => "UNSPECIFIED",
        }
    }
}

impl fmt::Display for CountryCodeSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// `Phonenumber.PhoneNumber`: the fields with their `has…` flags, the proto-style setters and clearers.
///
/// Equality is `exactlySameAs`: unset fields equal their defaults, except the preferred domestic carrier code,
/// whose presence counts too.
#[derive(Debug, Clone)]
pub struct PhoneNumber {
    country_code: i32,
    has_country_code: bool,
    national_number: u64,
    has_national_number: bool,
    extension: String,
    has_extension: bool,
    italian_leading_zero: bool,
    has_italian_leading_zero: bool,
    number_of_leading_zeros: i32,
    has_number_of_leading_zeros: bool,
    raw_input: String,
    has_raw_input: bool,
    country_code_source: CountryCodeSource,
    has_country_code_source: bool,
    preferred_domestic_carrier_code: String,
    has_preferred_domestic_carrier_code: bool,
}

impl Default for PhoneNumber {
    fn default() -> Self {
        PhoneNumber {
            country_code: 0,
            has_country_code: false,
            national_number: 0,
            has_national_number: false,
            extension: String::new(),
            has_extension: false,
            italian_leading_zero: false,
            has_italian_leading_zero: false,
            number_of_leading_zeros: 1,
            has_number_of_leading_zeros: false,
            raw_input: String::new(),
            has_raw_input: false,
            country_code_source: CountryCodeSource::Unspecified,
            has_country_code_source: false,
            preferred_domestic_carrier_code: String::new(),
            has_preferred_domestic_carrier_code: false,
        }
    }
}

impl PhoneNumber {
    /// `new PhoneNumber()`.
    pub fn new() -> Self {
        PhoneNumber::default()
    }

    /// Whether `country_code` is set.
    pub fn has_country_code(&self) -> bool {
        self.has_country_code
    }
    /// The country calling code; the default when unset.
    pub fn country_code(&self) -> i32 {
        self.country_code
    }
    /// Sets `country_code`: the country calling code.
    pub fn set_country_code(&mut self, value: i32) -> &mut Self {
        self.has_country_code = true;
        self.country_code = value;
        self
    }
    /// Unsets `country_code`, back to its default.
    pub fn clear_country_code(&mut self) -> &mut Self {
        self.has_country_code = false;
        self.country_code = 0;
        self
    }

    /// Whether `national_number` is set.
    pub fn has_national_number(&self) -> bool {
        self.has_national_number
    }
    /// The national significant number; the default when unset.
    pub fn national_number(&self) -> u64 {
        self.national_number
    }
    /// Sets `national_number`: the national significant number.
    pub fn set_national_number(&mut self, value: u64) -> &mut Self {
        self.has_national_number = true;
        self.national_number = value;
        self
    }
    /// Unsets `national_number`, back to its default.
    pub fn clear_national_number(&mut self) -> &mut Self {
        self.has_national_number = false;
        self.national_number = 0;
        self
    }

    /// Whether `extension` is set.
    pub fn has_extension(&self) -> bool {
        self.has_extension
    }
    /// The extension; the default when unset.
    pub fn extension(&self) -> &str {
        &self.extension
    }
    /// Sets `extension`: the extension.
    pub fn set_extension(&mut self, value: impl Into<String>) -> &mut Self {
        self.has_extension = true;
        self.extension = value.into();
        self
    }
    /// Unsets `extension`, back to its default.
    pub fn clear_extension(&mut self) -> &mut Self {
        self.has_extension = false;
        self.extension.clear();
        self
    }

    /// Whether `italian_leading_zero` is set.
    pub fn has_italian_leading_zero(&self) -> bool {
        self.has_italian_leading_zero
    }
    /// Whether the national number has a leading zero (Italian numbers and others keep it); the default when unset.
    pub fn is_italian_leading_zero(&self) -> bool {
        self.italian_leading_zero
    }
    /// Sets `italian_leading_zero`: whether the national number has a leading zero (Italian numbers and others keep it).
    pub fn set_italian_leading_zero(&mut self, value: bool) -> &mut Self {
        self.has_italian_leading_zero = true;
        self.italian_leading_zero = value;
        self
    }
    /// Unsets `italian_leading_zero`, back to its default.
    pub fn clear_italian_leading_zero(&mut self) -> &mut Self {
        self.has_italian_leading_zero = false;
        self.italian_leading_zero = false;
        self
    }

    /// Whether `number_of_leading_zeros` is set.
    pub fn has_number_of_leading_zeros(&self) -> bool {
        self.has_number_of_leading_zeros
    }
    /// How many leading zeros the national number has, when `italian_leading_zero` is set; the default when unset.
    pub fn number_of_leading_zeros(&self) -> i32 {
        self.number_of_leading_zeros
    }
    /// Sets `number_of_leading_zeros`: how many leading zeros the national number has, when `italian_leading_zero` is set.
    pub fn set_number_of_leading_zeros(&mut self, value: i32) -> &mut Self {
        self.has_number_of_leading_zeros = true;
        self.number_of_leading_zeros = value;
        self
    }
    /// Unsets `number_of_leading_zeros`, back to its default.
    pub fn clear_number_of_leading_zeros(&mut self) -> &mut Self {
        self.has_number_of_leading_zeros = false;
        self.number_of_leading_zeros = 1;
        self
    }

    /// Whether `raw_input` is set.
    pub fn has_raw_input(&self) -> bool {
        self.has_raw_input
    }
    /// The input as it was given to `parse_and_keep_raw_input`; the default when unset.
    pub fn raw_input(&self) -> &str {
        &self.raw_input
    }
    /// Sets `raw_input`: the input as it was given to `parse_and_keep_raw_input`.
    pub fn set_raw_input(&mut self, value: impl Into<String>) -> &mut Self {
        self.has_raw_input = true;
        self.raw_input = value.into();
        self
    }
    /// Unsets `raw_input`, back to its default.
    pub fn clear_raw_input(&mut self) -> &mut Self {
        self.has_raw_input = false;
        self.raw_input.clear();
        self
    }

    /// Whether `country_code_source` is set.
    pub fn has_country_code_source(&self) -> bool {
        self.has_country_code_source
    }
    /// Where the calling code came from; the default when unset.
    pub fn country_code_source(&self) -> CountryCodeSource {
        self.country_code_source
    }
    /// Sets `country_code_source`: where the calling code came from.
    pub fn set_country_code_source(&mut self, value: CountryCodeSource) -> &mut Self {
        self.has_country_code_source = true;
        self.country_code_source = value;
        self
    }
    /// Unsets `country_code_source`, back to its default.
    pub fn clear_country_code_source(&mut self) -> &mut Self {
        self.has_country_code_source = false;
        self.country_code_source = CountryCodeSource::Unspecified;
        self
    }

    /// Whether `preferred_domestic_carrier_code` is set.
    pub fn has_preferred_domestic_carrier_code(&self) -> bool {
        self.has_preferred_domestic_carrier_code
    }
    /// The carrier code to dial nationally, when one was in the input; the default when unset.
    pub fn preferred_domestic_carrier_code(&self) -> &str {
        &self.preferred_domestic_carrier_code
    }
    /// Sets `preferred_domestic_carrier_code`: the carrier code to dial nationally, when one was in the input.
    pub fn set_preferred_domestic_carrier_code(&mut self, value: impl Into<String>) -> &mut Self {
        self.has_preferred_domestic_carrier_code = true;
        self.preferred_domestic_carrier_code = value.into();
        self
    }
    /// Unsets `preferred_domestic_carrier_code`, back to its default.
    pub fn clear_preferred_domestic_carrier_code(&mut self) -> &mut Self {
        self.has_preferred_domestic_carrier_code = false;
        self.preferred_domestic_carrier_code.clear();
        self
    }

    /// `clear()`.
    pub fn clear(&mut self) -> &mut Self {
        *self = PhoneNumber::default();
        self
    }

    /// `mergeFrom(other)`: copies every field `other` has set.
    pub fn merge_from(&mut self, other: &PhoneNumber) -> &mut Self {
        if other.has_country_code {
            self.set_country_code(other.country_code);
        }
        if other.has_national_number {
            self.set_national_number(other.national_number);
        }
        if other.has_extension {
            self.set_extension(other.extension.clone());
        }
        if other.has_italian_leading_zero {
            self.set_italian_leading_zero(other.italian_leading_zero);
        }
        if other.has_number_of_leading_zeros {
            self.set_number_of_leading_zeros(other.number_of_leading_zeros);
        }
        if other.has_raw_input {
            self.set_raw_input(other.raw_input.clone());
        }
        if other.has_country_code_source {
            self.set_country_code_source(other.country_code_source);
        }
        if other.has_preferred_domestic_carrier_code {
            self.set_preferred_domestic_carrier_code(other.preferred_domestic_carrier_code.clone());
        }
        self
    }

    /// `exactlySameAs(other)`.
    pub fn exactly_same_as(&self, other: &PhoneNumber) -> bool {
        self.country_code == other.country_code
            && self.national_number == other.national_number
            && self.extension == other.extension
            && self.italian_leading_zero == other.italian_leading_zero
            && self.number_of_leading_zeros == other.number_of_leading_zeros
            && self.raw_input == other.raw_input
            && self.country_code_source == other.country_code_source
            && self.preferred_domestic_carrier_code == other.preferred_domestic_carrier_code
            && self.has_preferred_domestic_carrier_code == other.has_preferred_domestic_carrier_code
    }
}

impl PartialEq for PhoneNumber {
    fn eq(&self, other: &Self) -> bool {
        self.exactly_same_as(other)
    }
}

impl Eq for PhoneNumber {}

impl Hash for PhoneNumber {
    /// Hashes the fields `exactlySameAs` compares, as `hashCode()` does.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.country_code.hash(state);
        self.national_number.hash(state);
        self.extension.hash(state);
        self.italian_leading_zero.hash(state);
        self.number_of_leading_zeros.hash(state);
        self.raw_input.hash(state);
        self.country_code_source.hash(state);
        self.preferred_domestic_carrier_code.hash(state);
        self.has_preferred_domestic_carrier_code.hash(state);
    }
}

impl fmt::Display for PhoneNumber {
    /// `toString()`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Country Code: {} National Number: {}", self.country_code, self.national_number)?;
        if self.has_italian_leading_zero && self.italian_leading_zero {
            f.write_str(" Leading Zero(s): true")?;
        }
        if self.has_number_of_leading_zeros {
            write!(f, " Number of leading zeros: {}", self.number_of_leading_zeros)?;
        }
        if self.has_extension {
            write!(f, " Extension: {}", self.extension)?;
        }
        if self.has_country_code_source {
            write!(f, " Country Code Source: {}", self.country_code_source)?;
        }
        if self.has_preferred_domestic_carrier_code {
            write!(f, " Preferred Domestic Carrier Code: {}", self.preferred_domestic_carrier_code)?;
        }
        Ok(())
    }
}

/// `NumberParseException.ErrorType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorType {
    /// The calling code is not a known one, or no default region was given for a national number.
    InvalidCountryCode,
    /// The input does not look like a phone number.
    NotANumber,
    /// The input ends too soon after the international dialling prefix.
    TooShortAfterIdd,
    /// The national significant number is too short to be a number.
    TooShortNsn,
    /// The input is too long to be a number.
    TooLong,
}

impl ErrorType {
    /// The Java constant's name.
    pub fn name(self) -> &'static str {
        match self {
            ErrorType::InvalidCountryCode => "INVALID_COUNTRY_CODE",
            ErrorType::NotANumber => "NOT_A_NUMBER",
            ErrorType::TooShortAfterIdd => "TOO_SHORT_AFTER_IDD",
            ErrorType::TooShortNsn => "TOO_SHORT_NSN",
            ErrorType::TooLong => "TOO_LONG",
        }
    }
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// `NumberParseException`: an error type and a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberParseException {
    error_type: ErrorType,
    message: String,
}

impl NumberParseException {
    /// A `NumberParseException` of `error_type` with `message`.
    pub fn new(error_type: ErrorType, message: impl Into<String>) -> Self {
        NumberParseException { error_type, message: message.into() }
    }

    /// `getErrorType()`.
    pub fn error_type(&self) -> ErrorType {
        self.error_type
    }

    /// `getMessage()`.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for NumberParseException {
    /// `toString()`: `Error type: <TYPE>. <message>`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error type: {}. {}", self.error_type, self.message)
    }
}

impl std::error::Error for NumberParseException {}
