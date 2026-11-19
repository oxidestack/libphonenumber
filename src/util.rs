//! `PhoneNumberUtil`: parsing, validation, number types, possible-number checks and formatting.
//!
//! Method by method a port of libphonenumber 9.0.41's `PhoneNumberUtil.java`; the comments name the Java method.
//! String positions are byte offsets here where Java uses UTF-16 indices; lengths that Java compares on input that
//! may hold non-ASCII text are counted in UTF-16 units, as Java counts them.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use unicode_general_category::{GeneralCategory, get_general_category};

use crate::jregex::{JPattern, RegexCache};
use crate::metadata::{MetadataSet, NumberFormat, PhoneMetadata, PhoneNumberDesc};
use crate::phonenumber::{CountryCodeSource, ErrorType, NumberParseException, PhoneNumber};

const MIN_LENGTH_FOR_NSN: usize = 2;
/// `MAX_LENGTH_FOR_NSN`.
pub const MAX_LENGTH_FOR_NSN: usize = 17;
const MAX_LENGTH_COUNTRY_CODE: usize = 3;
const MAX_INPUT_STRING_LENGTH: usize = 250;
const UNKNOWN_REGION: &str = "ZZ";
const NANPA_COUNTRY_CODE: i32 = 1;
const PLUS_SIGN: char = '+';
const RFC3966_EXTN_PREFIX: &str = ";ext=";
const RFC3966_PREFIX: &str = "tel:";
const RFC3966_PHONE_CONTEXT: &str = ";phone-context=";
const RFC3966_ISDN_SUBADDRESS: &str = ";isub=";
const DEFAULT_EXTN_PREFIX: &str = " ext. ";
const CC_STRING: &str = "$CC";
const NP_STRING: &str = "$NP";
const FG_STRING: &str = "$FG";

/// `REGION_CODE_FOR_NON_GEO_ENTITY`.
pub const REGION_CODE_FOR_NON_GEO_ENTITY: &str = "001";

/// `VALID_PUNCTUATION`.
pub const VALID_PUNCTUATION: &str = "-x\u{2010}-\u{2015}\u{2212}\u{30FC}\u{FF0D}-\u{FF0F} \u{00A0}\u{00AD}\u{200B}\u{2060}\u{3000}()\u{FF08}\u{FF09}\u{FF3B}\u{FF3D}.\\[\\]/~\u{2053}\u{223C}\u{FF5E}";
const DIGITS: &str = "\\p{Nd}";
/// `VALID_ALPHA`: `ALPHA_MAPPINGS`' keys in `HashMap` order, then lower-cased.
const VALID_ALPHA: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
/// `PLUS_CHARS`.
pub const PLUS_CHARS: &str = "+\u{FF0B}";
const STAR_SIGN: char = '*';

/// `PhoneNumberFormat`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhoneNumberFormat {
    /// `+41446681800`.
    E164,
    /// `+41 44 668 1800`.
    International,
    /// `044 668 1800`.
    National,
    /// `tel:+41-44-668-1800`.
    Rfc3966,
}

/// `PhoneNumberType`, in declaration order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PhoneNumberType {
    /// A fixed-line number.
    FixedLine,
    /// A mobile number.
    Mobile,
    /// A number the region's patterns cannot tell between fixed-line and mobile.
    FixedLineOrMobile,
    /// A toll-free number.
    TollFree,
    /// A premium-rate number.
    PremiumRate,
    /// A shared-cost number, the cost split between caller and recipient.
    SharedCost,
    /// A voice-over-IP number.
    Voip,
    /// A personal number, routed to wherever its owner chooses.
    PersonalNumber,
    /// A pager number.
    Pager,
    /// A universal access number, a company's single number routed to its offices.
    Uan,
    /// A voicemail access number.
    Voicemail,
    /// A number that matches none of the region's types.
    Unknown,
}

impl PhoneNumberType {
    /// `values()`.
    pub const ALL: [PhoneNumberType; 12] = [
        PhoneNumberType::FixedLine,
        PhoneNumberType::Mobile,
        PhoneNumberType::FixedLineOrMobile,
        PhoneNumberType::TollFree,
        PhoneNumberType::PremiumRate,
        PhoneNumberType::SharedCost,
        PhoneNumberType::Voip,
        PhoneNumberType::PersonalNumber,
        PhoneNumberType::Pager,
        PhoneNumberType::Uan,
        PhoneNumberType::Voicemail,
        PhoneNumberType::Unknown,
    ];

    /// The Java constant's name.
    pub fn name(self) -> &'static str {
        [
            "FIXED_LINE",
            "MOBILE",
            "FIXED_LINE_OR_MOBILE",
            "TOLL_FREE",
            "PREMIUM_RATE",
            "SHARED_COST",
            "VOIP",
            "PERSONAL_NUMBER",
            "PAGER",
            "UAN",
            "VOICEMAIL",
            "UNKNOWN",
        ][self as usize]
    }
}

/// `ValidationResult`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValidationResult {
    /// The number's length is possible.
    IsPossible,
    /// The number's length is possible only when dialled within a local area.
    IsPossibleLocalOnly,
    /// The calling code is not a known one.
    InvalidCountryCode,
    /// The number is shorter than every possible length.
    TooShort,
    /// The number's length is between possible ones but is not one of them.
    InvalidLength,
    /// The number is longer than every possible length.
    TooLong,
}

impl ValidationResult {
    /// The Java constant's name.
    pub fn name(self) -> &'static str {
        match self {
            ValidationResult::IsPossible => "IS_POSSIBLE",
            ValidationResult::IsPossibleLocalOnly => "IS_POSSIBLE_LOCAL_ONLY",
            ValidationResult::InvalidCountryCode => "INVALID_COUNTRY_CODE",
            ValidationResult::TooShort => "TOO_SHORT",
            ValidationResult::InvalidLength => "INVALID_LENGTH",
            ValidationResult::TooLong => "TOO_LONG",
        }
    }
}

/// The static patterns of `PhoneNumberUtil`.
struct Patterns {
    single_international_prefix: JPattern,
    plus_chars: JPattern,
    separator: JPattern,
    capturing_digit: JPattern,
    valid_start_char: JPattern,
    second_number_start: JPattern,
    unwanted_end_char: JPattern,
    valid_alpha_phone: JPattern,
    extn: JPattern,
    valid_phone_number: JPattern,
    non_digits: JPattern,
    first_group: JPattern,
    first_group_only_prefix: JPattern,
    rfc3966_global_number_digits: JPattern,
    rfc3966_domainname: JPattern,
}

fn extn_digits(max_length: usize) -> String {
    format!("({DIGITS}{{1,{max_length}}})")
}

/// `createExtnPattern(forParsing)`.
fn create_extn_pattern(for_parsing: bool) -> String {
    let ext_limit_after_explicit_label = 20;
    let ext_limit_after_likely_label = 15;
    let ext_limit_after_ambiguous_char = 9;
    let ext_limit_when_not_sure = 6;
    let possible_separators_between_number_and_ext_label = "[ \u{00A0}\\t,]*";
    let possible_chars_after_ext_label = "[:\\.\u{FF0E}]?[ \u{00A0}\\t,-]*";
    let optional_extn_suffix = "#?";
    let explicit_ext_labels =
        "(?:e?xt(?:ensi(?:o\u{0301}?|\u{00F3}))?n?|\u{FF45}?\u{FF58}\u{FF54}\u{FF4E}?|\u{0434}\u{043E}\u{0431}|anexo)";
    let ambiguous_ext_labels = "(?:[x\u{FF58}#\u{FF03}~\u{FF5E}]|int|\u{FF49}\u{FF4E}\u{FF54})";
    let ambiguous_separator = "[- ]+";
    let rfc_extn = format!("{RFC3966_EXTN_PREFIX}{}", extn_digits(ext_limit_after_explicit_label));
    let explicit_extn = format!(
        "{possible_separators_between_number_and_ext_label}{explicit_ext_labels}{possible_chars_after_ext_label}{}{optional_extn_suffix}",
        extn_digits(ext_limit_after_explicit_label)
    );
    let ambiguous_extn = format!(
        "{possible_separators_between_number_and_ext_label}{ambiguous_ext_labels}{possible_chars_after_ext_label}{}{optional_extn_suffix}",
        extn_digits(ext_limit_after_ambiguous_char)
    );
    let american_style_extn_with_suffix = format!("{ambiguous_separator}{}#", extn_digits(ext_limit_when_not_sure));
    let extension_pattern = format!("{rfc_extn}|{explicit_extn}|{ambiguous_extn}|{american_style_extn_with_suffix}");
    if for_parsing {
        let possible_separators_number_ext_label_no_comma = "[ \u{00A0}\\t]*";
        let auto_dialling_and_ext_labels_found = "(?:,{2}|;)";
        let auto_dialling_extn = format!(
            "{possible_separators_number_ext_label_no_comma}{auto_dialling_and_ext_labels_found}{possible_chars_after_ext_label}{}{optional_extn_suffix}",
            extn_digits(ext_limit_after_likely_label)
        );
        let only_commas_extn = format!(
            "{possible_separators_number_ext_label_no_comma}(?:,)+{possible_chars_after_ext_label}{}{optional_extn_suffix}",
            extn_digits(ext_limit_after_ambiguous_char)
        );
        return format!("{extension_pattern}|{auto_dialling_extn}|{only_commas_extn}");
    }
    extension_pattern
}

