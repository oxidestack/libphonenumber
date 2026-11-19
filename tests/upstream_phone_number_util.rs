//! Ports of libphonenumber 9.0.41's `PhoneNumberUtilTest` (java/libphonenumber/test/…/PhoneNumberUtilTest.java),
//! run on upstream's test metadata (`PhoneNumberUtil::for_testing()`, `TestMetadataTestCase`). Methods whose
//! subject is not ported (out-of-country formatting, `formatInOriginalFormat`, mobile dialing, `isNumberMatch`)
//! are listed in docs/test-coverage.md.

use libphonenumber::PhoneNumberFormat::{E164, International, National, Rfc3966};
use libphonenumber::{
    CountryCodeSource, ErrorType, NumberFormat, NumberParseException, PhoneNumber, PhoneNumberType, PhoneNumberUtil,
    ValidationResult,
};

fn util() -> &'static PhoneNumberUtil {
    PhoneNumberUtil::for_testing()
}

fn num(country_code: i32, national_number: u64) -> PhoneNumber {
    let mut n = PhoneNumber::new();
    n.set_country_code(country_code).set_national_number(national_number);
    n
}

fn parse(number: &str, region: &str) -> PhoneNumber {
    util().parse(number, Some(region)).unwrap_or_else(|e| panic!("parse {number:?} {region}: {e}"))
}

fn parse_err(number: &str, region: Option<&str>) -> NumberParseException {
    match util().parse(number, region) {
        Ok(n) => panic!("{number:?} should not parse, got {n}"),
        Err(e) => e,
    }
}

// The numbers at the top of PhoneNumberUtilTest.
fn alpha_numeric_number() -> PhoneNumber {
    num(1, 80074935247)
}
fn ar_mobile() -> PhoneNumber {
    num(54, 91187654321)
}
fn ar_number() -> PhoneNumber {
    num(54, 1187654321)
}
fn au_number() -> PhoneNumber {
    num(61, 236618300)
}
fn bs_mobile() -> PhoneNumber {
    num(1, 2423570000)
}
fn bs_number() -> PhoneNumber {
    num(1, 2423651234)
}
fn de_number() -> PhoneNumber {
    num(49, 30123456)
}
fn de_short_number() -> PhoneNumber {
    num(49, 1234)
}
fn gb_mobile() -> PhoneNumber {
    num(44, 7912345678)
}
fn gb_number() -> PhoneNumber {
    num(44, 2070313000)
}
fn it_mobile() -> PhoneNumber {
    num(39, 345678901)
}
fn it_number() -> PhoneNumber {
    let mut n = num(39, 236618300);
    n.set_italian_leading_zero(true);
    n
}
fn jp_star_number() -> PhoneNumber {
    num(81, 2345)
}
fn mx_mobile1() -> PhoneNumber {
    num(52, 12345678900)
}
fn mx_mobile2() -> PhoneNumber {
    num(52, 15512345678)
}
fn mx_number1() -> PhoneNumber {
    num(52, 3312345678)
}
fn mx_number2() -> PhoneNumber {
    num(52, 8211234567)
}
fn nz_number() -> PhoneNumber {
    num(64, 33316005)
}
fn sg_number() -> PhoneNumber {
    num(65, 65218000)
}
fn us_long_number() -> PhoneNumber {
    num(1, 65025300001)
}
fn us_number() -> PhoneNumber {
    num(1, 6502530000)
}
fn us_premium() -> PhoneNumber {
    num(1, 9002530000)
}
fn us_local_number() -> PhoneNumber {
    num(1, 2530000)
}
fn us_short_by_one_number() -> PhoneNumber {
    num(1, 650253000)
}
fn us_tollfree() -> PhoneNumber {
    num(1, 8002530000)
}
fn us_spoof() -> PhoneNumber {
    num(1, 0)
}
fn us_spoof_with_raw_input() -> PhoneNumber {
    let mut n = num(1, 0);
    n.set_raw_input("000-000-0000");
    n
}
fn international_toll_free() -> PhoneNumber {
    num(800, 12345678)
}
fn international_toll_free_too_long() -> PhoneNumber {
    num(800, 123456789)
}
fn universal_premium_rate() -> PhoneNumber {
    num(979, 123456789)
}
fn unknown_country_code_no_raw_input() -> PhoneNumber {
    num(2, 12345)
}

/// PhoneNumberUtilTest#testGetSupportedRegions
#[test]
fn get_supported_regions() {
    assert!(!util().get_supported_regions().is_empty());
}

/// PhoneNumberUtilTest#testGetSupportedGlobalNetworkCallingCodes
#[test]
fn get_supported_global_network_calling_codes() {
    let codes = util().get_supported_global_network_calling_codes();
    assert!(!codes.is_empty());
    for &code in codes {
        assert!(code > 0);
        assert_eq!("001", util().get_region_code_for_country_code(code));
    }
}

/// PhoneNumberUtilTest#testGetSupportedCallingCodes
#[test]
fn get_supported_calling_codes() {
    let codes = util().get_supported_calling_codes();
    assert!(!codes.is_empty());
    for &code in codes {
        assert!(code > 0);
        assert_ne!("ZZ", util().get_region_code_for_country_code(code));
    }
    assert!(codes.len() > util().get_supported_global_network_calling_codes().len());
    assert!(codes.contains(&979));
}

/// PhoneNumberUtilTest#testGetInstanceLoadBadMetadata
#[test]
fn get_instance_load_bad_metadata() {
    assert!(util().get_metadata_for_region("No Such Region").is_none());
    assert!(util().get_metadata_for_non_geographical_region(-1).is_none());
}

/// PhoneNumberUtilTest#testGetSupportedTypesForRegion
#[test]
fn get_supported_types_for_region() {
    let br = util().get_supported_types_for_region("BR");
    assert!(br.contains(&PhoneNumberType::FixedLine));
    assert!(!br.contains(&PhoneNumberType::Mobile));
    assert!(!br.contains(&PhoneNumberType::Unknown));
    let us = util().get_supported_types_for_region("US");
    assert!(us.contains(&PhoneNumberType::FixedLine));
    assert!(us.contains(&PhoneNumberType::Mobile));
    assert!(!us.contains(&PhoneNumberType::FixedLineOrMobile));
    assert_eq!(0, util().get_supported_types_for_region("ZZ").len());
}

/// PhoneNumberUtilTest#testGetSupportedTypesForNonGeoEntity
#[test]
fn get_supported_types_for_non_geo_entity() {
    assert_eq!(0, util().get_supported_types_for_non_geo_entity(999).len());
    let types = util().get_supported_types_for_non_geo_entity(979);
    assert!(types.contains(&PhoneNumberType::PremiumRate));
    assert!(!types.contains(&PhoneNumberType::Mobile));
    assert!(!types.contains(&PhoneNumberType::Unknown));
}

/// PhoneNumberUtilTest#testGetInstanceLoadUSMetadata
#[test]
fn get_instance_load_us_metadata() {
    let m = util().get_metadata_for_region("US").unwrap();
    assert_eq!("US", m.id);
    assert_eq!(1, m.country_code);
    assert_eq!("011", m.international_prefix);
    assert!(m.has_national_prefix);
    assert_eq!(2, m.number_format.len());
    assert_eq!("(\\d{3})(\\d{3})(\\d{4})", m.number_format[1].pattern);
    assert_eq!("$1 $2 $3", m.number_format[1].format);
    assert_eq!("[13-689]\\d{9}|2[0-35-9]\\d{8}", m.general_desc().national_number_pattern);
    assert_eq!("[13-689]\\d{9}|2[0-35-9]\\d{8}", m.fixed_line.as_ref().unwrap().national_number_pattern);
    assert_eq!(1, m.general_desc().possible_length.len());
    assert_eq!(10, m.general_desc().possible_length[0]);
    assert_eq!(0, m.toll_free.as_ref().unwrap().possible_length.len());
    assert_eq!("900\\d{7}", m.premium_rate.as_ref().unwrap().national_number_pattern);
    assert!(!m.shared_cost.as_ref().unwrap().has_national_number_pattern);
}

/// PhoneNumberUtilTest#testGetInstanceLoadDEMetadata
#[test]
fn get_instance_load_de_metadata() {
    let m = util().get_metadata_for_region("DE").unwrap();
    assert_eq!("DE", m.id);
    assert_eq!(49, m.country_code);
    assert_eq!("00", m.international_prefix);
    assert_eq!("0", m.national_prefix);
    assert_eq!(6, m.number_format.len());
    assert_eq!(1, m.number_format[5].leading_digits_pattern.len());
    assert_eq!("900", m.number_format[5].leading_digits_pattern[0]);
    assert_eq!("(\\d{3})(\\d{3,4})(\\d{4})", m.number_format[5].pattern);
    assert_eq!("$1 $2 $3", m.number_format[5].format);
    assert_eq!(2, m.general_desc().possible_length_local_only.len());
    assert_eq!(8, m.general_desc().possible_length.len());
    let fixed_line = m.fixed_line.as_ref().unwrap();
    assert_eq!(0, fixed_line.possible_length.len());
    assert_eq!(2, m.mobile.as_ref().unwrap().possible_length.len());
    assert_eq!("(?:[24-6]\\d{2}|3[03-9]\\d|[789](?:0[2-9]|[1-9]\\d))\\d{1,8}", fixed_line.national_number_pattern);
    assert_eq!("30123456", fixed_line.example_number);
    assert_eq!(10, m.toll_free.as_ref().unwrap().possible_length[0]);
    assert_eq!("900([135]\\d{6}|9\\d{7})", m.premium_rate.as_ref().unwrap().national_number_pattern);
}

/// PhoneNumberUtilTest#testGetInstanceLoadARMetadata
#[test]
fn get_instance_load_ar_metadata() {
    let m = util().get_metadata_for_region("AR").unwrap();
    assert_eq!("AR", m.id);
    assert_eq!(54, m.country_code);
    assert_eq!("00", m.international_prefix);
    assert_eq!("0", m.national_prefix);
    assert_eq!("0(?:(11|343|3715)15)?", m.national_prefix_for_parsing);
    assert_eq!("9$1", m.national_prefix_transform_rule);
    assert_eq!("$2 15 $3-$4", m.number_format[2].format);
    assert_eq!("(\\d)(\\d{4})(\\d{2})(\\d{4})", m.number_format[3].pattern);
    assert_eq!("(\\d)(\\d{4})(\\d{2})(\\d{4})", m.intl_number_format[3].pattern);
    assert_eq!("$1 $2 $3 $4", m.intl_number_format[3].format);
}

/// PhoneNumberUtilTest#testGetInstanceLoadInternationalTollFreeMetadata
#[test]
fn get_instance_load_international_toll_free_metadata() {
    let m = util().get_metadata_for_non_geographical_region(800).unwrap();
    assert_eq!("001", m.id);
    assert_eq!(800, m.country_code);
    assert_eq!("$1 $2", m.number_format[0].format);
    assert_eq!("(\\d{4})(\\d{4})", m.number_format[0].pattern);
    assert_eq!(0, m.general_desc().possible_length_local_only.len());
    assert_eq!(1, m.general_desc().possible_length.len());
    assert_eq!("12345678", m.toll_free.as_ref().unwrap().example_number);
}

/// PhoneNumberUtilTest#testIsNumberGeographical
#[test]
fn is_number_geographical() {
    assert!(!util().is_number_geographical(&bs_mobile()));
    assert!(util().is_number_geographical(&au_number()));
    assert!(!util().is_number_geographical(&international_toll_free()));
    assert!(util().is_number_geographical(&ar_mobile()));
    assert!(util().is_number_geographical(&mx_mobile1()));
    assert!(util().is_number_geographical(&mx_mobile2()));
}

