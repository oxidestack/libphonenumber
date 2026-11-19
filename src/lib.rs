//! An unofficial Rust port of the parts of Google's [libphonenumber](https://github.com/google/libphonenumber)
//! 9.0.41 (Java, `com.googlecode.libphonenumber:libphonenumber`) that Datafaker 2.7.0 reaches: the region metadata,
//! parsing, validation, number types, possible-number checks and formatting. It is not affiliated with or endorsed by
//! Google.
//!
//! Results are libphonenumber's own: the metadata is libphonenumber 9.0.41's runtime metadata, embedded at build time
//! and decoded the way Java reads it, and the tests run libphonenumber's own test cases and thousands of rows
//! recorded from the Java library. Nothing is read from disk or the network at run time.
//!
//! [`PhoneNumberUtil`] is the entry point, as in Java: [`PhoneNumberUtil::get_instance`] over the real metadata.
//!
//! # Example
//!
//! ```
//! use libphonenumber::{PhoneNumberFormat, PhoneNumberType, PhoneNumberUtil};
//!
//! let util = PhoneNumberUtil::get_instance();
//! let number = util.parse("044 668 18 00", Some("CH")).expect("a Swiss number");
//!
//! assert_eq!(number.country_code(), 41);
//! assert!(util.is_valid_number(&number));
//! assert_eq!(util.get_number_type(&number), PhoneNumberType::FixedLine);
//! assert_eq!(util.format(&number, PhoneNumberFormat::E164), "+41446681800");
//! assert_eq!(util.format(&number, PhoneNumberFormat::International), "+41 44 668 18 00");
//!
//! // Input that is not a number fails as Java's `NumberParseException` does.
//! let error = util.parse("not a number", Some("CH")).unwrap_err();
//! assert_eq!(error.error_type(), libphonenumber::ErrorType::NotANumber);
//! ```
//!
//! Java's `null` is `None` where a method takes or returns it (the default region of `parse`), names are snake case
//! (`isValidNumber` is [`PhoneNumberUtil::is_valid_number`]), and an unchecked Java exception is a panic with the same
//! message. Not ported: `AsYouTypeFormatter`, `PhoneNumberMatcher`, `ShortNumberInfo`, the geocoder, carrier and
//! time-zone libraries; the README lists the rest.

#![warn(missing_docs)]

mod jregex;
pub mod metadata;
mod phonenumber;

pub use metadata::{MetadataSet, NumberFormat, PhoneMetadata, PhoneNumberDesc};
pub use phonenumber::{CountryCodeSource, ErrorType, NumberParseException, PhoneNumber};
mod util;

pub use util::{
    PhoneNumberFormat, PhoneNumberType, PhoneNumberUtil, ValidationResult, convert_alpha_characters_in_number,
    extract_possible_number, formatting_rule_has_first_group_only, get_country_mobile_token, get_national_significant_number,
    get_number_desc_by_type, is_number_geographical_type, is_viable_phone_number, maybe_strip_extension, normalize,
    normalize_diallable_chars_only, normalize_digits_only, set_italian_leading_zeros_for_phone_number,
    extn_patterns_for_matching, MAX_LENGTH_FOR_NSN, PLUS_CHARS, REGION_CODE_FOR_NON_GEO_ENTITY, VALID_PUNCTUATION,
};