/// `EXTN_PATTERNS_FOR_MATCHING`.
pub fn extn_patterns_for_matching() -> String {
    create_extn_pattern(false)
}

fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| {
        let extn_for_parsing = create_extn_pattern(true);
        // The possessive `[+＋]*+` of upstream is greedy here: what follows cannot match a plus sign, so the two agree.
        let valid_phone_number = format!(
            "{DIGITS}{{{MIN_LENGTH_FOR_NSN}}}|[{PLUS_CHARS}]*(?:[{VALID_PUNCTUATION}{STAR_SIGN}]*{DIGITS}){{3,}}[{VALID_PUNCTUATION}{STAR_SIGN}{VALID_ALPHA}{DIGITS}]*"
        );
        let rfc3966_visual_separator = "[\\-\\.\\(\\)]?";
        let rfc3966_phone_digit = format!("({DIGITS}|{rfc3966_visual_separator})");
        let alphanum = format!("{VALID_ALPHA}{DIGITS}");
        let rfc3966_domainlabel = format!("[{alphanum}]+((\\-)*[{alphanum}])*");
        let rfc3966_toplabel = format!("[{VALID_ALPHA}]+((\\-)*[{alphanum}])*");
        // Java's `.` stops at every line terminator, not only at `\n`.
        let dot = "[^\\n\\r\\x{85}\\x{2028}\\x{2029}]";
        Patterns {
            single_international_prefix: JPattern::new("[\\d]+(?:[~\u{2053}\u{223C}\u{FF5E}][\\d]+)?", ""),
            plus_chars: JPattern::new(&format!("[{PLUS_CHARS}]+"), ""),
            separator: JPattern::new(&format!("[{VALID_PUNCTUATION}]+"), ""),
            capturing_digit: JPattern::new(&format!("({DIGITS})"), ""),
            valid_start_char: JPattern::new(&format!("[{PLUS_CHARS}{DIGITS}]"), ""),
            second_number_start: JPattern::new("[\\\\/] *x", ""),
            // `[[\P{N}&&\P{L}]&&[^#]]+$`
            unwanted_end_char: JPattern::new("[^\\p{N}\\p{L}#]+$", ""),
            valid_alpha_phone: JPattern::new(&format!("(?:{dot}*?[A-Za-z]){{3}}{dot}*"), ""),
            extn: JPattern::new(&format!("(?:{extn_for_parsing})$"), "(?i)"),
            valid_phone_number: JPattern::new(&format!("{valid_phone_number}(?:{extn_for_parsing})?"), "(?i)"),
            non_digits: JPattern::new("(\\D+)", ""),
            first_group: JPattern::new("(\\$\\d)", ""),
            first_group_only_prefix: JPattern::new("\\(?\\$1\\)?", ""),
            rfc3966_global_number_digits: JPattern::new(
                &format!("^\\{PLUS_SIGN}{rfc3966_phone_digit}*{DIGITS}{rfc3966_phone_digit}*$"),
                "",
            ),
            rfc3966_domainname: JPattern::new(&format!("^({rfc3966_domainlabel}\\.)*{rfc3966_toplabel}\\.?$"), ""),
        }
    })
}

/// Java's `String.length()`: UTF-16 code units.
fn utf16_len(s: &str) -> usize {
    s.chars().map(char::len_utf16).sum()
}

/// `Character.digit(char, 10)`: the value of a decimal digit (`Nd`) in the Basic Multilingual Plane. Decimal digits
/// come in runs of ten consecutive code points from zero to nine.
fn java_digit(c: char) -> Option<u32> {
    if (c as u32) > 0xFFFF || get_general_category(c) != GeneralCategory::DecimalNumber {
        return None;
    }
    let mut start = c as u32;
    while let Some(prev) = start.checked_sub(1).and_then(char::from_u32) {
        if get_general_category(prev) != GeneralCategory::DecimalNumber {
            break;
        }
        start -= 1;
    }
    Some((c as u32 - start) % 10)
}

/// `ALPHA_MAPPINGS`.
fn alpha_mapping(c: char) -> Option<char> {
    Some(match c {
        'A' | 'B' | 'C' => '2',
        'D' | 'E' | 'F' => '3',
        'G' | 'H' | 'I' => '4',
        'J' | 'K' | 'L' => '5',
        'M' | 'N' | 'O' => '6',
        'P' | 'Q' | 'R' | 'S' => '7',
        'T' | 'U' | 'V' => '8',
        'W' | 'X' | 'Y' | 'Z' => '9',
        _ => return None,
    })
}

/// `ALPHA_PHONE_MAPPINGS`.
fn alpha_phone_mapping(c: char) -> Option<char> {
    if c.is_ascii_digit() { Some(c) } else { alpha_mapping(c) }
}

/// `DIALLABLE_CHAR_MAPPINGS`.
fn diallable_char_mapping(c: char) -> Option<char> {
    (c.is_ascii_digit() || c == '+' || c == '*' || c == '#').then_some(c)
}

/// `Character.toUpperCase(char)`.
fn java_to_upper(c: char) -> char {
    let mut upper = c.to_uppercase();
    match (upper.next(), upper.next()) {
        (Some(u), None) => u,
        _ => c,
    }
}

/// `normalizeHelper(number, normalizationReplacements, removeNonMatches)`.
fn normalize_helper(number: &str, mapping: fn(char) -> Option<char>, remove_non_matches: bool) -> String {
    let mut out = String::with_capacity(number.len());
    for c in number.chars() {
        if let Some(d) = mapping(java_to_upper(c)) {
            out.push(d);
        } else if !remove_non_matches {
            out.push(c);
        }
    }
    out
}

/// `normalizeDigits(number, keepNonDigits)`.
fn normalize_digits(number: &str, keep_non_digits: bool) -> String {
    let mut out = String::with_capacity(number.len());
    for c in number.chars() {
        if let Some(d) = java_digit(c) {
            out.push(char::from_digit(d, 10).expect("digit"));
        } else if keep_non_digits {
            out.push(c);
        }
    }
    out
}

/// `normalizeDigitsOnly(number)`: decimal digits of any script as ASCII digits, everything else dropped.
pub fn normalize_digits_only(number: &str) -> String {
    normalize_digits(number, false)
}

/// `normalizeDiallableCharsOnly(number)`.
pub fn normalize_diallable_chars_only(number: &str) -> String {
    normalize_helper(number, diallable_char_mapping, true)
}

/// `convertAlphaCharactersInNumber(number)`.
pub fn convert_alpha_characters_in_number(number: &str) -> String {
    normalize_helper(number, alpha_phone_mapping, false)
}

/// `normalize(number)` (package-private upstream): letters to keypad digits if there are three or more of them,
/// otherwise digits only.
pub fn normalize(number: &str) -> String {
    if patterns().valid_alpha_phone.matches(number) {
        normalize_helper(number, alpha_phone_mapping, true)
    } else {
        normalize_digits_only(number)
    }
}

/// `extractPossibleNumber(number)` (package-private upstream).
pub fn extract_possible_number(number: &str) -> &str {
    let p = patterns();
    let Some(start) = p.valid_start_char.find(number).map(|c| c.get(0).expect("match").start()) else {
        return "";
    };
    let mut number = &number[start..];
    if let Some(m) = p.unwanted_end_char.find(number) {
        number = &number[..m.get(0).expect("match").start()];
    }
    if let Some(m) = p.second_number_start.find(number) {
        number = &number[..m.get(0).expect("match").start()];
    }
    number
}

/// `isViablePhoneNumber(number)` (package-private upstream).
pub fn is_viable_phone_number(number: &str) -> bool {
    if utf16_len(number) < MIN_LENGTH_FOR_NSN {
        return false;
    }
    patterns().valid_phone_number.matches(number)
}

/// `getCountryMobileToken(countryCallingCode)`.
pub fn get_country_mobile_token(country_calling_code: i32) -> &'static str {
    if country_calling_code == 54 { "9" } else { "" }
}

/// `formattingRuleHasFirstGroupOnly(nationalPrefixFormattingRule)` (package-private upstream).
pub fn formatting_rule_has_first_group_only(national_prefix_formatting_rule: &str) -> bool {
    national_prefix_formatting_rule.is_empty() || patterns().first_group_only_prefix.matches(national_prefix_formatting_rule)
}