/// PhoneNumberUtilTest#testGetLengthOfGeographicalAreaCode
#[test]
fn get_length_of_geographical_area_code() {
    let u = util();
    assert_eq!(3, u.get_length_of_geographical_area_code(&us_number()));
    assert_eq!(0, u.get_length_of_geographical_area_code(&us_tollfree()));
    assert_eq!(2, u.get_length_of_geographical_area_code(&gb_number()));
    assert_eq!(0, u.get_length_of_geographical_area_code(&gb_mobile()));
    assert_eq!(2, u.get_length_of_geographical_area_code(&ar_number()));
    assert_eq!(3, u.get_length_of_geographical_area_code(&ar_mobile()));
    assert_eq!(1, u.get_length_of_geographical_area_code(&au_number()));
    assert_eq!(2, u.get_length_of_geographical_area_code(&it_number()));
    assert_eq!(2, u.get_length_of_geographical_area_code(&mx_number1()));
    assert_eq!(0, u.get_length_of_geographical_area_code(&sg_number()));
    assert_eq!(0, u.get_length_of_geographical_area_code(&us_short_by_one_number()));
    assert_eq!(0, u.get_length_of_geographical_area_code(&international_toll_free()));
    assert_eq!(0, u.get_length_of_geographical_area_code(&num(86, 18912341234)));
}

/// PhoneNumberUtilTest#testGetLengthOfNationalDestinationCode
#[test]
fn get_length_of_national_destination_code() {
    let u = util();
    assert_eq!(3, u.get_length_of_national_destination_code(&us_number()));
    assert_eq!(3, u.get_length_of_national_destination_code(&us_tollfree()));
    assert_eq!(2, u.get_length_of_national_destination_code(&gb_number()));
    assert_eq!(4, u.get_length_of_national_destination_code(&gb_mobile()));
    assert_eq!(2, u.get_length_of_national_destination_code(&ar_number()));
    assert_eq!(3, u.get_length_of_national_destination_code(&ar_mobile()));
    assert_eq!(1, u.get_length_of_national_destination_code(&au_number()));
    assert_eq!(4, u.get_length_of_national_destination_code(&sg_number()));
    assert_eq!(0, u.get_length_of_national_destination_code(&us_short_by_one_number()));
    assert_eq!(0, u.get_length_of_national_destination_code(&num(123, 6502530000)));
    assert_eq!(4, u.get_length_of_national_destination_code(&international_toll_free()));
    assert_eq!(3, u.get_length_of_national_destination_code(&num(86, 18912341234)));
}

/// PhoneNumberUtilTest#testGetCountryMobileToken
#[test]
fn get_country_mobile_token() {
    assert_eq!("9", libphonenumber::get_country_mobile_token(util().get_country_code_for_region("AR")));
    assert_eq!("", libphonenumber::get_country_mobile_token(util().get_country_code_for_region("SE")));
}

/// PhoneNumberUtilTest#testGetNationalSignificantNumber
#[test]
fn get_national_significant_number() {
    use libphonenumber::get_national_significant_number as nsn;
    assert_eq!("6502530000", nsn(&us_number()));
    assert_eq!("345678901", nsn(&it_mobile()));
    assert_eq!("0236618300", nsn(&it_number()));
    assert_eq!("12345678", nsn(&international_toll_free()));
}

/// PhoneNumberUtilTest#testGetNationalSignificantNumber_ManyLeadingZeros
#[test]
fn get_national_significant_number_many_leading_zeros() {
    let mut number = num(1, 650);
    number.set_italian_leading_zero(true).set_number_of_leading_zeros(2);
    assert_eq!("00650", libphonenumber::get_national_significant_number(&number));
    number.set_number_of_leading_zeros(-3);
    assert_eq!("650", libphonenumber::get_national_significant_number(&number));
}

/// PhoneNumberUtilTest#testGetExampleNumber
#[test]
fn get_example_number() {
    let u = util();
    assert_eq!(Some(de_number()), u.get_example_number("DE"));
    assert_eq!(Some(de_number()), u.get_example_number_for_type("DE", PhoneNumberType::FixedLine));
    assert_eq!(Some(de_number()), u.get_example_number_for_type("DE", PhoneNumberType::FixedLineOrMobile));
    assert!(u.get_example_number_for_type("US", PhoneNumberType::FixedLine).is_some());
    assert!(u.get_example_number_for_type("US", PhoneNumberType::Mobile).is_some());
    assert!(u.get_example_number_for_type("US", PhoneNumberType::Voicemail).is_none());
    assert!(u.get_example_number_for_type("CS", PhoneNumberType::Mobile).is_none());
    assert!(u.get_example_number("001").is_none());
}

/// PhoneNumberUtilTest#testGetInvalidExampleNumber
#[test]
fn get_invalid_example_number() {
    let u = util();
    assert!(u.get_invalid_example_number("001").is_none());
    assert!(u.get_invalid_example_number("CS").is_none());
    let us_invalid = u.get_invalid_example_number("US").unwrap();
    assert_eq!(1, us_invalid.country_code());
    assert_ne!(0, us_invalid.national_number());
}

/// PhoneNumberUtilTest#testGetExampleNumberForNonGeoEntity
#[test]
fn get_example_number_for_non_geo_entity() {
    assert_eq!(Some(international_toll_free()), util().get_example_number_for_non_geo_entity(800));
    assert_eq!(Some(universal_premium_rate()), util().get_example_number_for_non_geo_entity(979));
}

/// PhoneNumberUtilTest#testGetExampleNumberWithoutRegion (`getExampleNumberForType(type)` is
/// `get_example_number_for_any_region`)
#[test]
fn get_example_number_without_region() {
    assert!(util().get_example_number_for_any_region(PhoneNumberType::FixedLine).is_some());
    assert!(util().get_example_number_for_any_region(PhoneNumberType::Mobile).is_some());
    assert!(util().get_example_number_for_any_region(PhoneNumberType::PremiumRate).is_some());
}

/// PhoneNumberUtilTest#testConvertAlphaCharactersInNumber
#[test]
fn convert_alpha_characters_in_number() {
    assert_eq!("1800-222-333", libphonenumber::convert_alpha_characters_in_number("1800-ABC-DEF"));
}

/// PhoneNumberUtilTest#testNormaliseRemovePunctuation
#[test]
fn normalise_remove_punctuation() {
    assert_eq!("03456234", libphonenumber::normalize("034-56&+#2\u{00AD}34"));
}

/// PhoneNumberUtilTest#testNormaliseReplaceAlphaCharacters
#[test]
fn normalise_replace_alpha_characters() {
    assert_eq!("034426486479", libphonenumber::normalize("034-I-am-HUNGRY"));
}

/// PhoneNumberUtilTest#testNormaliseOtherDigits
#[test]
fn normalise_other_digits() {
    assert_eq!("255", libphonenumber::normalize("\u{FF12}5\u{0665}"));
    assert_eq!("520", libphonenumber::normalize("\u{06F5}2\u{06F0}"));
}

/// PhoneNumberUtilTest#testNormaliseStripAlphaCharacters
#[test]
fn normalise_strip_alpha_characters() {
    assert_eq!("03456234", libphonenumber::normalize_digits_only("034-56&+a#234"));
}

/// PhoneNumberUtilTest#testNormaliseStripNonDiallableCharacters
#[test]
fn normalise_strip_non_diallable_characters() {
    assert_eq!("03*456+1#234", libphonenumber::normalize_diallable_chars_only("03*4-56&+1a#234"));
}

/// PhoneNumberUtilTest#testFormatUSNumber
#[test]
fn format_us_number() {
    let u = util();
    assert_eq!("650 253 0000", u.format(&us_number(), National));
    assert_eq!("+1 650 253 0000", u.format(&us_number(), International));
    assert_eq!("800 253 0000", u.format(&us_tollfree(), National));
    assert_eq!("+1 800 253 0000", u.format(&us_tollfree(), International));
    assert_eq!("900 253 0000", u.format(&us_premium(), National));
    assert_eq!("+1 900 253 0000", u.format(&us_premium(), International));
    assert_eq!("tel:+1-900-253-0000", u.format(&us_premium(), Rfc3966));
    assert_eq!("000-000-0000", u.format(&us_spoof_with_raw_input(), National));
    assert_eq!("0", u.format(&us_spoof(), National));
}

/// PhoneNumberUtilTest#testFormatAUShortCodeNumber
#[test]
fn format_au_short_code_number() {
    let u = util();
    let au_short_code = parse("000", "AU");
    assert_eq!("+61000", u.format(&au_short_code, E164));
    let mut pg_short_code = num(675, 0);
    pg_short_code.set_raw_input("+675000");
    assert_eq!("+675000", u.format(&pg_short_code, E164));
}

/// PhoneNumberUtilTest#testFormatBSNumber
#[test]
fn format_bs_number() {
    assert_eq!("242 365 1234", util().format(&bs_number(), National));
    assert_eq!("+1 242 365 1234", util().format(&bs_number(), International));
}

/// PhoneNumberUtilTest#testFormatGBNumber
#[test]
fn format_gb_number() {
    let u = util();
    assert_eq!("(020) 7031 3000", u.format(&gb_number(), National));
    assert_eq!("+44 20 7031 3000", u.format(&gb_number(), International));
    assert_eq!("(07912) 345 678", u.format(&gb_mobile(), National));
    assert_eq!("+44 7912 345 678", u.format(&gb_mobile(), International));
}

/// PhoneNumberUtilTest#testFormatDENumber
#[test]
fn format_de_number() {
    let u = util();
    let mut de = num(49, 301234);
    assert_eq!("030/1234", u.format(&de, National));
    assert_eq!("+49 30/1234", u.format(&de, International));
    assert_eq!("tel:+49-30-1234", u.format(&de, Rfc3966));
    de.clear();
    de.set_country_code(49).set_national_number(291123);
    assert_eq!("0291 123", u.format(&de, National));
    assert_eq!("+49 291 123", u.format(&de, International));
    de.clear();
    de.set_country_code(49).set_national_number(29112345678);
    assert_eq!("0291 12345678", u.format(&de, National));
    assert_eq!("+49 291 12345678", u.format(&de, International));
    de.clear();
    de.set_country_code(49).set_national_number(912312345);
    assert_eq!("09123 12345", u.format(&de, National));
    assert_eq!("+49 9123 12345", u.format(&de, International));
    de.clear();
    de.set_country_code(49).set_national_number(80212345);
    assert_eq!("08021 2345", u.format(&de, National));
    assert_eq!("+49 8021 2345", u.format(&de, International));
    assert_eq!("1234", u.format(&de_short_number(), National));
    assert_eq!("+49 1234", u.format(&de_short_number(), International));
    de.clear();
    de.set_country_code(49).set_national_number(41341234);
    assert_eq!("04134 1234", u.format(&de, National));
}

/// PhoneNumberUtilTest#testFormatITNumber
#[test]
fn format_it_number() {
    let u = util();
    assert_eq!("02 3661 8300", u.format(&it_number(), National));
    assert_eq!("+39 02 3661 8300", u.format(&it_number(), International));
    assert_eq!("+390236618300", u.format(&it_number(), E164));
    assert_eq!("345 678 901", u.format(&it_mobile(), National));
    assert_eq!("+39 345 678 901", u.format(&it_mobile(), International));
    assert_eq!("+39345678901", u.format(&it_mobile(), E164));
}

