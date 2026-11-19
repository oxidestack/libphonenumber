//! Ports of libphonenumber 9.0.41's `PhonenumberTest`: equality of `PhoneNumber` (`exactlySameAs`). Upstream also
//! compares `hashCode()`; here the std `Hash` of the numbers is compared the same way.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use libphonenumber::{CountryCodeSource, PhoneNumber};

fn hash(n: &PhoneNumber) -> u64 {
    let mut h = DefaultHasher::new();
    n.hash(&mut h);
    h.finish()
}

fn us() -> PhoneNumber {
    let mut n = PhoneNumber::new();
    n.set_country_code(1).set_national_number(6502530000);
    n
}

/// PhonenumberTest#testEqualSimpleNumber
#[test]
fn equal_simple_number() {
    let (a, b) = (us(), us());
    assert_eq!(a, b);
    assert_eq!(hash(&a), hash(&b));
}

/// PhonenumberTest#testEqualWithItalianLeadingZeroSetToDefault
#[test]
fn equal_with_italian_leading_zero_set_to_default() {
    let mut a = us();
    a.set_italian_leading_zero(false);
    let b = us();
    assert_eq!(a, b);
    assert_eq!(hash(&a), hash(&b));
}

/// PhonenumberTest#testEqualWithCountryCodeSourceSet
#[test]
fn equal_with_country_code_source_set() {
    let mut a = PhoneNumber::new();
    a.set_raw_input("+1 650 253 00 00").set_country_code_source(CountryCodeSource::FromNumberWithPlusSign);
    let mut b = PhoneNumber::new();
    b.set_raw_input("+1 650 253 00 00").set_country_code_source(CountryCodeSource::FromNumberWithPlusSign);
    assert_eq!(a, b);
    assert_eq!(hash(&a), hash(&b));
}

/// PhonenumberTest#testNonEqualWithItalianLeadingZeroSetToTrue
#[test]
fn non_equal_with_italian_leading_zero_set_to_true() {
    let mut a = us();
    a.set_italian_leading_zero(true);
    let b = us();
    assert_ne!(a, b);
    assert_ne!(hash(&a), hash(&b));
}

/// PhonenumberTest#testNonEqualWithDifferingRawInput
#[test]
fn non_equal_with_differing_raw_input() {
    let mut a = us();
    a.set_raw_input("+1 650 253 00 00").set_country_code_source(CountryCodeSource::FromNumberWithPlusSign);
    let mut b = us();
    b.set_raw_input("+1-650-253-00-00").set_country_code_source(CountryCodeSource::FromNumberWithPlusSign);
    assert_ne!(a, b);
    assert_ne!(hash(&a), hash(&b));
}

/// PhonenumberTest#testNonEqualWithPreferredDomesticCarrierCodeSetToDefault
#[test]
fn non_equal_with_preferred_domestic_carrier_code_set_to_default() {
    let mut a = us();
    a.set_preferred_domestic_carrier_code("");
    let b = us();
    assert_ne!(a, b);
    assert_ne!(hash(&a), hash(&b));
}

/// PhonenumberTest#testEqualWithPreferredDomesticCarrierCodeSetToDefault
#[test]
fn equal_with_preferred_domestic_carrier_code_set_to_default() {
    let mut a = us();
    a.set_preferred_domestic_carrier_code("");
    let mut b = us();
    b.set_preferred_domestic_carrier_code("");
    assert_eq!(a, b);
    assert_eq!(hash(&a), hash(&b));
}

/// `toString()` (not in PhonenumberTest; checks the Java rendering the other ports print).
#[test]
fn to_string() {
    let mut n = us();
    n.set_extension("12").set_country_code_source(CountryCodeSource::FromDefaultCountry);
    assert_eq!(
        "Country Code: 1 National Number: 6502530000 Extension: 12 Country Code Source: FROM_DEFAULT_COUNTRY",
        n.to_string()
    );
}