/// `setItalianLeadingZerosForPhoneNumber(nationalNumber, phoneNumber)` (package-private upstream).
pub fn set_italian_leading_zeros_for_phone_number(national_number: &str, phone_number: &mut PhoneNumber) {
    let bytes = national_number.as_bytes();
    if bytes.len() > 1 && bytes[0] == b'0' {
        phone_number.set_italian_leading_zero(true);
        let mut number_of_leading_zeros = 1;
        while number_of_leading_zeros < bytes.len() - 1 && bytes[number_of_leading_zeros] == b'0' {
            number_of_leading_zeros += 1;
        }
        if number_of_leading_zeros != 1 {
            phone_number.set_number_of_leading_zeros(number_of_leading_zeros as i32);
        }
    }
}

/// Java `HashMap`/`HashSet` iteration order of keys inserted in `keys` order into a map created with
/// `initial_capacity`: by bucket (`hash ^ hash >>> 16`, masked to the final table size), then insertion order.
fn java_hash_order<T: Clone + Eq + std::hash::Hash>(keys: &[T], hash: impl Fn(&T) -> i32, initial_capacity: usize) -> Vec<T> {
    let mut seen = HashSet::new();
    let unique: Vec<T> = keys.iter().filter(|k| seen.insert((*k).clone())).cloned().collect();
    let mut table = initial_capacity.max(1).next_power_of_two();
    while unique.len() as f64 > table as f64 * 0.75 {
        table *= 2;
    }
    let mut out = unique;
    out.sort_by_key(|k| {
        let h = hash(k) as u32;
        ((h ^ (h >> 16)) as usize) & (table - 1)
    });
    out
}

/// `String.hashCode()`.
fn java_string_hash(s: &str) -> i32 {
    s.encode_utf16().fold(0i32, |h, u| h.wrapping_mul(31).wrapping_add(u as i32))
}

/// `PhoneNumberUtil`.
pub struct PhoneNumberUtil {
    metadata: &'static MetadataSet,
    country_calling_code_to_region_code_map: HashMap<i32, Vec<String>>,
    /// The map's keys in Java's `HashMap` order.
    calling_codes: Vec<i32>,
    /// `supportedRegions` in Java's `HashSet` order.
    supported_regions: Vec<String>,
    supported_region_set: HashSet<String>,
    /// `countryCodesForNonGeographicalRegion` in Java's `HashSet` order.
    country_codes_for_non_geographical_region: Vec<i32>,
    nanpa_regions: HashSet<String>,
    regex_cache: RegexCache,
}

impl std::fmt::Debug for PhoneNumberUtil {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhoneNumberUtil").field("supported_regions", &self.supported_regions.len()).finish()
    }
}