/// PhoneNumberUtilTest#testFormatAUNumber
#[test]
fn format_au_number() {
    let u = util();
    assert_eq!("02 3661 8300", u.format(&au_number(), National));
    assert_eq!("+61 2 3661 8300", u.format(&au_number(), International));
    assert_eq!("+61236618300", u.format(&au_number(), E164));
    let au = num(61, 1800123456);
    assert_eq!("1800 123 456", u.format(&au, National));
    assert_eq!("+61 1800 123 456", u.format(&au, International));
    assert_eq!("+611800123456", u.format(&au, E164));
}

/// PhoneNumberUtilTest#testFormatARNumber
#[test]
fn format_ar_number() {
    let u = util();
    assert_eq!("011 8765-4321", u.format(&ar_number(), National));
    assert_eq!("+54 11 8765-4321", u.format(&ar_number(), International));
    assert_eq!("+541187654321", u.format(&ar_number(), E164));
    assert_eq!("011 15 8765-4321", u.format(&ar_mobile(), National));
    assert_eq!("+54 9 11 8765 4321", u.format(&ar_mobile(), International));
    assert_eq!("+5491187654321", u.format(&ar_mobile(), E164));
}

/// PhoneNumberUtilTest#testFormatMXNumber
#[test]
fn format_mx_number() {
    let u = util();
    assert_eq!("045 234 567 8900", u.format(&mx_mobile1(), National));
    assert_eq!("+52 1 234 567 8900", u.format(&mx_mobile1(), International));
    assert_eq!("+5212345678900", u.format(&mx_mobile1(), E164));
    assert_eq!("045 55 1234 5678", u.format(&mx_mobile2(), National));
    assert_eq!("+52 1 55 1234 5678", u.format(&mx_mobile2(), International));
    assert_eq!("+5215512345678", u.format(&mx_mobile2(), E164));
    assert_eq!("01 33 1234 5678", u.format(&mx_number1(), National));
    assert_eq!("+52 33 1234 5678", u.format(&mx_number1(), International));
    assert_eq!("+523312345678", u.format(&mx_number1(), E164));
    assert_eq!("01 821 123 4567", u.format(&mx_number2(), National));
    assert_eq!("+52 821 123 4567", u.format(&mx_number2(), International));
    assert_eq!("+528211234567", u.format(&mx_number2(), E164));
}

/// PhoneNumberUtilTest#testFormatWithCarrierCode
#[test]
fn format_with_carrier_code() {
    let u = util();
    let ar = num(54, 92234654321);
    assert_eq!("02234 65-4321", u.format(&ar, National));
    assert_eq!("02234 14 65-4321", u.format_national_number_with_carrier_code(&ar, "14"));
    assert_eq!("02234 65-4321", u.format_national_number_with_carrier_code(&ar, ""));
    assert_eq!("+5492234654321", u.format(&ar, E164));
    assert_eq!("650 253 0000", u.format_national_number_with_carrier_code(&us_number(), "15"));
    assert_eq!("12345", u.format_national_number_with_carrier_code(&unknown_country_code_no_raw_input(), "89"));
}

/// PhoneNumberUtilTest#testFormatWithPreferredCarrierCode
#[test]
fn format_with_preferred_carrier_code() {
    let u = util();
    let mut ar = num(54, 91234125678);
    assert_eq!("01234 15 12-5678", u.format_national_number_with_preferred_carrier_code(&ar, "15"));
    assert_eq!("01234 12-5678", u.format_national_number_with_preferred_carrier_code(&ar, ""));
    ar.set_preferred_domestic_carrier_code("19");
    assert_eq!("01234 12-5678", u.format(&ar, National));
    assert_eq!("01234 19 12-5678", u.format_national_number_with_preferred_carrier_code(&ar, "15"));
    assert_eq!("01234 19 12-5678", u.format_national_number_with_preferred_carrier_code(&ar, ""));
    ar.set_preferred_domestic_carrier_code(" ");
    assert_eq!("01234   12-5678", u.format_national_number_with_preferred_carrier_code(&ar, "15"));
    ar.set_preferred_domestic_carrier_code("");
    assert_eq!("01234 15 12-5678", u.format_national_number_with_preferred_carrier_code(&ar, "15"));
    let mut us = num(1, 4241231234);
    us.set_preferred_domestic_carrier_code("99");
    assert_eq!("424 123 1234", u.format(&us, National));
    assert_eq!("424 123 1234", u.format_national_number_with_preferred_carrier_code(&us, "15"));
}

/// PhoneNumberUtilTest#testFormatByPattern
#[test]
fn format_by_pattern() {
    let u = util();
    let mut f = NumberFormat {
        pattern: "(\\d{3})(\\d{3})(\\d{4})".into(),
        format: "($1) $2-$3".into(),
        ..NumberFormat::default()
    };
    assert_eq!("(650) 253-0000", u.format_by_pattern(&us_number(), National, std::slice::from_ref(&f)));
    assert_eq!("+1 (650) 253-0000", u.format_by_pattern(&us_number(), International, std::slice::from_ref(&f)));
    assert_eq!("tel:+1-650-712-9823", u.format_by_pattern(&num(1, 6507129823), Rfc3966, std::slice::from_ref(&f)));

    f.national_prefix_formatting_rule = "$NP ($FG)".into();
    f.has_national_prefix_formatting_rule = true;
    f.format = "$1 $2-$3".into();
    assert_eq!("1 (242) 365-1234", u.format_by_pattern(&bs_number(), National, std::slice::from_ref(&f)));
    assert_eq!("+1 242 365-1234", u.format_by_pattern(&bs_number(), International, std::slice::from_ref(&f)));

    f.pattern = "(\\d{2})(\\d{5})(\\d{3})".into();
    f.format = "$1-$2 $3".into();
    assert_eq!("02-36618 300", u.format_by_pattern(&it_number(), National, std::slice::from_ref(&f)));
    assert_eq!("+39 02-36618 300", u.format_by_pattern(&it_number(), International, std::slice::from_ref(&f)));

    f.national_prefix_formatting_rule = "$NP$FG".into();
    f.pattern = "(\\d{2})(\\d{4})(\\d{4})".into();
    f.format = "$1 $2 $3".into();
    assert_eq!("020 7031 3000", u.format_by_pattern(&gb_number(), National, std::slice::from_ref(&f)));

    f.national_prefix_formatting_rule = "($NP$FG)".into();
    assert_eq!("(020) 7031 3000", u.format_by_pattern(&gb_number(), National, std::slice::from_ref(&f)));

    f.national_prefix_formatting_rule = "".into();
    assert_eq!("20 7031 3000", u.format_by_pattern(&gb_number(), National, std::slice::from_ref(&f)));
    assert_eq!("+44 20 7031 3000", u.format_by_pattern(&gb_number(), International, std::slice::from_ref(&f)));
}

/// PhoneNumberUtilTest#testFormatE164Number
#[test]
fn format_e164_number() {
    let u = util();
    assert_eq!("+16502530000", u.format(&us_number(), E164));
    assert_eq!("+4930123456", u.format(&de_number(), E164));
    assert_eq!("+80012345678", u.format(&international_toll_free(), E164));
}

/// PhoneNumberUtilTest#testFormatNumberWithExtension
#[test]
fn format_number_with_extension() {
    let u = util();
    let mut nz = nz_number();
    nz.set_extension("1234");
    assert_eq!("03-331 6005 ext. 1234", u.format(&nz, National));
    assert_eq!("tel:+64-3-331-6005;ext=1234", u.format(&nz, Rfc3966));
    let mut us = us_number();
    us.set_extension("4567");
    assert_eq!("650 253 0000 extn. 4567", u.format(&us, National));
}

/// PhoneNumberUtilTest#testIsPremiumRate
#[test]
fn is_premium_rate() {
    let u = util();
    assert_eq!(PhoneNumberType::PremiumRate, u.get_number_type(&us_premium()));
    for (cc, nn) in [(39, 892123), (44, 9187654321), (49, 9001654321), (49, 90091234567)] {
        assert_eq!(PhoneNumberType::PremiumRate, u.get_number_type(&num(cc, nn)), "{cc} {nn}");
    }
    assert_eq!(PhoneNumberType::PremiumRate, u.get_number_type(&universal_premium_rate()));
}

/// PhoneNumberUtilTest#testIsTollFree
#[test]
fn is_toll_free() {
    let u = util();
    for (cc, nn) in [(1, 8881234567), (39, 803123), (44, 8012345678), (49, 8001234567)] {
        assert_eq!(PhoneNumberType::TollFree, u.get_number_type(&num(cc, nn)), "{cc} {nn}");
    }
    assert_eq!(PhoneNumberType::TollFree, u.get_number_type(&international_toll_free()));
}

/// PhoneNumberUtilTest#testIsMobile
#[test]
fn is_mobile() {
    let u = util();
    assert_eq!(PhoneNumberType::Mobile, u.get_number_type(&bs_mobile()));
    assert_eq!(PhoneNumberType::Mobile, u.get_number_type(&gb_mobile()));
    assert_eq!(PhoneNumberType::Mobile, u.get_number_type(&it_mobile()));
    assert_eq!(PhoneNumberType::Mobile, u.get_number_type(&ar_mobile()));
    assert_eq!(PhoneNumberType::Mobile, u.get_number_type(&num(49, 15123456789)));
}

/// PhoneNumberUtilTest#testIsFixedLine
#[test]
fn is_fixed_line() {
    let u = util();
    assert_eq!(PhoneNumberType::FixedLine, u.get_number_type(&bs_number()));
    assert_eq!(PhoneNumberType::FixedLine, u.get_number_type(&it_number()));
    assert_eq!(PhoneNumberType::FixedLine, u.get_number_type(&gb_number()));
    assert_eq!(PhoneNumberType::FixedLine, u.get_number_type(&de_number()));
}

/// PhoneNumberUtilTest#testIsFixedLineAndMobile
#[test]
fn is_fixed_line_and_mobile() {
    assert_eq!(PhoneNumberType::FixedLineOrMobile, util().get_number_type(&us_number()));
    assert_eq!(PhoneNumberType::FixedLineOrMobile, util().get_number_type(&num(54, 1987654321)));
}

/// PhoneNumberUtilTest#testIsSharedCost
#[test]
fn is_shared_cost() {
    assert_eq!(PhoneNumberType::SharedCost, util().get_number_type(&num(44, 8431231234)));
}

/// PhoneNumberUtilTest#testIsVoip
#[test]
fn is_voip() {
    assert_eq!(PhoneNumberType::Voip, util().get_number_type(&num(44, 5631231234)));
}

/// PhoneNumberUtilTest#testIsPersonalNumber
#[test]
fn is_personal_number() {
    assert_eq!(PhoneNumberType::PersonalNumber, util().get_number_type(&num(44, 7031231234)));
}

/// PhoneNumberUtilTest#testIsUnknown
#[test]
fn is_unknown() {
    assert_eq!(PhoneNumberType::Unknown, util().get_number_type(&us_local_number()));
}

/// PhoneNumberUtilTest#testIsValidNumber
#[test]
fn is_valid_number() {
    let u = util();
    assert!(u.is_valid_number(&us_number()));
    assert!(u.is_valid_number(&it_number()));
    assert!(u.is_valid_number(&gb_mobile()));
    assert!(u.is_valid_number(&international_toll_free()));
    assert!(u.is_valid_number(&universal_premium_rate()));
    assert!(u.is_valid_number(&num(64, 21387835)));
}

