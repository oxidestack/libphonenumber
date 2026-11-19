//! Ports of libphonenumber 9.0.41's `ExampleNumbersTest`, run on the real metadata (`PhoneNumberUtil::get_instance()`):
//! every example number of the metadata parses, validates and has the type it is an example of. The four
//! `ShortNumberInfo` methods (`testShortNumbersValidAndCorrectCost`, `testEmergency`,
//! `testCarrierSpecificShortNumbers`, `testSmsServiceShortNumbers`) are out of scope: short numbers are not ported.

use libphonenumber::{PhoneNumber, PhoneNumberType, PhoneNumberUtil};

fn util() -> &'static PhoneNumberUtil {
    PhoneNumberUtil::get_instance()
}

/// `checkNumbersValidAndCorrectType`: the invalid example numbers and those of a wrong type.
fn check_numbers_valid_and_correct_type(
    requested: PhoneNumberType,
    expected: &[PhoneNumberType],
) -> (Vec<PhoneNumber>, Vec<(PhoneNumber, PhoneNumberType)>) {
    let u = util();
    let mut invalid = Vec::new();
    let mut wrong_type = Vec::new();
    for region in u.get_supported_regions() {
        if let Some(example) = u.get_example_number_for_type(region, requested) {
            if !u.is_valid_number(&example) {
                invalid.push(example);
            } else {
                let actual = u.get_number_type(&example);
                if !expected.contains(&actual) {
                    wrong_type.push((example, actual));
                }
            }
        }
    }
    (invalid, wrong_type)
}

fn assert_valid_and_correct_type(requested: PhoneNumberType, expected: &[PhoneNumberType]) {
    let (invalid, wrong_type) = check_numbers_valid_and_correct_type(requested, expected);
    assert!(invalid.is_empty(), "invalid: {invalid:?}");
    assert!(wrong_type.is_empty(), "wrong type: {wrong_type:?}");
}

/// ExampleNumbersTest#testFixedLine
#[test]
fn fixed_line() {
    assert_valid_and_correct_type(PhoneNumberType::FixedLine, &[PhoneNumberType::FixedLine, PhoneNumberType::FixedLineOrMobile]);
}

/// ExampleNumbersTest#testMobile
#[test]
fn mobile() {
    assert_valid_and_correct_type(PhoneNumberType::Mobile, &[PhoneNumberType::Mobile, PhoneNumberType::FixedLineOrMobile]);
}

/// ExampleNumbersTest#testTollFree
#[test]
fn toll_free() {
    assert_valid_and_correct_type(PhoneNumberType::TollFree, &[PhoneNumberType::TollFree]);
}

/// ExampleNumbersTest#testPremiumRate
#[test]
fn premium_rate() {
    assert_valid_and_correct_type(PhoneNumberType::PremiumRate, &[PhoneNumberType::PremiumRate]);
}

/// ExampleNumbersTest#testVoip
#[test]
fn voip() {
    assert_valid_and_correct_type(PhoneNumberType::Voip, &[PhoneNumberType::Voip]);
}

/// ExampleNumbersTest#testPager
#[test]
fn pager() {
    assert_valid_and_correct_type(PhoneNumberType::Pager, &[PhoneNumberType::Pager]);
}

/// ExampleNumbersTest#testUan
#[test]
fn uan() {
    assert_valid_and_correct_type(PhoneNumberType::Uan, &[PhoneNumberType::Uan]);
}

/// ExampleNumbersTest#testVoicemail
#[test]
fn voicemail() {
    assert_valid_and_correct_type(PhoneNumberType::Voicemail, &[PhoneNumberType::Voicemail]);
}

/// ExampleNumbersTest#testSharedCost
#[test]
fn shared_cost() {
    assert_valid_and_correct_type(PhoneNumberType::SharedCost, &[PhoneNumberType::SharedCost]);
}

/// ExampleNumbersTest#testCanBeInternationallyDialled
#[test]
fn can_be_internationally_dialled() {
    let u = util();
    let mut wrong = Vec::new();
    for region in u.get_supported_regions() {
        let metadata = u.get_metadata_for_region(region).unwrap();
        let desc = metadata.no_international_dialling.as_ref().unwrap();
        let example = if desc.has_example_number { u.parse(&desc.example_number, Some(region)).ok() } else { None };
        if let Some(example) = example {
            if u.can_be_internationally_dialled(&example) {
                wrong.push(example);
            }
        }
    }
    assert!(wrong.is_empty(), "{wrong:?}");
}

/// ExampleNumbersTest#testGlobalNetworkNumbers
#[test]
fn global_network_numbers() {
    let u = util();
    for &code in u.get_supported_global_network_calling_codes() {
        let example = u.get_example_number_for_non_geo_entity(code);
        let example = example.unwrap_or_else(|| panic!("No example phone number for calling code {code}"));
        assert!(u.is_valid_number(&example), "Failed validation for {example}");
    }
}

/// ExampleNumbersTest#testEveryRegionHasAnExampleNumber
#[test]
fn every_region_has_an_example_number() {
    for region in util().get_supported_regions() {
        assert!(util().get_example_number(region).is_some(), "No example number found for region {region}");
    }
}

/// ExampleNumbersTest#testEveryRegionHasAnInvalidExampleNumber
#[test]
fn every_region_has_an_invalid_example_number() {
    for region in util().get_supported_regions() {
        assert!(util().get_invalid_example_number(region).is_some(), "No invalid example number found for region {region}");
    }
}

/// ExampleNumbersTest#testEveryTypeHasAnExampleNumber (`getExampleNumberForType(type)` is
/// `get_example_number_for_any_region`)
#[test]
fn every_type_has_an_example_number() {
    for number_type in PhoneNumberType::ALL {
        if number_type == PhoneNumberType::Unknown {
            continue;
        }
        assert!(
            util().get_example_number_for_any_region(number_type).is_some(),
            "No example number found for type {}",
            number_type.name()
        );
    }
}