impl PhoneNumberUtil {
    /// `getInstance()`: the util over libphonenumber's metadata.
    pub fn get_instance() -> &'static PhoneNumberUtil {
        static INSTANCE: OnceLock<PhoneNumberUtil> = OnceLock::new();
        INSTANCE.get_or_init(|| PhoneNumberUtil::new(MetadataSet::default_metadata()))
    }

    /// The util over the test metadata upstream's unit tests use (`TestMetadataTestCase`).
    pub fn for_testing() -> &'static PhoneNumberUtil {
        static INSTANCE: OnceLock<PhoneNumberUtil> = OnceLock::new();
        INSTANCE.get_or_init(|| PhoneNumberUtil::new(MetadataSet::testing_metadata()))
    }

    /// `new PhoneNumberUtil(metadataSource, countryCallingCodeToRegionCodeMap)`.
    pub fn new(metadata: &'static MetadataSet) -> PhoneNumberUtil {
        let entries = &metadata.country_code_to_region_codes;
        let map: HashMap<i32, Vec<String>> = entries.iter().cloned().collect();
        let codes: Vec<i32> = entries.iter().map(|(c, _)| *c).collect();
        let calling_codes = java_hash_order(&codes, |c| *c, 286);
        let mut non_geo = Vec::new();
        let mut regions = Vec::new();
        for code in &calling_codes {
            let region_codes = &map[code];
            if region_codes.len() == 1 && region_codes[0] == REGION_CODE_FOR_NON_GEO_ENTITY {
                non_geo.push(*code);
            } else {
                regions.extend(region_codes.iter().cloned());
            }
        }
        regions.retain(|r| r != REGION_CODE_FOR_NON_GEO_ENTITY);
        let supported_regions = java_hash_order(&regions, |r| java_string_hash(r), 320);
        let country_codes_for_non_geographical_region = java_hash_order(&non_geo, |c| *c, 16);
        let nanpa_regions = map.get(&NANPA_COUNTRY_CODE).map(|r| r.iter().cloned().collect()).unwrap_or_default();
        PhoneNumberUtil {
            metadata,
            supported_region_set: supported_regions.iter().cloned().collect(),
            country_calling_code_to_region_code_map: map,
            calling_codes,
            supported_regions,
            country_codes_for_non_geographical_region,
            nanpa_regions,
            regex_cache: RegexCache::default(),
        }
    }

    /// `getSupportedRegions()`, in Java's iteration order.
    pub fn get_supported_regions(&self) -> &[String] {
        &self.supported_regions
    }

    /// `getSupportedGlobalNetworkCallingCodes()`, in Java's iteration order.
    pub fn get_supported_global_network_calling_codes(&self) -> &[i32] {
        &self.country_codes_for_non_geographical_region
    }

    /// `getSupportedCallingCodes()`, in Java's iteration order.
    pub fn get_supported_calling_codes(&self) -> &[i32] {
        &self.calling_codes
    }

    fn desc_has_possible_number_data(desc: &PhoneNumberDesc) -> bool {
        desc.possible_length.len() != 1 || desc.possible_length[0] != -1
    }

    fn desc_has_data(desc: &PhoneNumberDesc) -> bool {
        desc.has_example_number || Self::desc_has_possible_number_data(desc) || desc.has_national_number_pattern
    }

    fn get_supported_types_for_metadata(&self, metadata: &PhoneMetadata) -> Vec<PhoneNumberType> {
        PhoneNumberType::ALL
            .into_iter()
            .filter(|t| *t != PhoneNumberType::FixedLineOrMobile && *t != PhoneNumberType::Unknown)
            .filter(|t| Self::desc_has_data(get_number_desc_by_type(metadata, *t)))
            .collect()
    }

    /// `getSupportedTypesForRegion(regionCode)`, sorted as the `TreeSet` upstream returns.
    pub fn get_supported_types_for_region(&self, region_code: &str) -> Vec<PhoneNumberType> {
        match self.get_metadata_for_region(region_code) {
            Some(metadata) => self.get_supported_types_for_metadata(metadata),
            None => Vec::new(),
        }
    }

    /// `getSupportedTypesForNonGeoEntity(countryCallingCode)`.
    pub fn get_supported_types_for_non_geo_entity(&self, country_calling_code: i32) -> Vec<PhoneNumberType> {
        match self.get_metadata_for_non_geographical_region(country_calling_code) {
            Some(metadata) => self.get_supported_types_for_metadata(metadata),
            None => Vec::new(),
        }
    }

    /// `isNumberGeographical(phoneNumber)`.
    pub fn is_number_geographical(&self, phone_number: &PhoneNumber) -> bool {
        is_number_geographical_type(self.get_number_type(phone_number), phone_number.country_code())
    }

    fn is_valid_region_code(&self, region_code: Option<&str>) -> bool {
        region_code.is_some_and(|r| self.supported_region_set.contains(r))
    }

    fn has_valid_country_calling_code(&self, country_calling_code: i32) -> bool {
        self.country_calling_code_to_region_code_map.contains_key(&country_calling_code)
    }

    /// `getMetadataForRegion(regionCode)`: `None` for an invalid or unknown region code.
    ///
    /// # Panics
    /// When the region is supported but no metadata defines it (Java's `MissingMetadataException`).
    pub fn get_metadata_for_region(&self, region_code: &str) -> Option<&'static PhoneMetadata> {
        if !self.is_valid_region_code(Some(region_code)) {
            return None;
        }
        let metadata = self.metadata.region(region_code);
        assert!(metadata.is_some(), "Missing metadata for region code {region_code}");
        metadata
    }

    fn get_metadata_for_region_opt(&self, region_code: Option<&str>) -> Option<&'static PhoneMetadata> {
        region_code.and_then(|r| self.get_metadata_for_region(r))
    }

    /// `getMetadataForNonGeographicalRegion(countryCallingCode)`.
    ///
    /// # Panics
    /// When the calling code is a non-geographical one but no metadata defines it.
    pub fn get_metadata_for_non_geographical_region(&self, country_calling_code: i32) -> Option<&'static PhoneMetadata> {
        if !self.country_codes_for_non_geographical_region.contains(&country_calling_code) {
            return None;
        }
        let metadata = self.metadata.non_geographical(country_calling_code);
        assert!(metadata.is_some(), "Missing metadata for country code {country_calling_code}");
        metadata
    }

    fn get_metadata_for_region_or_calling_code(
        &self,
        country_calling_code: i32,
        region_code: Option<&str>,
    ) -> Option<&'static PhoneMetadata> {
        if region_code == Some(REGION_CODE_FOR_NON_GEO_ENTITY) {
            self.get_metadata_for_non_geographical_region(country_calling_code)
        } else {
            self.get_metadata_for_region_opt(region_code)
        }
    }

    /// `getLengthOfGeographicalAreaCode(number)`.
    pub fn get_length_of_geographical_area_code(&self, number: &PhoneNumber) -> usize {
        let region = self.get_region_code_for_number(number);
        let Some(metadata) = self.get_metadata_for_region_opt(region.as_deref()) else {
            return 0;
        };
        let number_type = self.get_number_type(number);
        let country_calling_code = number.country_code();
        if !metadata.has_national_prefix && !number.is_italian_leading_zero() && country_calling_code != 52 {
            return 0;
        }
        if number_type == PhoneNumberType::Mobile && country_calling_code == 86 {
            return 0;
        }
        if !is_number_geographical_type(number_type, country_calling_code) {
            return 0;
        }
        self.get_length_of_national_destination_code(number)
    }

    /// `getLengthOfNationalDestinationCode(number)`.
    pub fn get_length_of_national_destination_code(&self, number: &PhoneNumber) -> usize {
        let mut copied = number.clone();
        if number.has_extension() {
            copied.clear_extension();
        }
        let formatted = self.format(&copied, PhoneNumberFormat::International);
        let groups = java_split(&patterns().non_digits, &formatted);
        if groups.len() <= 3 {
            return 0;
        }
        if self.get_number_type(number) == PhoneNumberType::Mobile && !get_country_mobile_token(number.country_code()).is_empty() {
            return groups[2].len() + groups[3].len();
        }
        groups[2].len()
    }

    /// `format(number, numberFormat)`.
    pub fn format(&self, number: &PhoneNumber, number_format: PhoneNumberFormat) -> String {
        if number.national_number() == 0 {
            let raw_input = number.raw_input();
            if !raw_input.is_empty()
                && number.has_country_code()
                && number.country_code_source() == CountryCodeSource::FromDefaultCountry
                && number_format == PhoneNumberFormat::E164
            {
                let mut formatted = raw_input.to_string();
                prefix_number_with_country_calling_code(number.country_code(), number_format, &mut formatted);
                return formatted;
            } else if !raw_input.is_empty() || !number.has_country_code() {
                return raw_input.to_string();
            }
        }
        let country_calling_code = number.country_code();
        let national_significant_number = get_national_significant_number(number);
        let mut formatted = String::with_capacity(20);
        if number_format == PhoneNumberFormat::E164 {
            formatted.push_str(&national_significant_number);
            prefix_number_with_country_calling_code(country_calling_code, PhoneNumberFormat::E164, &mut formatted);
            return formatted;
        }
        if !self.has_valid_country_calling_code(country_calling_code) {
            return national_significant_number;
        }
        let region_code = self.get_region_code_for_country_code(country_calling_code);
        let metadata = self
            .get_metadata_for_region_or_calling_code(country_calling_code, Some(region_code))
            .expect("metadata of a valid calling code");
        formatted.push_str(&self.format_nsn(&national_significant_number, metadata, number_format, None));
        maybe_append_formatted_extension(number, metadata, number_format, &mut formatted);
        prefix_number_with_country_calling_code(country_calling_code, number_format, &mut formatted);
        formatted
    }

    /// `formatByPattern(number, numberFormat, userDefinedFormats)`.
    pub fn format_by_pattern(
        &self,
        number: &PhoneNumber,
        number_format: PhoneNumberFormat,
        user_defined_formats: &[NumberFormat],
    ) -> String {
        let country_calling_code = number.country_code();
        let national_significant_number = get_national_significant_number(number);
        if !self.has_valid_country_calling_code(country_calling_code) {
            return national_significant_number;
        }
        let region_code = self.get_region_code_for_country_code(country_calling_code);
        let metadata = self
            .get_metadata_for_region_or_calling_code(country_calling_code, Some(region_code))
            .expect("metadata of a valid calling code");
        let mut formatted = String::with_capacity(20);
        match self.choose_formatting_pattern_for_number(user_defined_formats, &national_significant_number) {
            None => formatted.push_str(&national_significant_number),
            Some(pattern) => {
                let mut copy = pattern.clone();
                let rule = &pattern.national_prefix_formatting_rule;
                if !rule.is_empty() {
                    if !metadata.national_prefix.is_empty() {
                        copy.national_prefix_formatting_rule =
                            rule.replace(NP_STRING, &metadata.national_prefix).replace(FG_STRING, "$1");
                        copy.has_national_prefix_formatting_rule = true;
                    } else {
                        copy.national_prefix_formatting_rule.clear();
                        copy.has_national_prefix_formatting_rule = false;
                    }
                }
                formatted.push_str(&self.format_nsn_using_pattern(&national_significant_number, &copy, number_format, None));
            }
        }
        maybe_append_formatted_extension(number, metadata, number_format, &mut formatted);
        prefix_number_with_country_calling_code(country_calling_code, number_format, &mut formatted);
        formatted
    }

    /// `formatNationalNumberWithCarrierCode(number, carrierCode)`.
    pub fn format_national_number_with_carrier_code(&self, number: &PhoneNumber, carrier_code: &str) -> String {
        let country_calling_code = number.country_code();
        let national_significant_number = get_national_significant_number(number);
        if !self.has_valid_country_calling_code(country_calling_code) {
            return national_significant_number;
        }
        let region_code = self.get_region_code_for_country_code(country_calling_code);
        let metadata = self
            .get_metadata_for_region_or_calling_code(country_calling_code, Some(region_code))
            .expect("metadata of a valid calling code");
        let mut formatted = String::with_capacity(20);
        formatted.push_str(&self.format_nsn(
            &national_significant_number,
            metadata,
            PhoneNumberFormat::National,
            Some(carrier_code),
        ));
        maybe_append_formatted_extension(number, metadata, PhoneNumberFormat::National, &mut formatted);
        prefix_number_with_country_calling_code(country_calling_code, PhoneNumberFormat::National, &mut formatted);
        formatted
    }

    /// `formatNationalNumberWithPreferredCarrierCode(number, fallbackCarrierCode)`.
    pub fn format_national_number_with_preferred_carrier_code(&self, number: &PhoneNumber, fallback_carrier_code: &str) -> String {
        let preferred = number.preferred_domestic_carrier_code();
        self.format_national_number_with_carrier_code(
            number,
            if preferred.is_empty() { fallback_carrier_code } else { preferred },
        )
    }

    /// `chooseFormattingPatternForNumber(availableFormats, nationalNumber)` (package-private upstream).
    pub fn choose_formatting_pattern_for_number<'f>(
        &self,
        available_formats: &'f [NumberFormat],
        national_number: &str,
    ) -> Option<&'f NumberFormat> {
        available_formats.iter().find(|f| {
            let leading = f.leading_digits_pattern.last();
            (leading.is_none() || self.regex_cache.get(leading.expect("pattern")).looking_at(national_number).is_some())
                && self.regex_cache.get(&f.pattern).matches(national_number)
        })
    }

    fn format_nsn(
        &self,
        number: &str,
        metadata: &PhoneMetadata,
        number_format: PhoneNumberFormat,
        carrier_code: Option<&str>,
    ) -> String {
        let available_formats = if metadata.intl_number_format.is_empty() || number_format == PhoneNumberFormat::National {
            &metadata.number_format
        } else {
            &metadata.intl_number_format
        };
        match self.choose_formatting_pattern_for_number(available_formats, number) {
            None => number.to_string(),
            Some(pattern) => self.format_nsn_using_pattern(number, pattern, number_format, carrier_code),
        }
    }

    /// `formatNsnUsingPattern(nationalNumber, formattingPattern, numberFormat)` (package-private upstream), with the
    /// optional carrier code of the private overload.
    pub fn format_nsn_using_pattern(
        &self,
        national_number: &str,
        formatting_pattern: &NumberFormat,
        number_format: PhoneNumberFormat,
        carrier_code: Option<&str>,
    ) -> String {
        let mut number_format_rule = formatting_pattern.format.clone();
        let m = self.regex_cache.get(&formatting_pattern.pattern);
        let p = patterns();
        let mut formatted;
        if number_format == PhoneNumberFormat::National
            && carrier_code.is_some_and(|c| !c.is_empty())
            && !formatting_pattern.domestic_carrier_code_formatting_rule.is_empty()
        {
            let carrier_code_formatting_rule =
                formatting_pattern.domestic_carrier_code_formatting_rule.replace(CC_STRING, carrier_code.expect("carrier"));
            number_format_rule = p.first_group.replace_first(&number_format_rule, &carrier_code_formatting_rule);
            formatted = m.replace_all(national_number, &number_format_rule);
        } else {
            let rule = &formatting_pattern.national_prefix_formatting_rule;
            if number_format == PhoneNumberFormat::National && !rule.is_empty() {
                formatted = m.replace_all(national_number, &p.first_group.replace_first(&number_format_rule, rule));
            } else {
                formatted = m.replace_all(national_number, &number_format_rule);
            }
        }
        if number_format == PhoneNumberFormat::Rfc3966 {
            if p.separator.looking_at(&formatted).is_some() {
                formatted = p.separator.replace_first(&formatted, "");
            }
            formatted = p.separator.replace_all(&formatted, "-");
        }
        formatted
    }

    /// `getExampleNumber(regionCode)`.
    pub fn get_example_number(&self, region_code: &str) -> Option<PhoneNumber> {
        self.get_example_number_for_type(region_code, PhoneNumberType::FixedLine)
    }

    /// `getInvalidExampleNumber(regionCode)`.
    pub fn get_invalid_example_number(&self, region_code: &str) -> Option<PhoneNumber> {
        let metadata = self.get_metadata_for_region(region_code)?;
        let desc = get_number_desc_by_type(metadata, PhoneNumberType::FixedLine);
        if !desc.has_example_number {
            return None;
        }
        let example = &desc.example_number;
        for length in (MIN_LENGTH_FOR_NSN..example.len()).rev() {
            if let Ok(possibly_valid) = self.parse(&example[..length], Some(region_code)) {
                if !self.is_valid_number(&possibly_valid) {
                    return Some(possibly_valid);
                }
            }
        }
        None
    }

    /// `getExampleNumberForType(regionCode, type)`.
    pub fn get_example_number_for_type(&self, region_code: &str, number_type: PhoneNumberType) -> Option<PhoneNumber> {
        let metadata = self.get_metadata_for_region(region_code)?;
        let desc = get_number_desc_by_type(metadata, number_type);
        if desc.has_example_number {
            return self.parse(&desc.example_number, Some(region_code)).ok();
        }
        None
    }

    /// `getExampleNumberForType(type)`: the first region (in Java's iteration order) or non-geographical entity with
    /// an example of that type.
    pub fn get_example_number_for_any_region(&self, number_type: PhoneNumberType) -> Option<PhoneNumber> {
        for region_code in &self.supported_regions {
            if let Some(number) = self.get_example_number_for_type(region_code, number_type) {
                return Some(number);
            }
        }
        for code in &self.country_codes_for_non_geographical_region {
            let metadata = self.get_metadata_for_non_geographical_region(*code).expect("non-geo metadata");
            let desc = get_number_desc_by_type(metadata, number_type);
            if desc.has_example_number {
                if let Ok(number) = self.parse(&format!("+{code}{}", desc.example_number), Some(UNKNOWN_REGION)) {
                    return Some(number);
                }
            }
        }
        None
    }

    /// `getExampleNumberForNonGeoEntity(countryCallingCode)`.
    pub fn get_example_number_for_non_geo_entity(&self, country_calling_code: i32) -> Option<PhoneNumber> {
        let metadata = self.get_metadata_for_non_geographical_region(country_calling_code)?;
        for desc in [
            &metadata.mobile,
            &metadata.toll_free,
            &metadata.shared_cost,
            &metadata.voip,
            &metadata.voicemail,
            &metadata.uan,
            &metadata.premium_rate,
        ]
        .into_iter()
        .flatten()
        {
            if desc.has_example_number {
                if let Ok(number) = self.parse(&format!("+{country_calling_code}{}", desc.example_number), Some(UNKNOWN_REGION)) {
                    return Some(number);
                }
            }
        }
        None
    }

    /// `getNumberType(number)`.
    pub fn get_number_type(&self, number: &PhoneNumber) -> PhoneNumberType {
        let region_code = self.get_region_code_for_number(number);
        let Some(metadata) = self.get_metadata_for_region_or_calling_code(number.country_code(), region_code.as_deref()) else {
            return PhoneNumberType::Unknown;
        };
        self.get_number_type_helper(&get_national_significant_number(number), metadata)
    }

    fn get_number_type_helper(&self, national_number: &str, metadata: &PhoneMetadata) -> PhoneNumberType {
        if !self.is_number_matching_desc(national_number, metadata.general_desc()) {
            return PhoneNumberType::Unknown;
        }
        let checks = [
            (PhoneNumberType::PremiumRate, &metadata.premium_rate),
            (PhoneNumberType::TollFree, &metadata.toll_free),
            (PhoneNumberType::SharedCost, &metadata.shared_cost),
            (PhoneNumberType::Voip, &metadata.voip),
            (PhoneNumberType::PersonalNumber, &metadata.personal_number),
            (PhoneNumberType::Pager, &metadata.pager),
            (PhoneNumberType::Uan, &metadata.uan),
            (PhoneNumberType::Voicemail, &metadata.voicemail),
        ];
        for (number_type, desc) in checks {
            if self.is_number_matching_desc(national_number, desc_or_empty(desc)) {
                return number_type;
            }
        }
        let is_fixed_line = self.is_number_matching_desc(national_number, desc_or_empty(&metadata.fixed_line));
        if is_fixed_line {
            if metadata.same_mobile_and_fixed_line_pattern
                || self.is_number_matching_desc(national_number, desc_or_empty(&metadata.mobile))
            {
                return PhoneNumberType::FixedLineOrMobile;
            }
            return PhoneNumberType::FixedLine;
        }
        if !metadata.same_mobile_and_fixed_line_pattern
            && self.is_number_matching_desc(national_number, desc_or_empty(&metadata.mobile))
        {
            return PhoneNumberType::Mobile;
        }
        PhoneNumberType::Unknown
    }

    /// `isNumberMatchingDesc(nationalNumber, numberDesc)` (package-private upstream).
    pub fn is_number_matching_desc(&self, national_number: &str, number_desc: &PhoneNumberDesc) -> bool {
        let actual_length = national_number.len() as i32;
        if !number_desc.possible_length.is_empty() && !number_desc.possible_length.contains(&actual_length) {
            return false;
        }
        self.match_national_number(national_number, number_desc, false)
    }

    /// `RegexBasedMatcher.matchNationalNumber(number, numberDesc, allowPrefixMatch)`.
    fn match_national_number(&self, number: &str, number_desc: &PhoneNumberDesc, allow_prefix_match: bool) -> bool {
        let pattern = &number_desc.national_number_pattern;
        if pattern.is_empty() {
            return false;
        }
        let p = self.regex_cache.get(pattern);
        if p.looking_at(number).is_none() {
            return false;
        }
        p.matches(number) || allow_prefix_match
    }

    /// `isValidNumber(number)`.
    pub fn is_valid_number(&self, number: &PhoneNumber) -> bool {
        let region_code = self.get_region_code_for_number(number);
        self.is_valid_number_for_region_opt(number, region_code.as_deref())
    }

    /// `isValidNumberForRegion(number, regionCode)`.
    pub fn is_valid_number_for_region(&self, number: &PhoneNumber, region_code: &str) -> bool {
        self.is_valid_number_for_region_opt(number, Some(region_code))
    }

    fn is_valid_number_for_region_opt(&self, number: &PhoneNumber, region_code: Option<&str>) -> bool {
        let country_code = number.country_code();
        let Some(metadata) = self.get_metadata_for_region_or_calling_code(country_code, region_code) else {
            return false;
        };
        if region_code != Some(REGION_CODE_FOR_NON_GEO_ENTITY)
            && country_code != self.get_country_code_for_valid_region(region_code.expect("region"))
        {
            return false;
        }
        self.get_number_type_helper(&get_national_significant_number(number), metadata) != PhoneNumberType::Unknown
    }

    /// `getRegionCodeForNumber(number)`: `None` (Java's `null`) when no region matches.
    pub fn get_region_code_for_number(&self, number: &PhoneNumber) -> Option<String> {
        let regions = self.country_calling_code_to_region_code_map.get(&number.country_code())?;
        if regions.len() == 1 {
            return Some(regions[0].clone());
        }
        self.get_region_code_for_number_from_region_list(number, regions)
    }

    fn get_region_code_for_number_from_region_list(&self, number: &PhoneNumber, region_codes: &[String]) -> Option<String> {
        let national_number = get_national_significant_number(number);
        for region_code in region_codes {
            let metadata = self.get_metadata_for_region(region_code).expect("metadata of a mapped region");
            if metadata.has_leading_digits {
                if self.regex_cache.get(&metadata.leading_digits).looking_at(&national_number).is_some() {
                    return Some(region_code.clone());
                }
            } else if self.get_number_type_helper(&national_number, metadata) != PhoneNumberType::Unknown {
                return Some(region_code.clone());
            }
        }
        None
    }

    /// `getRegionCodeForCountryCode(countryCallingCode)`: `ZZ` when unknown.
    pub fn get_region_code_for_country_code(&self, country_calling_code: i32) -> &str {
        self.country_calling_code_to_region_code_map
            .get(&country_calling_code)
            .map_or(UNKNOWN_REGION, |r| r[0].as_str())
    }

    /// `getRegionCodesForCountryCode(countryCallingCode)`.
    pub fn get_region_codes_for_country_code(&self, country_calling_code: i32) -> Vec<String> {
        self.country_calling_code_to_region_code_map.get(&country_calling_code).cloned().unwrap_or_default()
    }

    /// `getCountryCodeForRegion(regionCode)`: 0 for an invalid or unknown region.
    pub fn get_country_code_for_region(&self, region_code: &str) -> i32 {
        if !self.is_valid_region_code(Some(region_code)) {
            return 0;
        }
        self.get_country_code_for_valid_region(region_code)
    }

    fn get_country_code_for_valid_region(&self, region_code: &str) -> i32 {
        self.get_metadata_for_region(region_code)
            .unwrap_or_else(|| panic!("Invalid region code: {region_code}"))
            .country_code
    }

    /// `getNddPrefixForRegion(regionCode, stripNonDigits)`.
    pub fn get_ndd_prefix_for_region(&self, region_code: &str, strip_non_digits: bool) -> Option<String> {
        let metadata = self.get_metadata_for_region(region_code)?;
        let national_prefix = &metadata.national_prefix;
        if national_prefix.is_empty() {
            return None;
        }
        Some(if strip_non_digits { national_prefix.replace('~', "") } else { national_prefix.clone() })
    }

    /// `isNANPACountry(regionCode)`.
    pub fn is_nanpa_country(&self, region_code: &str) -> bool {
        self.nanpa_regions.contains(region_code)
    }

    /// `isAlphaNumber(number)`.
    pub fn is_alpha_number(&self, number: &str) -> bool {
        if utf16_len(number) > MAX_INPUT_STRING_LENGTH || !is_viable_phone_number(number) {
            return false;
        }
        let mut stripped = number.to_string();
        maybe_strip_extension(&mut stripped);
        stripped.chars().filter(char::is_ascii_alphabetic).take(3).count() >= 3
    }

    /// `isPossibleNumber(number)`.
    pub fn is_possible_number(&self, number: &PhoneNumber) -> bool {
        matches!(
            self.is_possible_number_with_reason(number),
            ValidationResult::IsPossible | ValidationResult::IsPossibleLocalOnly
        )
    }

    /// `isPossibleNumberForType(number, type)`.
    pub fn is_possible_number_for_type(&self, number: &PhoneNumber, number_type: PhoneNumberType) -> bool {
        matches!(
            self.is_possible_number_for_type_with_reason(number, number_type),
            ValidationResult::IsPossible | ValidationResult::IsPossibleLocalOnly
        )
    }

    /// `testNumberLength(number, metadata, type)`.
    fn test_number_length(&self, number: &str, metadata: &PhoneMetadata, number_type: PhoneNumberType) -> ValidationResult {
        let desc_for_type = get_number_desc_by_type(metadata, number_type);
        let mut possible_lengths: Vec<i32> = if desc_for_type.possible_length.is_empty() {
            metadata.general_desc().possible_length.clone()
        } else {
            desc_for_type.possible_length.clone()
        };
        let mut local_lengths = desc_for_type.possible_length_local_only.clone();
        if number_type == PhoneNumberType::FixedLineOrMobile {
            if !Self::desc_has_possible_number_data(get_number_desc_by_type(metadata, PhoneNumberType::FixedLine)) {
                return self.test_number_length(number, metadata, PhoneNumberType::Mobile);
            }
            let mobile_desc = get_number_desc_by_type(metadata, PhoneNumberType::Mobile);
            if Self::desc_has_possible_number_data(mobile_desc) {
                possible_lengths.extend(if mobile_desc.possible_length.is_empty() {
                    &metadata.general_desc().possible_length
                } else {
                    &mobile_desc.possible_length
                });
                possible_lengths.sort();
                if local_lengths.is_empty() {
                    local_lengths = mobile_desc.possible_length_local_only.clone();
                } else {
                    local_lengths.extend(&mobile_desc.possible_length_local_only);
                    local_lengths.sort();
                }
            }
        }
        let first = *possible_lengths.first().expect("possible lengths");
        if first == -1 {
            return ValidationResult::InvalidLength;
        }
        let actual_length = number.len() as i32;
        if local_lengths.contains(&actual_length) {
            return ValidationResult::IsPossibleLocalOnly;
        }
        if first == actual_length {
            return ValidationResult::IsPossible;
        } else if first > actual_length {
            return ValidationResult::TooShort;
        } else if *possible_lengths.last().expect("possible lengths") < actual_length {
            return ValidationResult::TooLong;
        }
        if possible_lengths[1..].contains(&actual_length) {
            ValidationResult::IsPossible
        } else {
            ValidationResult::InvalidLength
        }
    }

    /// `isPossibleNumberWithReason(number)`.
    pub fn is_possible_number_with_reason(&self, number: &PhoneNumber) -> ValidationResult {
        self.is_possible_number_for_type_with_reason(number, PhoneNumberType::Unknown)
    }

    /// `isPossibleNumberForTypeWithReason(number, type)`.
    pub fn is_possible_number_for_type_with_reason(&self, number: &PhoneNumber, number_type: PhoneNumberType) -> ValidationResult {
        let national_number = get_national_significant_number(number);
        let country_code = number.country_code();
        if !self.has_valid_country_calling_code(country_code) {
            return ValidationResult::InvalidCountryCode;
        }
        let region_code = self.get_region_code_for_country_code(country_code);
        let metadata = self
            .get_metadata_for_region_or_calling_code(country_code, Some(region_code))
            .expect("metadata of a valid calling code");
        self.test_number_length(&national_number, metadata, number_type)
    }

    /// `isPossibleNumber(number, regionDialingFrom)`.
    pub fn is_possible_number_str(&self, number: &str, region_dialing_from: &str) -> bool {
        self.parse(number, Some(region_dialing_from)).is_ok_and(|n| self.is_possible_number(&n))
    }

    /// `truncateTooLongNumber(number)`.
    pub fn truncate_too_long_number(&self, number: &mut PhoneNumber) -> bool {
        if self.is_valid_number(number) {
            return true;
        }
        let mut copy = number.clone();
        let mut national_number = number.national_number();
        loop {
            national_number /= 10;
            copy.set_national_number(national_number);
            if self.is_possible_number_with_reason(&copy) == ValidationResult::TooShort || national_number == 0 {
                return false;
            }
            if self.is_valid_number(&copy) {
                break;
            }
        }
        number.set_national_number(national_number);
        true
    }

    /// `extractCountryCode(fullNumber, nationalNumber)` (package-private upstream).
    pub fn extract_country_code(&self, full_number: &str, national_number: &mut String) -> i32 {
        if full_number.is_empty() || full_number.starts_with('0') {
            return 0;
        }
        for i in 1..=MAX_LENGTH_COUNTRY_CODE.min(full_number.len()) {
            let Ok(potential) = full_number[..i].parse::<i32>() else {
                return 0;
            };
            if self.country_calling_code_to_region_code_map.contains_key(&potential) {
                national_number.push_str(&full_number[i..]);
                return potential;
            }
        }
        0
    }

    /// `maybeExtractCountryCode(number, defaultRegionMetadata, nationalNumber, keepRawInput, phoneNumber)`
    /// (package-private upstream).
    pub fn maybe_extract_country_code(
        &self,
        number: &str,
        default_region_metadata: Option<&PhoneMetadata>,
        national_number: &mut String,
        keep_raw_input: bool,
        phone_number: &mut PhoneNumber,
    ) -> Result<i32, NumberParseException> {
        if number.is_empty() {
            return Ok(0);
        }
        let mut full_number = number.to_string();
        let possible_country_idd_prefix = default_region_metadata.map_or("NonMatch", |m| m.international_prefix.as_str());
        let country_code_source = self.maybe_strip_international_prefix_and_normalize(&mut full_number, possible_country_idd_prefix);
        if keep_raw_input {
            phone_number.set_country_code_source(country_code_source);
        }
        if country_code_source != CountryCodeSource::FromDefaultCountry {
            if full_number.len() <= MIN_LENGTH_FOR_NSN {
                return Err(NumberParseException::new(
                    ErrorType::TooShortAfterIdd,
                    "Phone number had an IDD, but after this was not long enough to be a viable phone number.",
                ));
            }
            let potential_country_code = self.extract_country_code(&full_number, national_number);
            if potential_country_code != 0 {
                phone_number.set_country_code(potential_country_code);
                return Ok(potential_country_code);
            }
            return Err(NumberParseException::new(
                ErrorType::InvalidCountryCode,
                "Country calling code supplied was not recognised.",
            ));
        } else if let Some(metadata) = default_region_metadata {
            let default_country_code = metadata.country_code;
            let default_country_code_string = default_country_code.to_string();
            if let Some(rest) = full_number.strip_prefix(&default_country_code_string) {
                let mut potential_national_number = rest.to_string();
                let general_desc = metadata.general_desc();
                self.maybe_strip_national_prefix_and_carrier_code(&mut potential_national_number, metadata, None);
                if (!self.match_national_number(&full_number, general_desc, false)
                    && self.match_national_number(&potential_national_number, general_desc, false))
                    || self.test_number_length(&full_number, metadata, PhoneNumberType::Unknown) == ValidationResult::TooLong
                {
                    national_number.push_str(&potential_national_number);
                    if keep_raw_input {
                        phone_number.set_country_code_source(CountryCodeSource::FromNumberWithoutPlusSign);
                    }
                    phone_number.set_country_code(default_country_code);
                    return Ok(default_country_code);
                }
            }
        }
        phone_number.set_country_code(0);
        Ok(0)
    }

    /// `parsePrefixAsIdd(iddPattern, number)`.
    fn parse_prefix_as_idd(&self, idd_pattern: &JPattern, number: &mut String) -> bool {
        let Some(match_end) = idd_pattern.looking_at(number).map(|c| c.get(0).expect("match").end()) else {
            return false;
        };
        if let Some(digit) = patterns().capturing_digit.find(&number[match_end..]) {
            if normalize_digits_only(digit.get(1).expect("group").as_str()) == "0" {
                return false;
            }
        }
        number.replace_range(..match_end, "");
        true
    }

    /// `maybeStripInternationalPrefixAndNormalize(number, possibleIddPrefix)` (package-private upstream).
    pub fn maybe_strip_international_prefix_and_normalize(&self, number: &mut String, possible_idd_prefix: &str) -> CountryCodeSource {
        if number.is_empty() {
            return CountryCodeSource::FromDefaultCountry;
        }
        if let Some(end) = patterns().plus_chars.looking_at(number).map(|c| c.get(0).expect("match").end()) {
            number.replace_range(..end, "");
            *number = normalize(number);
            return CountryCodeSource::FromNumberWithPlusSign;
        }
        let idd_pattern = self.regex_cache.get(possible_idd_prefix);
        *number = normalize(number);
        if self.parse_prefix_as_idd(&idd_pattern, number) {
            CountryCodeSource::FromNumberWithIdd
        } else {
            CountryCodeSource::FromDefaultCountry
        }
    }

    /// `maybeStripNationalPrefixAndCarrierCode(number, metadata, carrierCode)` (package-private upstream).
    pub fn maybe_strip_national_prefix_and_carrier_code(
        &self,
        number: &mut String,
        metadata: &PhoneMetadata,
        carrier_code: Option<&mut String>,
    ) -> bool {
        let possible_national_prefix = &metadata.national_prefix_for_parsing;
        if number.is_empty() || possible_national_prefix.is_empty() {
            return false;
        }
        let prefix_pattern = self.regex_cache.get(possible_national_prefix);
        let Some(caps) = prefix_pattern.looking_at(number) else {
            return false;
        };
        let general_desc = metadata.general_desc();
        let is_viable_original_number = self.match_national_number(number, general_desc, false);
        let num_of_groups = prefix_pattern.group_count();
        let transform_rule = &metadata.national_prefix_transform_rule;
        let prefix_end = caps.get(0).expect("match").end();
        let last_group = caps.get(num_of_groups).map(|m| m.as_str().to_string());
        let first_group = caps.get(1).map(|m| m.as_str().to_string());
        if transform_rule.is_empty() || last_group.is_none() {
            if is_viable_original_number && !self.match_national_number(&number[prefix_end..], general_desc, false) {
                return false;
            }
            if let Some(carrier_code) = carrier_code {
                if num_of_groups > 0 && last_group.is_some() {
                    carrier_code.push_str(first_group.as_deref().unwrap_or("null"));
                }
            }
            number.replace_range(..prefix_end, "");
            true
        } else {
            // `prefixMatcher.replaceFirst(transformRule)`: the matcher was reset, so this is the first match anywhere,
            // which is the one `lookingAt` found.
            let transformed = prefix_pattern.replace_first(number, transform_rule);
            if is_viable_original_number && !self.match_national_number(&transformed, general_desc, false) {
                return false;
            }
            if let Some(carrier_code) = carrier_code {
                if num_of_groups > 1 {
                    carrier_code.push_str(first_group.as_deref().unwrap_or("null"));
                }
            }
            *number = transformed;
            true
        }
    }

    fn check_region_for_parsing(&self, number_to_parse: &str, default_region: Option<&str>) -> bool {
        if !self.is_valid_region_code(default_region)
            && (number_to_parse.is_empty() || patterns().plus_chars.looking_at(number_to_parse).is_none())
        {
            return false;
        }
        true
    }

    /// `parse(numberToParse, defaultRegion)`; `None` stands for Java's `null` region.
    pub fn parse(&self, number_to_parse: &str, default_region: Option<&str>) -> Result<PhoneNumber, NumberParseException> {
        let mut phone_number = PhoneNumber::new();
        self.parse_helper(number_to_parse, default_region, false, true, &mut phone_number)?;
        Ok(phone_number)
    }

    /// `parse(numberToParse, defaultRegion, phoneNumber)`: fills a number in place (fields it does not set stay).
    pub fn parse_into(
        &self,
        number_to_parse: &str,
        default_region: Option<&str>,
        phone_number: &mut PhoneNumber,
    ) -> Result<(), NumberParseException> {
        self.parse_helper(number_to_parse, default_region, false, true, phone_number)
    }

    /// `parseAndKeepRawInput(numberToParse, defaultRegion)`.
    pub fn parse_and_keep_raw_input(
        &self,
        number_to_parse: &str,
        default_region: Option<&str>,
    ) -> Result<PhoneNumber, NumberParseException> {
        let mut phone_number = PhoneNumber::new();
        self.parse_helper(number_to_parse, default_region, true, true, &mut phone_number)?;
        Ok(phone_number)
    }

    fn parse_helper(
        &self,
        number_to_parse: &str,
        default_region: Option<&str>,
        keep_raw_input: bool,
        check_region: bool,
        phone_number: &mut PhoneNumber,
    ) -> Result<(), NumberParseException> {
        if utf16_len(number_to_parse) > MAX_INPUT_STRING_LENGTH {
            return Err(NumberParseException::new(ErrorType::TooLong, "The string supplied was too long to parse."));
        }
        let mut national_number = build_national_number_for_parsing(number_to_parse)?;
        if !is_viable_phone_number(&national_number) {
            return Err(NumberParseException::new(
                ErrorType::NotANumber,
                "The string supplied did not seem to be a phone number.",
            ));
        }
        if check_region && !self.check_region_for_parsing(&national_number, default_region) {
            return Err(NumberParseException::new(ErrorType::InvalidCountryCode, "Missing or invalid default region."));
        }
        if keep_raw_input {
            phone_number.set_raw_input(number_to_parse);
        }
        let extension = maybe_strip_extension(&mut national_number);
        if !extension.is_empty() {
            phone_number.set_extension(extension);
        }
        let mut region_metadata = self.get_metadata_for_region_opt(default_region);
        let mut normalized_national_number = String::new();
        let mut country_code = match self.maybe_extract_country_code(
            &national_number,
            region_metadata,
            &mut normalized_national_number,
            keep_raw_input,
            phone_number,
        ) {
            Ok(code) => code,
            Err(e) => {
                let plus_end = patterns().plus_chars.looking_at(&national_number).map(|c| c.get(0).expect("match").end());
                match plus_end {
                    Some(end) if e.error_type() == ErrorType::InvalidCountryCode => {
                        let code = self.maybe_extract_country_code(
                            &national_number[end..],
                            region_metadata,
                            &mut normalized_national_number,
                            keep_raw_input,
                            phone_number,
                        )?;
                        if code == 0 {
                            return Err(NumberParseException::new(
                                ErrorType::InvalidCountryCode,
                                "Could not interpret numbers after plus-sign.",
                            ));
                        }
                        code
                    }
                    _ => return Err(e),
                }
            }
        };
        if country_code != 0 {
            let phone_number_region = self.get_region_code_for_country_code(country_code);
            if Some(phone_number_region) != default_region {
                region_metadata = self.get_metadata_for_region_or_calling_code(country_code, Some(phone_number_region));
            }
        } else {
            normalized_national_number.push_str(&normalize(&national_number));
            if default_region.is_some() {
                country_code = region_metadata.expect("metadata of the default region").country_code;
                phone_number.set_country_code(country_code);
            } else if keep_raw_input {
                phone_number.clear_country_code_source();
            }
        }
        if normalized_national_number.len() < MIN_LENGTH_FOR_NSN {
            return Err(NumberParseException::new(
                ErrorType::TooShortNsn,
                "The string supplied is too short to be a phone number.",
            ));
        }
        if let Some(metadata) = region_metadata {
            let mut carrier_code = String::new();
            let mut potential_national_number = normalized_national_number.clone();
            self.maybe_strip_national_prefix_and_carrier_code(&mut potential_national_number, metadata, Some(&mut carrier_code));
            let validation_result = self.test_number_length(&potential_national_number, metadata, PhoneNumberType::Unknown);
            if !matches!(
                validation_result,
                ValidationResult::TooShort | ValidationResult::IsPossibleLocalOnly | ValidationResult::InvalidLength
            ) {
                normalized_national_number = potential_national_number;
                if keep_raw_input && !carrier_code.is_empty() {
                    phone_number.set_preferred_domestic_carrier_code(carrier_code);
                }
            }
        }
        let length_of_national_number = normalized_national_number.len();
        if length_of_national_number < MIN_LENGTH_FOR_NSN {
            return Err(NumberParseException::new(
                ErrorType::TooShortNsn,
                "The string supplied is too short to be a phone number.",
            ));
        }
        if length_of_national_number > MAX_LENGTH_FOR_NSN {
            return Err(NumberParseException::new(
                ErrorType::TooLong,
                "The string supplied is too long to be a phone number.",
            ));
        }
        set_italian_leading_zeros_for_phone_number(&normalized_national_number, phone_number);
        phone_number.set_national_number(normalized_national_number.parse().expect("digits"));
        Ok(())
    }

    /// `canBeInternationallyDialled(number)`.
    pub fn can_be_internationally_dialled(&self, number: &PhoneNumber) -> bool {
        let region = self.get_region_code_for_number(number);
        let Some(metadata) = self.get_metadata_for_region_opt(region.as_deref()) else {
            return true;
        };
        !self.is_number_matching_desc(&get_national_significant_number(number), desc_or_empty(&metadata.no_international_dialling))
    }

    /// `isMobileNumberPortableRegion(regionCode)`.
    pub fn is_mobile_number_portable_region(&self, region_code: &str) -> bool {
        self.get_metadata_for_region(region_code).is_some_and(|m| m.mobile_number_portable_region)
    }

    /// Whether an international prefix is a single one (`SINGLE_INTERNATIONAL_PREFIX`), as the out-of-country
    /// formatters ask.
    pub fn is_single_international_prefix(prefix: &str) -> bool {
        patterns().single_international_prefix.matches(prefix)
    }
}