/// PhoneNumberUtilTest#testIsValidForRegion
#[test]
fn is_valid_for_region() {
    let u = util();
    assert!(u.is_valid_number(&bs_number()));
    assert!(u.is_valid_number_for_region(&bs_number(), "BS"));
    assert!(!u.is_valid_number_for_region(&bs_number(), "US"));
    assert!(!u.is_valid_number(&num(1, 2421232345)));

    let mut re = num(262, 262123456);
    assert!(u.is_valid_number(&re));
    assert!(u.is_valid_number_for_region(&re, "RE"));
    assert!(!u.is_valid_number_for_region(&re, "YT"));
    re.set_national_number(269601234);
    assert!(u.is_valid_number_for_region(&re, "YT"));
    assert!(!u.is_valid_number_for_region(&re, "RE"));
    re.set_national_number(269123456);
    assert!(!u.is_valid_number_for_region(&re, "YT"));
    assert!(!u.is_valid_number_for_region(&re, "RE"));
    assert!(!u.is_valid_number(&re));
    assert_eq!(Some("YT".to_string()), u.get_region_code_for_number(&re));
    re.set_national_number(800123456);
    assert!(u.is_valid_number_for_region(&re, "YT"));
    assert!(u.is_valid_number_for_region(&re, "RE"));
    assert!(u.is_valid_number_for_region(&international_toll_free(), "001"));
    assert!(!u.is_valid_number_for_region(&international_toll_free(), "US"));
    assert!(!u.is_valid_number_for_region(&international_toll_free(), "ZZ"));

    let mut invalid = num(3923, 2366);
    assert!(!u.is_valid_number_for_region(&invalid, "ZZ"));
    assert!(!u.is_valid_number_for_region(&invalid, "001"));
    invalid.set_country_code(0);
    assert!(!u.is_valid_number_for_region(&invalid, "001"));
    assert!(!u.is_valid_number_for_region(&invalid, "ZZ"));
}

/// PhoneNumberUtilTest#testIsNotValidNumber
#[test]
fn is_not_valid_number() {
    let u = util();
    assert!(!u.is_valid_number(&us_local_number()));
    let mut invalid = num(39, 23661830000);
    invalid.set_italian_leading_zero(true);
    assert!(!u.is_valid_number(&invalid));
    assert!(!u.is_valid_number(&num(44, 791234567)));
    assert!(!u.is_valid_number(&num(49, 1234)));
    assert!(!u.is_valid_number(&num(64, 3316005)));
    let mut invalid = num(3923, 2366);
    assert!(!u.is_valid_number(&invalid));
    invalid.set_country_code(0);
    assert!(!u.is_valid_number(&invalid));
    assert!(!u.is_valid_number(&international_toll_free_too_long()));
}

/// PhoneNumberUtilTest#testGetRegionCodeForCountryCode
#[test]
fn get_region_code_for_country_code() {
    let u = util();
    assert_eq!("US", u.get_region_code_for_country_code(1));
    assert_eq!("GB", u.get_region_code_for_country_code(44));
    assert_eq!("DE", u.get_region_code_for_country_code(49));
    assert_eq!("001", u.get_region_code_for_country_code(800));
    assert_eq!("001", u.get_region_code_for_country_code(979));
}

/// PhoneNumberUtilTest#testGetRegionCodeForNumber
#[test]
fn get_region_code_for_number() {
    let u = util();
    let region = |n: PhoneNumber| u.get_region_code_for_number(&n);
    assert_eq!(Some("BS".into()), region(bs_number()));
    assert_eq!(Some("US".into()), region(us_number()));
    assert_eq!(Some("GB".into()), region(gb_mobile()));
    assert_eq!(Some("001".into()), region(international_toll_free()));
    assert_eq!(Some("001".into()), region(universal_premium_rate()));
}

/// PhoneNumberUtilTest#testGetRegionCodesForCountryCode
#[test]
fn get_region_codes_for_country_code() {
    let u = util();
    let nanpa = u.get_region_codes_for_country_code(1);
    assert!(nanpa.contains(&"US".to_string()));
    assert!(nanpa.contains(&"BS".to_string()));
    assert!(u.get_region_codes_for_country_code(44).contains(&"GB".to_string()));
    assert!(u.get_region_codes_for_country_code(49).contains(&"DE".to_string()));
    assert!(u.get_region_codes_for_country_code(800).contains(&"001".to_string()));
    assert!(u.get_region_codes_for_country_code(-1).is_empty());
}

/// PhoneNumberUtilTest#testGetCountryCodeForRegion (the `null` region case has no counterpart: regions are `&str`)
#[test]
fn get_country_code_for_region() {
    let u = util();
    assert_eq!(1, u.get_country_code_for_region("US"));
    assert_eq!(64, u.get_country_code_for_region("NZ"));
    assert_eq!(0, u.get_country_code_for_region("ZZ"));
    assert_eq!(0, u.get_country_code_for_region("001"));
    assert_eq!(0, u.get_country_code_for_region("CS"));
}

/// PhoneNumberUtilTest#testGetNationalDiallingPrefixForRegion (the `null` region case has no counterpart)
#[test]
fn get_national_dialling_prefix_for_region() {
    let u = util();
    assert_eq!(Some("1".into()), u.get_ndd_prefix_for_region("US", false));
    assert_eq!(Some("1".into()), u.get_ndd_prefix_for_region("BS", false));
    assert_eq!(Some("0".into()), u.get_ndd_prefix_for_region("NZ", false));
    assert_eq!(Some("0~0".into()), u.get_ndd_prefix_for_region("AO", false));
    assert_eq!(Some("00".into()), u.get_ndd_prefix_for_region("AO", true));
    assert_eq!(None, u.get_ndd_prefix_for_region("ZZ", false));
    assert_eq!(None, u.get_ndd_prefix_for_region("001", false));
    assert_eq!(None, u.get_ndd_prefix_for_region("CS", false));
}

/// PhoneNumberUtilTest#testIsNANPACountry (the `null` region case has no counterpart)
#[test]
fn is_nanpa_country() {
    let u = util();
    assert!(u.is_nanpa_country("US"));
    assert!(u.is_nanpa_country("BS"));
    assert!(!u.is_nanpa_country("DE"));
    assert!(!u.is_nanpa_country("ZZ"));
    assert!(!u.is_nanpa_country("001"));
}

/// PhoneNumberUtilTest#testIsPossibleNumber
#[test]
fn is_possible_number() {
    let u = util();
    assert!(u.is_possible_number(&us_number()));
    assert!(u.is_possible_number(&us_local_number()));
    assert!(u.is_possible_number(&gb_number()));
    assert!(u.is_possible_number(&international_toll_free()));
    assert!(u.is_possible_number_str("+1 650 253 0000", "US"));
    assert!(u.is_possible_number_str("+1 650 GOO OGLE", "US"));
    assert!(u.is_possible_number_str("(650) 253-0000", "US"));
    assert!(u.is_possible_number_str("253-0000", "US"));
    assert!(u.is_possible_number_str("+1 650 253 0000", "GB"));
    assert!(u.is_possible_number_str("+44 20 7031 3000", "GB"));
    assert!(u.is_possible_number_str("(020) 7031 300", "GB"));
    assert!(u.is_possible_number_str("7031 3000", "GB"));
    assert!(u.is_possible_number_str("3331 6005", "NZ"));
    assert!(u.is_possible_number_str("+800 1234 5678", "001"));
}

/// PhoneNumberUtilTest#testIsPossibleNumberForType_DifferentTypeLengths
#[test]
fn is_possible_number_for_type_different_type_lengths() {
    use PhoneNumberType::{FixedLine, Mobile, TollFree, Unknown};
    let u = util();
    let mut number = num(54, 12345);
    assert!(!u.is_possible_number_for_type(&number, FixedLine));
    assert!(!u.is_possible_number_for_type(&number, Unknown));
    number.set_national_number(123456);
    assert!(u.is_possible_number_for_type(&number, Unknown));
    assert!(u.is_possible_number_for_type(&number, FixedLine));
    assert!(!u.is_possible_number_for_type(&number, Mobile));
    assert!(!u.is_possible_number_for_type(&number, TollFree));
    number.set_national_number(123456789);
    assert!(u.is_possible_number_for_type(&number, Unknown));
    assert!(u.is_possible_number_for_type(&number, FixedLine));
    assert!(!u.is_possible_number_for_type(&number, Mobile));
    assert!(!u.is_possible_number_for_type(&number, TollFree));
    number.set_national_number(1234567890);
    assert!(u.is_possible_number_for_type(&number, Unknown));
    assert!(u.is_possible_number_for_type(&number, FixedLine));
    assert!(u.is_possible_number_for_type(&number, Mobile));
    assert!(u.is_possible_number_for_type(&number, TollFree));
    number.set_national_number(12345678901);
    assert!(u.is_possible_number_for_type(&number, Unknown));
    assert!(!u.is_possible_number_for_type(&number, FixedLine));
    assert!(u.is_possible_number_for_type(&number, Mobile));
    assert!(!u.is_possible_number_for_type(&number, TollFree));
}

/// PhoneNumberUtilTest#testIsPossibleNumberForType_LocalOnly
#[test]
fn is_possible_number_for_type_local_only() {
    let u = util();
    let number = num(49, 12);
    assert!(u.is_possible_number_for_type(&number, PhoneNumberType::Unknown));
    assert!(u.is_possible_number_for_type(&number, PhoneNumberType::FixedLine));
    assert!(!u.is_possible_number_for_type(&number, PhoneNumberType::Mobile));
}

/// PhoneNumberUtilTest#testIsPossibleNumberForType_DataMissingForSizeReasons
#[test]
fn is_possible_number_for_type_data_missing_for_size_reasons() {
    let u = util();
    let mut number = num(55, 12345678);
    assert!(u.is_possible_number_for_type(&number, PhoneNumberType::Unknown));
    assert!(u.is_possible_number_for_type(&number, PhoneNumberType::FixedLine));
    number.set_national_number(1234567890);
    assert!(u.is_possible_number_for_type(&number, PhoneNumberType::Unknown));
    assert!(u.is_possible_number_for_type(&number, PhoneNumberType::FixedLine));
}

/// PhoneNumberUtilTest#testIsPossibleNumberForType_NumberTypeNotSupportedForRegion
#[test]
fn is_possible_number_for_type_number_type_not_supported_for_region() {
    use PhoneNumberType::{FixedLine, FixedLineOrMobile, Mobile, PremiumRate};
    let u = util();
    let mut number = num(55, 12345678);
    assert!(!u.is_possible_number_for_type(&number, Mobile));
    assert!(u.is_possible_number_for_type(&number, FixedLine));
    assert!(u.is_possible_number_for_type(&number, FixedLineOrMobile));
    number.set_country_code(979).set_national_number(123456789);
    assert!(!u.is_possible_number_for_type(&number, Mobile));
    assert!(!u.is_possible_number_for_type(&number, FixedLine));
    assert!(!u.is_possible_number_for_type(&number, FixedLineOrMobile));
    assert!(u.is_possible_number_for_type(&number, PremiumRate));
}