/// `isNumberGeographical(phoneNumberType, countryCallingCode)`.
pub fn is_number_geographical_type(phone_number_type: PhoneNumberType, country_calling_code: i32) -> bool {
    phone_number_type == PhoneNumberType::FixedLine
        || phone_number_type == PhoneNumberType::FixedLineOrMobile
        || ([52, 54, 55, 62, 86].contains(&country_calling_code) && phone_number_type == PhoneNumberType::Mobile)
}

fn desc_or_empty(desc: &Option<PhoneNumberDesc>) -> &PhoneNumberDesc {
    static EMPTY: OnceLock<PhoneNumberDesc> = OnceLock::new();
    desc.as_ref().unwrap_or_else(|| EMPTY.get_or_init(PhoneNumberDesc::default))
}

/// `getNumberDescByType(metadata, type)` (package-private upstream).
pub fn get_number_desc_by_type(metadata: &PhoneMetadata, number_type: PhoneNumberType) -> &PhoneNumberDesc {
    desc_or_empty(match number_type {
        PhoneNumberType::PremiumRate => &metadata.premium_rate,
        PhoneNumberType::TollFree => &metadata.toll_free,
        PhoneNumberType::Mobile => &metadata.mobile,
        PhoneNumberType::FixedLine | PhoneNumberType::FixedLineOrMobile => &metadata.fixed_line,
        PhoneNumberType::SharedCost => &metadata.shared_cost,
        PhoneNumberType::Voip => &metadata.voip,
        PhoneNumberType::PersonalNumber => &metadata.personal_number,
        PhoneNumberType::Pager => &metadata.pager,
        PhoneNumberType::Uan => &metadata.uan,
        PhoneNumberType::Voicemail => &metadata.voicemail,
        PhoneNumberType::Unknown => &metadata.general_desc,
    })
}