/// PhoneNumberUtilTest#testIsPossibleNumberWithReason
#[test]
fn is_possible_number_with_reason() {
    let u = util();
    assert_eq!(ValidationResult::IsPossible, u.is_possible_number_with_reason(&us_number()));
    assert_eq!(ValidationResult::IsPossibleLocalOnly, u.is_possible_number_with_reason(&us_local_number()));
    assert_eq!(ValidationResult::TooLong, u.is_possible_number_with_reason(&us_long_number()));
    assert_eq!(ValidationResult::InvalidCountryCode, u.is_possible_number_with_reason(&num(0, 2530000)));
    assert_eq!(ValidationResult::TooShort, u.is_possible_number_with_reason(&num(1, 253000)));
    assert_eq!(ValidationResult::IsPossible, u.is_possible_number_with_reason(&num(65, 1234567890)));
    assert_eq!(ValidationResult::TooLong, u.is_possible_number_with_reason(&international_toll_free_too_long()));
}

/// PhoneNumberUtilTest#testIsPossibleNumberForTypeWithReason_DifferentTypeLengths
#[test]
fn is_possible_number_for_type_with_reason_different_type_lengths() {
    use PhoneNumberType::{FixedLine, Mobile, TollFree, Unknown};
    use ValidationResult::{IsPossible, TooLong, TooShort};
    let u = util();
    let r = |n: &PhoneNumber, t| u.is_possible_number_for_type_with_reason(n, t);
    let mut number = num(54, 12345);
    assert_eq!(TooShort, r(&number, Unknown));
    assert_eq!(TooShort, r(&number, FixedLine));
    number.set_national_number(123456);
    assert_eq!(IsPossible, r(&number, Unknown));
    assert_eq!(IsPossible, r(&number, FixedLine));
    assert_eq!(TooShort, r(&number, Mobile));
    assert_eq!(TooShort, r(&number, TollFree));
    number.set_national_number(123456789);
    assert_eq!(IsPossible, r(&number, Unknown));
    assert_eq!(IsPossible, r(&number, FixedLine));
    assert_eq!(TooShort, r(&number, Mobile));
    assert_eq!(TooShort, r(&number, TollFree));
    number.set_national_number(1234567890);
    assert_eq!(IsPossible, r(&number, Unknown));
    assert_eq!(IsPossible, r(&number, FixedLine));
    assert_eq!(IsPossible, r(&number, Mobile));
    assert_eq!(IsPossible, r(&number, TollFree));
    number.set_national_number(12345678901);
    assert_eq!(IsPossible, r(&number, Unknown));
    assert_eq!(TooLong, r(&number, FixedLine));
    assert_eq!(IsPossible, r(&number, Mobile));
    assert_eq!(TooLong, r(&number, TollFree));
}

/// PhoneNumberUtilTest#testIsPossibleNumberForTypeWithReason_LocalOnly
#[test]
fn is_possible_number_for_type_with_reason_local_only() {
    let u = util();
    let number = num(49, 12);
    assert_eq!(ValidationResult::IsPossibleLocalOnly, u.is_possible_number_for_type_with_reason(&number, PhoneNumberType::Unknown));
    assert_eq!(ValidationResult::IsPossibleLocalOnly, u.is_possible_number_for_type_with_reason(&number, PhoneNumberType::FixedLine));
    assert_eq!(ValidationResult::TooShort, u.is_possible_number_for_type_with_reason(&number, PhoneNumberType::Mobile));
}

/// PhoneNumberUtilTest#testIsPossibleNumberForTypeWithReason_DataMissingForSizeReasons
#[test]
fn is_possible_number_for_type_with_reason_data_missing_for_size_reasons() {
    let u = util();
    let mut number = num(55, 12345678);
    assert_eq!(ValidationResult::IsPossibleLocalOnly, u.is_possible_number_for_type_with_reason(&number, PhoneNumberType::Unknown));
    assert_eq!(ValidationResult::IsPossibleLocalOnly, u.is_possible_number_for_type_with_reason(&number, PhoneNumberType::FixedLine));
    number.set_national_number(1234567890);
    assert_eq!(ValidationResult::IsPossible, u.is_possible_number_for_type_with_reason(&number, PhoneNumberType::Unknown));
    assert_eq!(ValidationResult::IsPossible, u.is_possible_number_for_type_with_reason(&number, PhoneNumberType::FixedLine));
}

/// PhoneNumberUtilTest#testIsPossibleNumberForTypeWithReason_NumberTypeNotSupportedForRegion
#[test]
fn is_possible_number_for_type_with_reason_number_type_not_supported_for_region() {
    use PhoneNumberType::{FixedLine, FixedLineOrMobile, Mobile, PremiumRate};
    use ValidationResult::{InvalidLength, IsPossible, IsPossibleLocalOnly, TooShort};
    let u = util();
    let r = |n: &PhoneNumber, t| u.is_possible_number_for_type_with_reason(n, t);
    let mut number = num(55, 12345678);
    assert_eq!(InvalidLength, r(&number, Mobile));
    assert_eq!(IsPossibleLocalOnly, r(&number, FixedLineOrMobile));
    number.set_country_code(55).set_national_number(1234567);
    assert_eq!(InvalidLength, r(&number, Mobile));
    assert_eq!(TooShort, r(&number, FixedLineOrMobile));
    assert_eq!(TooShort, r(&number, FixedLine));
    number.set_country_code(882).set_national_number(1234567);
    assert_eq!(TooShort, r(&number, Mobile));
    assert_eq!(TooShort, r(&number, FixedLineOrMobile));
    assert_eq!(InvalidLength, r(&number, FixedLine));
    number.set_country_code(979).set_national_number(123456789);
    assert_eq!(InvalidLength, r(&number, Mobile));
    assert_eq!(InvalidLength, r(&number, FixedLine));
    assert_eq!(InvalidLength, r(&number, FixedLineOrMobile));
    assert_eq!(IsPossible, r(&number, PremiumRate));
}

/// PhoneNumberUtilTest#testIsPossibleNumberForTypeWithReason_FixedLineOrMobile
#[test]
fn is_possible_number_for_type_with_reason_fixed_line_or_mobile() {
    use PhoneNumberType::{FixedLine, FixedLineOrMobile, Mobile, TollFree};
    use ValidationResult::{InvalidLength, IsPossible, TooLong, TooShort};
    let u = util();
    let r = |n: &PhoneNumber, t| u.is_possible_number_for_type_with_reason(n, t);
    let mut number = num(290, 1234);
    assert_eq!(TooShort, r(&number, FixedLine));
    assert_eq!(IsPossible, r(&number, Mobile));
    assert_eq!(IsPossible, r(&number, FixedLineOrMobile));
    number.set_national_number(12345);
    assert_eq!(TooShort, r(&number, FixedLine));
    assert_eq!(TooLong, r(&number, Mobile));
    assert_eq!(InvalidLength, r(&number, FixedLineOrMobile));
    number.set_national_number(123456);
    assert_eq!(IsPossible, r(&number, FixedLine));
    assert_eq!(TooLong, r(&number, Mobile));
    assert_eq!(IsPossible, r(&number, FixedLineOrMobile));
    number.set_national_number(1234567);
    assert_eq!(TooLong, r(&number, FixedLine));
    assert_eq!(TooLong, r(&number, Mobile));
    assert_eq!(TooLong, r(&number, FixedLineOrMobile));
    number.set_national_number(12345678);
    assert_eq!(IsPossible, r(&number, TollFree));
    assert_eq!(TooLong, r(&number, FixedLineOrMobile));
}

/// PhoneNumberUtilTest#testIsNotPossibleNumber
#[test]
fn is_not_possible_number() {
    let u = util();
    assert!(!u.is_possible_number(&us_long_number()));
    assert!(!u.is_possible_number(&international_toll_free_too_long()));
    assert!(!u.is_possible_number(&num(1, 253000)));
    assert!(!u.is_possible_number(&num(44, 300)));
    assert!(!u.is_possible_number_str("+1 650 253 00000", "US"));
    assert!(!u.is_possible_number_str("(650) 253-00000", "US"));
    assert!(!u.is_possible_number_str("I want a Pizza", "US"));
    assert!(!u.is_possible_number_str("253-000", "US"));
    assert!(!u.is_possible_number_str("1 3000", "GB"));
    assert!(!u.is_possible_number_str("+44 300", "GB"));
    assert!(!u.is_possible_number_str("+800 1234 5678 9", "001"));
}

/// PhoneNumberUtilTest#testTruncateTooLongNumber
#[test]
fn truncate_too_long_number() {
    let u = util();
    let mut too_long = num(44, 80123456780123);
    let mut valid = num(44, 8012345678);
    assert!(u.truncate_too_long_number(&mut too_long));
    assert_eq!(valid, too_long);

    too_long.clear();
    too_long.set_country_code(39).set_national_number(2234567890123).set_italian_leading_zero(true);
    valid.clear();
    valid.set_country_code(39).set_national_number(2234567890).set_italian_leading_zero(true);
    assert!(u.truncate_too_long_number(&mut too_long));
    assert_eq!(valid, too_long);

    too_long.clear();
    too_long.merge_from(&us_long_number());
    assert!(u.truncate_too_long_number(&mut too_long));
    assert_eq!(us_number(), too_long);

    too_long.clear();
    too_long.merge_from(&international_toll_free_too_long());
    assert!(u.truncate_too_long_number(&mut too_long));
    assert_eq!(international_toll_free(), too_long);

    let valid_copy = valid.clone();
    assert!(u.truncate_too_long_number(&mut valid));
    assert_eq!(valid_copy, valid);

    let mut invalid_prefix = num(1, 2401234567);
    let invalid_copy = invalid_prefix.clone();
    assert!(!u.truncate_too_long_number(&mut invalid_prefix));
    assert_eq!(invalid_copy, invalid_prefix);

    let mut too_short = num(1, 1234);
    let too_short_copy = too_short.clone();
    assert!(!u.truncate_too_long_number(&mut too_short));
    assert_eq!(too_short_copy, too_short);
}

/// PhoneNumberUtilTest#testIsViablePhoneNumber
#[test]
fn is_viable_phone_number() {
    use libphonenumber::is_viable_phone_number as viable;
    assert!(!viable("1"));
    assert!(!viable("1+1+1"));
    assert!(!viable("80+0"));
    assert!(viable("00"));
    assert!(viable("111"));
    assert!(viable("0800-4-pizza"));
    assert!(viable("0800-4-PIZZA"));
    assert!(!viable("08-PIZZA"));
    assert!(!viable("8-PIZZA"));
    assert!(!viable("12. March"));
}

/// PhoneNumberUtilTest#testIsViablePhoneNumberNonAscii
#[test]
fn is_viable_phone_number_non_ascii() {
    use libphonenumber::is_viable_phone_number as viable;
    assert!(viable("1\u{3000}34"));
    assert!(!viable("1\u{3000}3+4"));
    assert!(viable("\u{FF08}1\u{FF09}\u{3000}3456789"));
    assert!(viable("+1\u{FF09}\u{3000}3456789"));
}

/// PhoneNumberUtilTest#testExtractPossibleNumber
#[test]
fn extract_possible_number() {
    use libphonenumber::extract_possible_number as extract;
    assert_eq!("0800-345-600", extract("Tel:0800-345-600"));
    assert_eq!("0800 FOR PIZZA", extract("Tel:0800 FOR PIZZA"));
    assert_eq!("+800-345-600", extract("Tel:+800-345-600"));
    assert_eq!("\u{FF10}\u{FF12}\u{FF13}", extract("\u{FF10}\u{FF12}\u{FF13}"));
    assert_eq!("\u{FF11}\u{FF12}\u{FF13}", extract("Num-\u{FF11}\u{FF12}\u{FF13}"));
    assert_eq!("", extract("Num-...."));
    assert_eq!("650) 253-0000", extract("(650) 253-0000"));
    assert_eq!("650) 253-0000", extract("(650) 253-0000..- .."));
    assert_eq!("650) 253-0000", extract("(650) 253-0000."));
    assert_eq!("650) 253-0000", extract("(650) 253-0000\u{200F}"));
}