/// `getNationalSignificantNumber(number)`.
pub fn get_national_significant_number(number: &PhoneNumber) -> String {
    let mut out = String::new();
    if number.is_italian_leading_zero() && number.number_of_leading_zeros() > 0 {
        out.push_str(&"0".repeat(number.number_of_leading_zeros().min(10) as usize));
    }
    out.push_str(&number.national_number().to_string());
    out
}

/// `prefixNumberWithCountryCallingCode(countryCallingCode, numberFormat, formattedNumber)`.
fn prefix_number_with_country_calling_code(country_calling_code: i32, number_format: PhoneNumberFormat, formatted: &mut String) {
    let prefix = match number_format {
        PhoneNumberFormat::E164 => format!("{PLUS_SIGN}{country_calling_code}"),
        PhoneNumberFormat::International => format!("{PLUS_SIGN}{country_calling_code} "),
        PhoneNumberFormat::Rfc3966 => format!("{RFC3966_PREFIX}{PLUS_SIGN}{country_calling_code}-"),
        PhoneNumberFormat::National => return,
    };
    formatted.insert_str(0, &prefix);
}

/// `maybeAppendFormattedExtension(number, metadata, numberFormat, formattedNumber)`.
fn maybe_append_formatted_extension(
    number: &PhoneNumber,
    metadata: &PhoneMetadata,
    number_format: PhoneNumberFormat,
    formatted: &mut String,
) {
    if number.has_extension() && !number.extension().is_empty() {
        if number_format == PhoneNumberFormat::Rfc3966 {
            formatted.push_str(RFC3966_EXTN_PREFIX);
        } else if metadata.has_preferred_extn_prefix {
            formatted.push_str(&metadata.preferred_extn_prefix);
        } else {
            formatted.push_str(DEFAULT_EXTN_PREFIX);
        }
        formatted.push_str(number.extension());
    }
}

/// `maybeStripExtension(number)` (package-private upstream): removes an extension from the end of `number` and
/// returns it, or returns `""`.
pub fn maybe_strip_extension(number: &mut String) -> String {
    let Some(caps) = patterns().extn.find(number) else {
        return String::new();
    };
    let start = caps.get(0).expect("match").start();
    if !is_viable_phone_number(&number[..start]) {
        return String::new();
    }
    for i in 1..caps.len() {
        if let Some(m) = caps.get(i) {
            let extension = m.as_str().to_string();
            number.truncate(start);
            return extension;
        }
    }
    String::new()
}

/// `extractPhoneContext(numberToExtractFrom, indexOfPhoneContext)`.
fn extract_phone_context(number: &str, index_of_phone_context: Option<usize>) -> Option<&str> {
    let index = index_of_phone_context?;
    let start = index + RFC3966_PHONE_CONTEXT.len();
    if start >= number.len() {
        return Some("");
    }
    match number[start..].find(';') {
        Some(end) => Some(&number[start..start + end]),
        None => Some(&number[start..]),
    }
}

/// `isPhoneContextValid(phoneContext)`.
fn is_phone_context_valid(phone_context: Option<&str>) -> bool {
    let Some(context) = phone_context else {
        return true;
    };
    if context.is_empty() {
        return false;
    }
    let p = patterns();
    p.rfc3966_global_number_digits.matches(context) || p.rfc3966_domainname.matches(context)
}