/// PhoneNumberUtilTest#testMaybeStripNationalPrefix
#[test]
fn maybe_strip_national_prefix() {
    let u = util();
    let mut metadata = libphonenumber::PhoneMetadata {
        id: "ignored".into(),
        national_prefix_for_parsing: "34".into(),
        has_national_prefix_for_parsing: true,
        general_desc: Some(libphonenumber::PhoneNumberDesc {
            national_number_pattern: "\\d{4,8}".into(),
            has_national_number_pattern: true,
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut number = String::from("34356778");
    assert!(u.maybe_strip_national_prefix_and_carrier_code(&mut number, &metadata, None));
    assert_eq!("356778", number);
    assert!(!u.maybe_strip_national_prefix_and_carrier_code(&mut number, &metadata, None));
    assert_eq!("356778", number);
    metadata.national_prefix_for_parsing = "".into();
    assert!(!u.maybe_strip_national_prefix_and_carrier_code(&mut number, &metadata, None));
    assert_eq!("356778", number);
    metadata.national_prefix_for_parsing = "3".into();
    let mut number = String::from("3123");
    assert!(!u.maybe_strip_national_prefix_and_carrier_code(&mut number, &metadata, None));
    assert_eq!("3123", number);
    metadata.national_prefix_for_parsing = "0(81)?".into();
    let mut number = String::from("08122123456");
    let mut carrier_code = String::new();
    assert!(u.maybe_strip_national_prefix_and_carrier_code(&mut number, &metadata, Some(&mut carrier_code)));
    assert_eq!("81", carrier_code);
    assert_eq!("22123456", number);
    metadata.national_prefix_transform_rule = "5$15".into();
    metadata.has_national_prefix_transform_rule = true;
    metadata.national_prefix_for_parsing = "0(\\d{2})".into();
    let mut number = String::from("031123");
    assert!(u.maybe_strip_national_prefix_and_carrier_code(&mut number, &metadata, None));
    assert_eq!("5315123", number);
}

/// PhoneNumberUtilTest#testMaybeStripInternationalPrefix
#[test]
fn maybe_strip_international_prefix() {
    let u = util();
    let idd = "00[39]";
    let mut number = String::from("0034567700-3898003");
    assert_eq!(CountryCodeSource::FromNumberWithIdd, u.maybe_strip_international_prefix_and_normalize(&mut number, idd));
    assert_eq!("45677003898003", number);
    assert_eq!(CountryCodeSource::FromDefaultCountry, u.maybe_strip_international_prefix_and_normalize(&mut number, idd));

    let mut number = String::from("00945677003898003");
    assert_eq!(CountryCodeSource::FromNumberWithIdd, u.maybe_strip_international_prefix_and_normalize(&mut number, idd));
    assert_eq!("45677003898003", number);
    let mut number = String::from("00 9 45677003898003");
    assert_eq!(CountryCodeSource::FromNumberWithIdd, u.maybe_strip_international_prefix_and_normalize(&mut number, idd));
    assert_eq!("45677003898003", number);
    assert_eq!(CountryCodeSource::FromDefaultCountry, u.maybe_strip_international_prefix_and_normalize(&mut number, idd));

    let mut number = String::from("+45677003898003");
    assert_eq!(CountryCodeSource::FromNumberWithPlusSign, u.maybe_strip_international_prefix_and_normalize(&mut number, idd));
    assert_eq!("45677003898003", number);

    let mut number = String::from("0090112-3123");
    assert_eq!(CountryCodeSource::FromDefaultCountry, u.maybe_strip_international_prefix_and_normalize(&mut number, idd));
    assert_eq!("00901123123", number);
    let mut number = String::from("009 0-112-3123");
    assert_eq!(CountryCodeSource::FromDefaultCountry, u.maybe_strip_international_prefix_and_normalize(&mut number, idd));
}

/// PhoneNumberUtilTest#testMaybeExtractCountryCode
#[test]
fn maybe_extract_country_code() {
    let u = util();
    let metadata = u.get_metadata_for_region("US");
    let extract = |input: &str, keep: bool, number: &mut PhoneNumber, fill: &mut String| {
        u.maybe_extract_country_code(input, metadata, fill, keep, number)
    };

    let mut number = PhoneNumber::new();
    let mut fill = String::new();
    assert_eq!(Ok(1), extract("011112-3456789", true, &mut number, &mut fill));
    assert_eq!(CountryCodeSource::FromNumberWithIdd, number.country_code_source());
    assert_eq!("123456789", fill);

    number.clear();
    let mut fill = String::new();
    assert_eq!(Ok(64), extract("+6423456789", true, &mut number, &mut fill));
    assert_eq!(CountryCodeSource::FromNumberWithPlusSign, number.country_code_source());

    number.clear();
    let mut fill = String::new();
    assert_eq!(Ok(800), extract("+80012345678", true, &mut number, &mut fill));
    assert_eq!(CountryCodeSource::FromNumberWithPlusSign, number.country_code_source());

    number.clear();
    let mut fill = String::new();
    assert_eq!(Ok(0), extract("2345-6789", true, &mut number, &mut fill));
    assert_eq!(CountryCodeSource::FromDefaultCountry, number.country_code_source());

    number.clear();
    let mut fill = String::new();
    let err = extract("0119991123456789", true, &mut number, &mut fill).unwrap_err();
    assert_eq!(ErrorType::InvalidCountryCode, err.error_type());

    number.clear();
    let mut fill = String::new();
    assert_eq!(Ok(1), extract("(1 610) 619 4466", true, &mut number, &mut fill));
    assert_eq!(CountryCodeSource::FromNumberWithoutPlusSign, number.country_code_source());

    number.clear();
    let mut fill = String::new();
    assert_eq!(Ok(1), extract("(1 610) 619 4466", false, &mut number, &mut fill));
    assert!(!number.has_country_code_source());

    number.clear();
    let mut fill = String::new();
    assert_eq!(Ok(0), extract("(1 610) 619 446", false, &mut number, &mut fill));
    assert!(!number.has_country_code_source());

    number.clear();
    let mut fill = String::new();
    assert_eq!(Ok(0), extract("(1 610) 619", true, &mut number, &mut fill));
    assert_eq!(CountryCodeSource::FromDefaultCountry, number.country_code_source());
}

fn with_extension(mut n: PhoneNumber, extension: &str) -> PhoneNumber {
    n.set_extension(extension);
    n
}

/// PhoneNumberUtilTest#testParseNationalNumber
#[test]
fn parse_national_number() {
    let nz = nz_number();
    assert_eq!(nz, parse("033316005", "NZ"));
    assert!(!nz.has_country_code_source());
    assert_eq!(CountryCodeSource::Unspecified, nz.country_code_source());
    for (input, region) in [
        ("33316005", "NZ"),
        ("03-331 6005", "NZ"),
        ("03 331 6005", "NZ"),
        ("tel:03-331-6005;phone-context=+64", "NZ"),
        ("tel:331-6005;phone-context=+64-3", "NZ"),
        ("tel:331-6005;phone-context=+64-3", "US"),
        ("My number is tel:03-331-6005;phone-context=+64", "NZ"),
        ("tel:03-331-6005;phone-context=+64;a=%A1", "NZ"),
        ("tel:03-331-6005;isub=12345;phone-context=+64", "NZ"),
        ("tel:+64-3-331-6005;isub=12345", "NZ"),
        ("03-331-6005;phone-context=+64", "NZ"),
        ("0064 3 331 6005", "NZ"),
        ("01164 3 331 6005", "US"),
        ("+64 3 331 6005", "US"),
        ("+01164 3 331 6005", "US"),
        ("+0064 3 331 6005", "NZ"),
        ("+ 00 64 3 331 6005", "NZ"),
    ] {
        assert_eq!(nz, parse(input, region), "{input}");
    }
    assert_eq!(us_local_number(), parse("tel:253-0000;phone-context=www.google.com", "US"));
    assert_eq!(us_local_number(), parse("tel:253-0000;isub=12345;phone-context=www.google.com", "US"));
    assert_eq!(us_local_number(), parse("tel:2530000;isub=12345;phone-context=1234.com", "US"));
    assert_eq!(num(64, 64123456), parse("64(0)64123456", "NZ"));
    assert_eq!(de_number(), parse("301/23456", "DE"));
    assert_eq!(num(1, 1234567890), parse("123-456-7890", "US"));
    assert_eq!(jp_star_number(), parse("+81 *2345", "JP"));
    let mut short_number = num(64, 12);
    assert_eq!(short_number, parse("12", "NZ"));
    short_number.set_country_code(44).set_national_number(123456).set_italian_leading_zero(true);
    assert_eq!(short_number, parse("0123456", "GB"));
}

/// PhoneNumberUtilTest#testParseNumberWithAlphaCharacters
#[test]
fn parse_number_with_alpha_characters() {
    assert_eq!(num(64, 800332005), parse("0800 DDA 005", "NZ"));
    let premium = num(64, 9003326005);
    assert_eq!(premium, parse("0900 DDA 6005", "NZ"));
    assert_eq!(premium, parse("0900 332 6005a", "NZ"));
    assert_eq!(premium, parse("0900 332 600a5", "NZ"));
    assert_eq!(premium, parse("0900 332 600A5", "NZ"));
    assert_eq!(premium, parse("0900 a332 600A5", "NZ"));
}

/// PhoneNumberUtilTest#testParseMaliciousInput
#[test]
fn parse_malicious_input() {
    let malicious = format!("{}12222-33-244 extensioB 343+", "+".repeat(6000));
    assert_eq!(ErrorType::TooLong, parse_err(&malicious, Some("US")).error_type());
    let almost_ext = format!("{} extensiOB 345", "200".repeat(350));
    assert_eq!(ErrorType::TooLong, parse_err(&almost_ext, Some("US")).error_type());
}

/// PhoneNumberUtilTest#testParseWithInternationalPrefixes
#[test]
fn parse_with_international_prefixes() {
    assert_eq!(us_number(), parse("+1 (650) 253-0000", "NZ"));
    assert_eq!(international_toll_free(), parse("011 800 1234 5678", "US"));
    assert_eq!(us_number(), parse("1-650-253-0000", "US"));
    assert_eq!(us_number(), parse("0011-650-253-0000", "SG"));
    assert_eq!(us_number(), parse("0081-650-253-0000", "SG"));
    assert_eq!(us_number(), parse("0191-650-253-0000", "SG"));
    assert_eq!(us_number(), parse("0~01-650-253-0000", "PL"));
    assert_eq!(us_number(), parse("++1 (650) 253-0000", "PL"));
}

/// PhoneNumberUtilTest#testParseNonAscii
#[test]
fn parse_non_ascii() {
    assert_eq!(us_number(), parse("\u{FF0B}1 (650) 253-0000", "SG"));
    assert_eq!(us_number(), parse("1 (650) 253\u{00AD}-0000", "US"));
    assert_eq!(
        us_number(),
        parse(
            "\u{FF0B}\u{FF11}\u{3000}\u{FF08}\u{FF16}\u{FF15}\u{FF10}\u{FF09}\u{3000}\u{FF12}\u{FF15}\u{FF13}\u{FF0D}\u{FF10}\u{FF10}\u{FF10}\u{FF10}",
            "SG"
        )
    );
    assert_eq!(
        us_number(),
        parse(
            "\u{FF0B}\u{FF11}\u{3000}\u{FF08}\u{FF16}\u{FF15}\u{FF10}\u{FF09}\u{3000}\u{FF12}\u{FF15}\u{FF13}\u{30FC}\u{FF10}\u{FF10}\u{FF10}\u{FF10}",
            "SG"
        )
    );
    assert_eq!(
        us_number(),
        parse("\u{1811} \u{1816}\u{1815}\u{1810} \u{1812}\u{1815}\u{1813} \u{1810}\u{1810}\u{1810}\u{1810}", "US")
    );
}

/// PhoneNumberUtilTest#testParseWithLeadingZero
#[test]
fn parse_with_leading_zero() {
    assert_eq!(it_number(), parse("+39 02-36618 300", "NZ"));
    assert_eq!(it_number(), parse("02-36618 300", "IT"));
    assert_eq!(it_mobile(), parse("345 678 901", "IT"));
}

/// PhoneNumberUtilTest#testParseNationalNumberArgentina
#[test]
fn parse_national_number_argentina() {
    assert_eq!(num(54, 93435551212), parse("+54 9 343 555 1212", "AR"));
    assert_eq!(num(54, 93435551212), parse("0343 15 555 1212", "AR"));
    assert_eq!(num(54, 93715654320), parse("+54 9 3715 65 4320", "AR"));
    assert_eq!(num(54, 93715654320), parse("03715 15 65 4320", "AR"));
    assert_eq!(ar_mobile(), parse("911 876 54321", "AR"));
    assert_eq!(ar_number(), parse("+54 11 8765 4321", "AR"));
    assert_eq!(ar_number(), parse("011 8765 4321", "AR"));
    assert_eq!(num(54, 3715654321), parse("+54 3715 65 4321", "AR"));
    assert_eq!(num(54, 3715654321), parse("03715 65 4321", "AR"));
    assert_eq!(num(54, 2312340000), parse("+54 23 1234 0000", "AR"));
    assert_eq!(num(54, 2312340000), parse("023 1234 0000", "AR"));
}

/// PhoneNumberUtilTest#testParseWithXInNumber
#[test]
fn parse_with_x_in_number() {
    assert_eq!(ar_number(), parse("01187654321", "AR"));
    assert_eq!(ar_number(), parse("(0) 1187654321", "AR"));
    assert_eq!(ar_number(), parse("0 1187654321", "AR"));
    assert_eq!(ar_number(), parse("(0xx) 1187654321", "AR"));
    assert_eq!(num(54, 81429712), parse("011xx5481429712", "US"));
}

/// PhoneNumberUtilTest#testParseNumbersMexico
#[test]
fn parse_numbers_mexico() {
    assert_eq!(num(52, 4499780001), parse("+52 (449)978-0001", "MX"));
    assert_eq!(num(52, 4499780001), parse("01 (449)978-0001", "MX"));
    assert_eq!(num(52, 4499780001), parse("(449)978-0001", "MX"));
    assert_eq!(num(52, 13312345678), parse("+52 1 33 1234-5678", "MX"));
    assert_eq!(num(52, 13312345678), parse("044 (33) 1234-5678", "MX"));
    assert_eq!(num(52, 13312345678), parse("045 33 1234-5678", "MX"));
}

/// PhoneNumberUtilTest#testFailedParseOnInvalidNumbers (the two `null` number cases have no counterpart: numbers
/// are `&str`)
#[test]
fn failed_parse_on_invalid_numbers() {
    use ErrorType::{InvalidCountryCode, NotANumber, TooLong, TooShortAfterIdd, TooShortNsn};
    for (input, region, expected) in [
        ("This is not a phone number", Some("NZ"), NotANumber),
        ("1 Still not a number", Some("NZ"), NotANumber),
        ("1 MICROSOFT", Some("NZ"), NotANumber),
        ("12 MICROSOFT", Some("NZ"), NotANumber),
        ("01495 72553301873 810104", Some("GB"), TooLong),
        ("+---", Some("DE"), NotANumber),
        ("+***", Some("DE"), NotANumber),
        ("+*******91", Some("DE"), NotANumber),
        ("+49 0", Some("DE"), TooShortNsn),
        ("+210 3456 56789", Some("NZ"), InvalidCountryCode),
        ("+ 00 210 3 331 6005", Some("NZ"), InvalidCountryCode),
        ("123 456 7890", Some("ZZ"), InvalidCountryCode),
        ("123 456 7890", Some("CS"), InvalidCountryCode),
        ("123 456 7890", None, InvalidCountryCode),
        ("0044------", Some("GB"), TooShortAfterIdd),
        ("0044", Some("GB"), TooShortAfterIdd),
        ("011", Some("US"), TooShortAfterIdd),
        ("0119", Some("US"), TooShortAfterIdd),
        ("", Some("ZZ"), NotANumber),
        ("tel:555-1234;phone-context=www.google.com", Some("ZZ"), InvalidCountryCode),
        ("tel:555-1234;phone-context=1-331", Some("ZZ"), NotANumber),
        (";phone-context=", Some("ZZ"), NotANumber),
    ] {
        assert_eq!(expected, parse_err(input, region).error_type(), "{input:?} {region:?}");
    }
}

/// PhoneNumberUtilTest#testParseNumbersWithPlusWithNoRegion
#[test]
fn parse_numbers_with_plus_with_no_region() {
    let u = util();
    assert_eq!(nz_number(), parse("+64 3 331 6005", "ZZ"));
    assert_eq!(nz_number(), parse("\u{FF0B}64 3 331 6005", "ZZ"));
    assert_eq!(nz_number(), parse("Tel: +64 3 331 6005", "ZZ"));
    assert_eq!(nz_number(), u.parse("+64 3 331 6005", None).unwrap());
    assert_eq!(international_toll_free(), u.parse("+800 1234 5678", None).unwrap());
    assert_eq!(universal_premium_rate(), u.parse("+979 123 456 789", None).unwrap());
    assert_eq!(nz_number(), parse("tel:03-331-6005;phone-context=+64", "ZZ"));
    assert_eq!(nz_number(), parse("  tel:03-331-6005;phone-context=+64", "ZZ"));
    assert_eq!(nz_number(), parse("tel:03-331-6005;isub=12345;phone-context=+64", "ZZ"));
    let mut with_raw = nz_number();
    with_raw.set_raw_input("+64 3 331 6005").set_country_code_source(CountryCodeSource::FromNumberWithPlusSign);
    assert_eq!(with_raw, u.parse_and_keep_raw_input("+64 3 331 6005", Some("ZZ")).unwrap());
    assert_eq!(with_raw, u.parse_and_keep_raw_input("+64 3 331 6005", None).unwrap());
}

/// PhoneNumberUtilTest#testParseNumberTooShortIfNationalPrefixStripped
#[test]
fn parse_number_too_short_if_national_prefix_stripped() {
    assert_eq!(num(375, 8123), parse("8123", "BY"));
    assert_eq!(num(375, 81234), parse("81234", "BY"));
    assert_eq!(num(375, 812345), parse("812345", "BY"));
    assert_eq!(num(375, 123456), parse("8123456", "BY"));
}

/// PhoneNumberUtilTest#testParseExtensions
#[test]
fn parse_extensions() {
    let nz = with_extension(num(64, 33316005), "3456");
    assert_eq!(nz, parse("03 331 6005 ext 3456", "NZ"));
    assert_eq!(nz, parse("03-3316005x3456", "NZ"));
    assert_eq!(nz, parse("03-3316005 int.3456", "NZ"));
    assert_eq!(nz, parse("03 3316005 #3456", "NZ"));
    assert_eq!(alpha_numeric_number(), parse("1800 six-flags", "US"));
    assert_eq!(alpha_numeric_number(), parse("1800 SIX FLAGS", "US"));
    assert_eq!(alpha_numeric_number(), parse("0~0 1800 7493 5247", "PL"));
    assert_eq!(alpha_numeric_number(), parse("(1800) 7493.5247", "US"));
    assert_eq!(with_extension(alpha_numeric_number(), "1234"), parse("0~0 1800 7493 5247 ~1234", "PL"));

    let uk = with_extension(num(44, 2034567890), "456");
    assert_eq!(uk, parse("+44 2034567890x456", "NZ"));
    for input in [
        "+44 2034567890x456",
        "+44 2034567890 x456",
        "+44 2034567890 X456",
        "+44 2034567890 X 456",
        "+44 2034567890 X  456",
        "+44 2034567890 x 456  ",
        "+44 2034567890  X 456",
        "+44-2034567890;ext=456",
        "+442034567890\u{FF45}\u{FF58}\u{FF54}\u{FF4E}456",
        "+442034567890\u{FF58}\u{FF54}\u{FF4E}456",
        "+442034567890\u{FF58}\u{FF54}456",
    ] {
        assert_eq!(uk, parse(input, "GB"), "{input}");
    }
    assert_eq!(uk, parse("tel:2034567890;ext=456;phone-context=+44", "ZZ"));

    let us = with_extension(num(1, 8009013355), "7246433");
    for input in [
        "(800) 901-3355 x 7246433",
        "(800) 901-3355 , ext 7246433",
        "(800) 901-3355 ; 7246433",
        "(800) 901-3355;7246433",
        "(800) 901-3355 ,extension 7246433",
        "(800) 901-3355 ,extensi\u{00F3}n 7246433",
        "(800) 901-3355 ,extensio\u{0301}n 7246433",
        "(800) 901-3355 , 7246433",
        "(800) 901-3355 ext: 7246433",
    ] {
        assert_eq!(us, parse(input, "US"), "{input}");
    }

    let ru = with_extension(num(7, 4232022511), "100");
    for input in [
        "8 (423) 202-25-11, \u{0434}\u{043E}\u{0431}. 100",
        "8 (423) 202-25-11 \u{0434}\u{043E}\u{0431}. 100",
        "8 (423) 202-25-11, \u{0434}\u{043E}\u{0431} 100",
        "8 (423) 202-25-11 \u{0434}\u{043E}\u{0431} 100",
        "8 (423) 202-25-11\u{0434}\u{043E}\u{0431}100",
        "8 (423) 202-25-11, \u{0414}\u{041E}\u{0411}. 100",
    ] {
        assert_eq!(ru, parse(input, "RU"), "{input}");
    }

    let two = with_extension(num(1, 2121231234), "508");
    assert_eq!(two, parse("(212)123-1234 x508/x1234", "US"));
    assert_eq!(two, parse("(212)123-1234 x508/ x1234", "US"));
    assert_eq!(two, parse("(212)123-1234 x508\\x1234", "US"));

    let us = with_extension(num(1, 6451231234), "910");
    assert_eq!(us, parse("+1 (645) 123 1234-910#", "US"));
    assert_eq!(us, parse("+1 (645) 123 1234 ext. 910#", "US"));
}

/// PhoneNumberUtilTest#testParseHandlesLongExtensionsWithExplicitLabels
#[test]
fn parse_handles_long_extensions_with_explicit_labels() {
    let nz = |e: &str| with_extension(num(64, 33316005), e);
    assert_eq!(nz("0"), parse("tel:+6433316005;ext=0", "NZ"));
    assert_eq!(nz("01234567890123456789"), parse("tel:+6433316005;ext=01234567890123456789", "NZ"));
    assert_eq!(ErrorType::NotANumber, parse_err("tel:+6433316005;ext=012345678901234567890", Some("NZ")).error_type());
    assert_eq!(nz("1"), parse("03 3316005ext:1", "NZ"));
    let long = nz("12345678901234567890");
    for input in [
        "03 3316005 xtn:12345678901234567890",
        "03 3316005 extension\t12345678901234567890",
        "03 3316005 xtensio:12345678901234567890",
        "03 3316005 xtensi\u{00F3}n, 12345678901234567890#",
        "03 3316005extension.12345678901234567890",
        "03 3316005 \u{0434}\u{043E}\u{0431}:12345678901234567890",
    ] {
        assert_eq!(long, parse(input, "NZ"), "{input}");
    }
    assert_eq!(ErrorType::TooLong, parse_err("03 3316005 extension 123456789012345678901", Some("NZ")).error_type());
}

/// PhoneNumberUtilTest#testParseHandlesLongExtensionsWithAutoDiallingLabels
#[test]
fn parse_handles_long_extensions_with_auto_dialling_labels() {
    let us = with_extension(num(1, 2679000000), "123456789012345");
    assert_eq!(us, parse("+12679000000,,123456789012345#", "US"));
    assert_eq!(us, parse("+12679000000;123456789012345#", "US"));
    assert_eq!(with_extension(num(44, 2034000000), "123456789"), parse("+442034000000,,123456789#", "GB"));
    assert_eq!(ErrorType::NotANumber, parse_err("+12679000000,,1234567890123456#", Some("US")).error_type());
}

/// PhoneNumberUtilTest#testParseHandlesShortExtensionsWithAmbiguousChar
#[test]
fn parse_handles_short_extensions_with_ambiguous_char() {
    let nz = with_extension(num(64, 33316005), "123456789");
    assert_eq!(nz, parse("03 3316005 x 123456789", "NZ"));
    assert_eq!(nz, parse("03 3316005 x. 123456789", "NZ"));
    assert_eq!(nz, parse("03 3316005 #123456789#", "NZ"));
    assert_eq!(nz, parse("03 3316005 ~ 123456789", "NZ"));
    assert_eq!(ErrorType::TooLong, parse_err("03 3316005 ~ 1234567890", Some("NZ")).error_type());
}

/// PhoneNumberUtilTest#testParseHandlesShortExtensionsWhenNotSureOfLabel
#[test]
fn parse_handles_short_extensions_when_not_sure_of_label() {
    assert_eq!(with_extension(num(1, 1234567890), "666666"), parse("+1123-456-7890 666666#", "US"));
    assert_eq!(with_extension(num(1, 1234567890), "6"), parse("+11234567890-6#", "US"));
    assert_eq!(ErrorType::NotANumber, parse_err("+1123-456-7890 7777777#", Some("US")).error_type());
}

/// PhoneNumberUtilTest#testParseAndKeepRaw
#[test]
fn parse_and_keep_raw() {
    let u = util();
    let mut alpha = alpha_numeric_number();
    alpha.set_raw_input("800 six-flags").set_country_code_source(CountryCodeSource::FromDefaultCountry);
    assert_eq!(alpha, u.parse_and_keep_raw_input("800 six-flags", Some("US")).unwrap());

    let mut shorter = num(1, 8007493524);
    shorter.set_raw_input("1800 six-flag").set_country_code_source(CountryCodeSource::FromNumberWithoutPlusSign);
    assert_eq!(shorter, u.parse_and_keep_raw_input("1800 six-flag", Some("US")).unwrap());
    shorter.set_raw_input("+1800 six-flag").set_country_code_source(CountryCodeSource::FromNumberWithPlusSign);
    assert_eq!(shorter, u.parse_and_keep_raw_input("+1800 six-flag", Some("NZ")).unwrap());
    shorter.set_raw_input("001800 six-flag").set_country_code_source(CountryCodeSource::FromNumberWithIdd);
    assert_eq!(shorter, u.parse_and_keep_raw_input("001800 six-flag", Some("NZ")).unwrap());

    assert_eq!(
        ErrorType::InvalidCountryCode,
        u.parse_and_keep_raw_input("123 456 7890", Some("CS")).unwrap_err().error_type()
    );

    let mut korean = num(82, 22123456);
    korean
        .set_raw_input("08122123456")
        .set_country_code_source(CountryCodeSource::FromDefaultCountry)
        .set_preferred_domestic_carrier_code("81");
    assert_eq!(korean, u.parse_and_keep_raw_input("08122123456", Some("KR")).unwrap());
}

/// PhoneNumberUtilTest#testParseItalianLeadingZeros
#[test]
fn parse_italian_leading_zeros() {
    let mut one_zero = num(61, 11);
    one_zero.set_italian_leading_zero(true);
    assert_eq!(one_zero, parse("011", "AU"));
    let mut two_zeros = num(61, 1);
    two_zeros.set_italian_leading_zero(true).set_number_of_leading_zeros(2);
    assert_eq!(two_zeros, parse("001", "AU"));
    let mut still_two_zeros = num(61, 0);
    still_two_zeros.set_italian_leading_zero(true).set_number_of_leading_zeros(2);
    assert_eq!(still_two_zeros, parse("000", "AU"));
    let mut three_zeros = num(61, 0);
    three_zeros.set_italian_leading_zero(true).set_number_of_leading_zeros(3);
    assert_eq!(three_zeros, parse("0000", "AU"));
}

/// PhoneNumberUtilTest#testParseWithPhoneContext
#[test]
fn parse_with_phone_context() {
    assert_eq!(nz_number(), parse("tel:033316005;phone-context=+64", "ZZ"));
    assert_eq!(nz_number(), parse("tel:033316005;phone-context=+64;{this isn't part of phone-context anymore!}", "ZZ"));
    assert_eq!(num(64, 3033316005), parse("tel:033316005;phone-context=+64-3", "ZZ"));
    assert_eq!(num(55, 5033316005), parse("tel:033316005;phone-context=+(555)", "ZZ"));
    assert_eq!(num(1, 23033316005), parse("tel:033316005;phone-context=+-1-2.3()", "ZZ"));
    for input in [
        "tel:033316005;phone-context=abc.nz",
        "tel:033316005;phone-context=www.PHONE-numb3r.com",
        "tel:033316005;phone-context=a",
        "tel:033316005;phone-context=3phone.J.",
        "tel:033316005;phone-context=a--z",
    ] {
        assert_eq!(nz_number(), parse(input, "NZ"), "{input}");
    }
    for input in [
        "tel:033316005;phone-context=",
        "tel:033316005;phone-context=+",
        "tel:033316005;phone-context=64",
        "tel:033316005;phone-context=++64",
        "tel:033316005;phone-context=+abc",
        "tel:033316005;phone-context=.",
        "tel:033316005;phone-context=3phone",
        "tel:033316005;phone-context=a-.nz",
        "tel:033316005;phone-context=a{b}c",
    ] {
        assert_eq!(ErrorType::NotANumber, parse_err(input, Some("ZZ")).error_type(), "{input}");
    }
}

/// PhoneNumberUtilTest#testCountryWithNoNumberDesc (the out-of-country assertion is dropped:
/// `formatOutOfCountryCallingNumber` is not ported)
#[test]
fn country_with_no_number_desc() {
    let u = util();
    let ad = num(376, 12345);
    assert_eq!("+376 12345", u.format(&ad, International));
    assert_eq!("+37612345", u.format(&ad, E164));
    assert_eq!("12345", u.format(&ad, National));
    assert_eq!(PhoneNumberType::Unknown, u.get_number_type(&ad));
    assert!(!u.is_valid_number(&ad));
}

/// PhoneNumberUtilTest#testUnknownCountryCallingCode
#[test]
fn unknown_country_calling_code() {
    assert!(!util().is_valid_number(&unknown_country_code_no_raw_input()));
    assert_eq!("+212345", util().format(&unknown_country_code_no_raw_input(), E164));
}

/// PhoneNumberUtilTest#testCanBeInternationallyDialled
#[test]
fn can_be_internationally_dialled() {
    let u = util();
    assert!(!u.can_be_internationally_dialled(&us_tollfree()));
    assert!(u.can_be_internationally_dialled(&us_number()));
    assert!(u.can_be_internationally_dialled(&us_local_number()));
    assert!(u.can_be_internationally_dialled(&nz_number()));
    assert!(u.can_be_internationally_dialled(&international_toll_free()));
}

/// PhoneNumberUtilTest#testIsAlphaNumber
#[test]
fn is_alpha_number() {
    let u = util();
    assert!(u.is_alpha_number("1800 six-flags"));
    assert!(u.is_alpha_number("1800 six-flags ext. 1234"));
    assert!(u.is_alpha_number("+800 six-flags"));
    assert!(u.is_alpha_number("180 six-flags"));
    assert!(!u.is_alpha_number("1800 123-1234"));
    assert!(!u.is_alpha_number("1 six-flags"));
    assert!(!u.is_alpha_number("18 six-flags"));
    assert!(!u.is_alpha_number("1800 123-1234 extension: 1234"));
    assert!(!u.is_alpha_number("+800 1234-1234"));
}

/// PhoneNumberUtilTest#testIsMobileNumberPortableRegion
#[test]
fn is_mobile_number_portable_region() {
    let u = util();
    assert!(u.is_mobile_number_portable_region("US"));
    assert!(u.is_mobile_number_portable_region("GB"));
    assert!(!u.is_mobile_number_portable_region("AE"));
    assert!(!u.is_mobile_number_portable_region("BS"));
}

/// PhoneNumberUtilTest#testGetMetadataForRegionForNonGeoEntity_shouldBeNull
#[test]
fn get_metadata_for_region_for_non_geo_entity_should_be_null() {
    assert!(util().get_metadata_for_region("001").is_none());
}

/// PhoneNumberUtilTest#testGetMetadataForRegionForUnknownRegion_shouldBeNull
#[test]
fn get_metadata_for_region_for_unknown_region_should_be_null() {
    assert!(util().get_metadata_for_region("ZZ").is_none());
}

/// PhoneNumberUtilTest#testGetMetadataForNonGeographicalRegionForGeoRegion_shouldBeNull
#[test]
fn get_metadata_for_non_geographical_region_for_geo_region_should_be_null() {
    assert!(util().get_metadata_for_non_geographical_region(1).is_none());
}

/// A util over the test calling-code map with no metadata files (upstream mocks the `MetadataSource`).
fn util_with_missing_metadata() -> PhoneNumberUtil {
    let map = libphonenumber::MetadataSet::testing_metadata().country_code_to_region_codes().to_vec();
    let set = libphonenumber::MetadataSet::from_files(std::iter::empty(), map).unwrap();
    PhoneNumberUtil::new(Box::leak(Box::new(set)))
}

/// PhoneNumberUtilTest#testGetMetadataForRegionForMissingMetadata (`MissingMetadataException` is a panic)
#[test]
#[should_panic(expected = "Missing metadata for region code US")]
fn get_metadata_for_region_for_missing_metadata() {
    util_with_missing_metadata().get_metadata_for_region("US");
}

/// PhoneNumberUtilTest#testGetMetadataForNonGeographicalRegionForMissingMetadata
#[test]
#[should_panic(expected = "Missing metadata for country code 800")]
fn get_metadata_for_non_geographical_region_for_missing_metadata() {
    util_with_missing_metadata().get_metadata_for_non_geographical_region(800);
}

/// `NumberParseException.toString()`, which upstream's messages above rely on.
#[test]
fn number_parse_exception_to_string() {
    let e = parse_err("+49 0", Some("DE"));
    assert_eq!("Error type: TOO_SHORT_NSN. The string supplied is too short to be a phone number.", e.to_string());
}