/// `buildNationalNumberForParsing(numberToParse, nationalNumber)`.
fn build_national_number_for_parsing(number_to_parse: &str) -> Result<String, NumberParseException> {
    let mut national_number = String::new();
    let index_of_phone_context = number_to_parse.find(RFC3966_PHONE_CONTEXT);
    let phone_context = extract_phone_context(number_to_parse, index_of_phone_context);
    if !is_phone_context_valid(phone_context) {
        return Err(NumberParseException::new(ErrorType::NotANumber, "The phone-context value is invalid."));
    }
    if let Some(context) = phone_context {
        if context.starts_with(PLUS_SIGN) {
            national_number.push_str(context);
        }
        let index_of_rfc3966_prefix = number_to_parse.find(RFC3966_PREFIX);
        let index_of_national_number = index_of_rfc3966_prefix.map_or(0, |i| i + RFC3966_PREFIX.len());
        let end = index_of_phone_context.expect("phone context");
        // Java's `substring` throws when "tel:" comes after the phone-context.
        let part = number_to_parse.get(index_of_national_number..end).unwrap_or_else(|| {
            panic!("begin {index_of_national_number}, end {end}, length {}", number_to_parse.len())
        });
        national_number.push_str(part);
    } else {
        national_number.push_str(extract_possible_number(number_to_parse));
    }
    if let Some(index_of_isdn) = national_number.find(RFC3966_ISDN_SUBADDRESS) {
        if index_of_isdn > 0 {
            national_number.truncate(index_of_isdn);
        }
    }
    Ok(national_number)
}

/// `Pattern.split(input)`: trailing empty strings removed.
fn java_split<'s>(pattern: &JPattern, input: &'s str) -> Vec<&'s str> {
    let mut out = Vec::new();
    let mut last = 0;
    let mut rest = 0;
    while let Some(caps) = pattern.find(&input[rest..]) {
        let m = caps.get(0).expect("match");
        let (start, end) = (rest + m.start(), rest + m.end());
        if end == start {
            break;
        }
        out.push(&input[last..start]);
        last = end;
        rest = end;
        if rest >= input.len() {
            break;
        }
    }
    out.push(&input[last..]);
    while out.last().is_some_and(|s| s.is_empty()) {
        out.pop();
    }
    out
}
