# Test coverage

libphonenumber 9.0.41's own Java tests (`upstream/libphonenumber/java/libphonenumber/test/com/google/i18n/phonenumbers/`),
mapped to their ports.

The port covers what Datafaker 2.7.0 reaches (see README.md, "Scope"). `PhoneNumberUtilTest` runs on upstream's
test metadata (`PhoneNumberUtil::for_testing()`, as `TestMetadataTestCase` does), `ExampleNumbersTest` on the real
metadata. 128 test methods are ported (107 + 7 + 14), and 18 test methods of these classes are out of scope. The
classes that test only out-of-scope code are listed at the end.

There are also differential vectors in `tests/vectors/phonenumbers.tsv`, recorded from libphonenumber 9.0.41 itself
by `scripts/record.sh` (13,624 rows): for every supported region and every number type with an example, the
example in national digits, the four formats, with a leading 0, one digit short and long, with an extension; random
digit strings per region; odd inputs (alpha numbers, RFC 3966, IDD prefixes, non-ASCII digits, extensions,
malformed input); and the region/calling-code queries, including Java's `HashSet` iteration order of the supported
regions and calling codes. Each parse row checks `parse`, `parseAndKeepRawInput`, `isValidNumber`,
`isValidNumberForRegion`, `getNumberType`, `getRegionCodeForNumber`, `isPossibleNumberWithReason`, the four formats,
`getLengthOfGeographicalAreaCode`, `getLengthOfNationalDestinationCode` and `canBeInternationallyDialled`. Every row
matched on the first replay.

`tests/resources.rs` checks that `resources/` is a verbatim copy of upstream's metadata.

## Adaptations

- Java's `null` region and `null` number arguments have no counterpart (`&str`; the region of `parse` is an
  `Option`). The assertions on them are dropped: the `null` region in `testGetCountryCodeForRegion`,
  `testGetNationalDiallingPrefixForRegion` and `testIsNANPACountry`, the two `null` numbers in
  `testFailedParseOnInvalidNumbers`.
- `testCountryWithNoNumberDesc` drops its `formatOutOfCountryCallingNumber` assertion (not ported).
- `getExampleNumberForType(type)` (no region) is `get_example_number_for_any_region`.
- `testGetMetadataForRegionForMissingMetadata` and `testGetMetadataForNonGeographicalRegionForMissingMetadata`
  expect a panic where Java throws `MissingMetadataException`.
- `PhonenumberTest` compares `hashCode()`s; the ports compare the std `Hash` of the numbers, which hashes the fields
  `exactlySameAs` compares.

## Ported

| Upstream test | Port |
|---|---|
| ExampleNumbersTest#testFixedLine | `tests/upstream_example_numbers.rs::fixed_line` |
| ExampleNumbersTest#testMobile | `tests/upstream_example_numbers.rs::mobile` |
| ExampleNumbersTest#testTollFree | `tests/upstream_example_numbers.rs::toll_free` |
| ExampleNumbersTest#testPremiumRate | `tests/upstream_example_numbers.rs::premium_rate` |
| ExampleNumbersTest#testVoip | `tests/upstream_example_numbers.rs::voip` |
| ExampleNumbersTest#testPager | `tests/upstream_example_numbers.rs::pager` |
| ExampleNumbersTest#testUan | `tests/upstream_example_numbers.rs::uan` |
| ExampleNumbersTest#testVoicemail | `tests/upstream_example_numbers.rs::voicemail` |
| ExampleNumbersTest#testSharedCost | `tests/upstream_example_numbers.rs::shared_cost` |
| ExampleNumbersTest#testCanBeInternationallyDialled | `tests/upstream_example_numbers.rs::can_be_internationally_dialled` |
| ExampleNumbersTest#testGlobalNetworkNumbers | `tests/upstream_example_numbers.rs::global_network_numbers` |
| ExampleNumbersTest#testEveryRegionHasAnExampleNumber | `tests/upstream_example_numbers.rs::every_region_has_an_example_number` |
| ExampleNumbersTest#testEveryRegionHasAnInvalidExampleNumber | `tests/upstream_example_numbers.rs::every_region_has_an_invalid_example_number` |
| ExampleNumbersTest#testEveryTypeHasAnExampleNumber | `tests/upstream_example_numbers.rs::every_type_has_an_example_number` |
| PhoneNumberUtilTest#testGetSupportedRegions | `tests/upstream_phone_number_util.rs::get_supported_regions` |
| PhoneNumberUtilTest#testGetSupportedGlobalNetworkCallingCodes | `tests/upstream_phone_number_util.rs::get_supported_global_network_calling_codes` |
| PhoneNumberUtilTest#testGetSupportedCallingCodes | `tests/upstream_phone_number_util.rs::get_supported_calling_codes` |
| PhoneNumberUtilTest#testGetInstanceLoadBadMetadata | `tests/upstream_phone_number_util.rs::get_instance_load_bad_metadata` |
| PhoneNumberUtilTest#testGetSupportedTypesForRegion | `tests/upstream_phone_number_util.rs::get_supported_types_for_region` |
| PhoneNumberUtilTest#testGetSupportedTypesForNonGeoEntity | `tests/upstream_phone_number_util.rs::get_supported_types_for_non_geo_entity` |
| PhoneNumberUtilTest#testGetInstanceLoadUSMetadata | `tests/upstream_phone_number_util.rs::get_instance_load_us_metadata` |
| PhoneNumberUtilTest#testGetInstanceLoadDEMetadata | `tests/upstream_phone_number_util.rs::get_instance_load_de_metadata` |
| PhoneNumberUtilTest#testGetInstanceLoadARMetadata | `tests/upstream_phone_number_util.rs::get_instance_load_ar_metadata` |
| PhoneNumberUtilTest#testGetInstanceLoadInternationalTollFreeMetadata | `tests/upstream_phone_number_util.rs::get_instance_load_international_toll_free_metadata` |
| PhoneNumberUtilTest#testIsNumberGeographical | `tests/upstream_phone_number_util.rs::is_number_geographical` |
| PhoneNumberUtilTest#testGetLengthOfGeographicalAreaCode | `tests/upstream_phone_number_util.rs::get_length_of_geographical_area_code` |
| PhoneNumberUtilTest#testGetLengthOfNationalDestinationCode | `tests/upstream_phone_number_util.rs::get_length_of_national_destination_code` |
| PhoneNumberUtilTest#testGetCountryMobileToken | `tests/upstream_phone_number_util.rs::get_country_mobile_token` |
| PhoneNumberUtilTest#testGetNationalSignificantNumber | `tests/upstream_phone_number_util.rs::get_national_significant_number` |
| PhoneNumberUtilTest#testGetNationalSignificantNumber_ManyLeadingZeros | `tests/upstream_phone_number_util.rs::get_national_significant_number_many_leading_zeros` |
| PhoneNumberUtilTest#testGetExampleNumber | `tests/upstream_phone_number_util.rs::get_example_number` |
| PhoneNumberUtilTest#testGetInvalidExampleNumber | `tests/upstream_phone_number_util.rs::get_invalid_example_number` |
| PhoneNumberUtilTest#testGetExampleNumberForNonGeoEntity | `tests/upstream_phone_number_util.rs::get_example_number_for_non_geo_entity` |
| PhoneNumberUtilTest#testGetExampleNumberWithoutRegion | `tests/upstream_phone_number_util.rs::get_example_number_without_region` |
| PhoneNumberUtilTest#testConvertAlphaCharactersInNumber | `tests/upstream_phone_number_util.rs::convert_alpha_characters_in_number` |
| PhoneNumberUtilTest#testNormaliseRemovePunctuation | `tests/upstream_phone_number_util.rs::normalise_remove_punctuation` |
| PhoneNumberUtilTest#testNormaliseReplaceAlphaCharacters | `tests/upstream_phone_number_util.rs::normalise_replace_alpha_characters` |
| PhoneNumberUtilTest#testNormaliseOtherDigits | `tests/upstream_phone_number_util.rs::normalise_other_digits` |
| PhoneNumberUtilTest#testNormaliseStripAlphaCharacters | `tests/upstream_phone_number_util.rs::normalise_strip_alpha_characters` |
| PhoneNumberUtilTest#testNormaliseStripNonDiallableCharacters | `tests/upstream_phone_number_util.rs::normalise_strip_non_diallable_characters` |
| PhoneNumberUtilTest#testFormatUSNumber | `tests/upstream_phone_number_util.rs::format_us_number` |
| PhoneNumberUtilTest#testFormatAUShortCodeNumber | `tests/upstream_phone_number_util.rs::format_au_short_code_number` |
| PhoneNumberUtilTest#testFormatBSNumber | `tests/upstream_phone_number_util.rs::format_bs_number` |
| PhoneNumberUtilTest#testFormatGBNumber | `tests/upstream_phone_number_util.rs::format_gb_number` |
| PhoneNumberUtilTest#testFormatDENumber | `tests/upstream_phone_number_util.rs::format_de_number` |
| PhoneNumberUtilTest#testFormatITNumber | `tests/upstream_phone_number_util.rs::format_it_number` |
| PhoneNumberUtilTest#testFormatAUNumber | `tests/upstream_phone_number_util.rs::format_au_number` |
| PhoneNumberUtilTest#testFormatARNumber | `tests/upstream_phone_number_util.rs::format_ar_number` |
| PhoneNumberUtilTest#testFormatMXNumber | `tests/upstream_phone_number_util.rs::format_mx_number` |
| PhoneNumberUtilTest#testFormatWithCarrierCode | `tests/upstream_phone_number_util.rs::format_with_carrier_code` |
| PhoneNumberUtilTest#testFormatWithPreferredCarrierCode | `tests/upstream_phone_number_util.rs::format_with_preferred_carrier_code` |
| PhoneNumberUtilTest#testFormatByPattern | `tests/upstream_phone_number_util.rs::format_by_pattern` |
| PhoneNumberUtilTest#testFormatE164Number | `tests/upstream_phone_number_util.rs::format_e164_number` |
| PhoneNumberUtilTest#testFormatNumberWithExtension | `tests/upstream_phone_number_util.rs::format_number_with_extension` |
| PhoneNumberUtilTest#testIsPremiumRate | `tests/upstream_phone_number_util.rs::is_premium_rate` |
| PhoneNumberUtilTest#testIsTollFree | `tests/upstream_phone_number_util.rs::is_toll_free` |
| PhoneNumberUtilTest#testIsMobile | `tests/upstream_phone_number_util.rs::is_mobile` |
| PhoneNumberUtilTest#testIsFixedLine | `tests/upstream_phone_number_util.rs::is_fixed_line` |
| PhoneNumberUtilTest#testIsFixedLineAndMobile | `tests/upstream_phone_number_util.rs::is_fixed_line_and_mobile` |
| PhoneNumberUtilTest#testIsSharedCost | `tests/upstream_phone_number_util.rs::is_shared_cost` |
| PhoneNumberUtilTest#testIsVoip | `tests/upstream_phone_number_util.rs::is_voip` |
| PhoneNumberUtilTest#testIsPersonalNumber | `tests/upstream_phone_number_util.rs::is_personal_number` |
| PhoneNumberUtilTest#testIsUnknown | `tests/upstream_phone_number_util.rs::is_unknown` |
| PhoneNumberUtilTest#testIsValidNumber | `tests/upstream_phone_number_util.rs::is_valid_number` |
| PhoneNumberUtilTest#testIsValidForRegion | `tests/upstream_phone_number_util.rs::is_valid_for_region` |
| PhoneNumberUtilTest#testIsNotValidNumber | `tests/upstream_phone_number_util.rs::is_not_valid_number` |
| PhoneNumberUtilTest#testGetRegionCodeForCountryCode | `tests/upstream_phone_number_util.rs::get_region_code_for_country_code` |
| PhoneNumberUtilTest#testGetRegionCodeForNumber | `tests/upstream_phone_number_util.rs::get_region_code_for_number` |
| PhoneNumberUtilTest#testGetRegionCodesForCountryCode | `tests/upstream_phone_number_util.rs::get_region_codes_for_country_code` |
| PhoneNumberUtilTest#testGetCountryCodeForRegion | `tests/upstream_phone_number_util.rs::get_country_code_for_region` |
| PhoneNumberUtilTest#testGetNationalDiallingPrefixForRegion | `tests/upstream_phone_number_util.rs::get_national_dialling_prefix_for_region` |
| PhoneNumberUtilTest#testIsNANPACountry | `tests/upstream_phone_number_util.rs::is_nanpa_country` |
| PhoneNumberUtilTest#testIsPossibleNumber | `tests/upstream_phone_number_util.rs::is_possible_number` |
| PhoneNumberUtilTest#testIsPossibleNumberForType_DifferentTypeLengths | `tests/upstream_phone_number_util.rs::is_possible_number_for_type_different_type_lengths` |
| PhoneNumberUtilTest#testIsPossibleNumberForType_LocalOnly | `tests/upstream_phone_number_util.rs::is_possible_number_for_type_local_only` |
| PhoneNumberUtilTest#testIsPossibleNumberForType_DataMissingForSizeReasons | `tests/upstream_phone_number_util.rs::is_possible_number_for_type_data_missing_for_size_reasons` |
| PhoneNumberUtilTest#testIsPossibleNumberForType_NumberTypeNotSupportedForRegion | `tests/upstream_phone_number_util.rs::is_possible_number_for_type_number_type_not_supported_for_region` |
| PhoneNumberUtilTest#testIsPossibleNumberWithReason | `tests/upstream_phone_number_util.rs::is_possible_number_with_reason` |
| PhoneNumberUtilTest#testIsPossibleNumberForTypeWithReason_DifferentTypeLengths | `tests/upstream_phone_number_util.rs::is_possible_number_for_type_with_reason_different_type_lengths` |
| PhoneNumberUtilTest#testIsPossibleNumberForTypeWithReason_LocalOnly | `tests/upstream_phone_number_util.rs::is_possible_number_for_type_with_reason_local_only` |
| PhoneNumberUtilTest#testIsPossibleNumberForTypeWithReason_DataMissingForSizeReasons | `tests/upstream_phone_number_util.rs::is_possible_number_for_type_with_reason_data_missing_for_size_reasons` |
| PhoneNumberUtilTest#testIsPossibleNumberForTypeWithReason_NumberTypeNotSupportedForRegion | `tests/upstream_phone_number_util.rs::is_possible_number_for_type_with_reason_number_type_not_supported_for_region` |
| PhoneNumberUtilTest#testIsPossibleNumberForTypeWithReason_FixedLineOrMobile | `tests/upstream_phone_number_util.rs::is_possible_number_for_type_with_reason_fixed_line_or_mobile` |
| PhoneNumberUtilTest#testIsNotPossibleNumber | `tests/upstream_phone_number_util.rs::is_not_possible_number` |
| PhoneNumberUtilTest#testTruncateTooLongNumber | `tests/upstream_phone_number_util.rs::truncate_too_long_number` |
| PhoneNumberUtilTest#testIsViablePhoneNumber | `tests/upstream_phone_number_util.rs::is_viable_phone_number` |
| PhoneNumberUtilTest#testIsViablePhoneNumberNonAscii | `tests/upstream_phone_number_util.rs::is_viable_phone_number_non_ascii` |
| PhoneNumberUtilTest#testExtractPossibleNumber | `tests/upstream_phone_number_util.rs::extract_possible_number` |
| PhoneNumberUtilTest#testMaybeStripNationalPrefix | `tests/upstream_phone_number_util.rs::maybe_strip_national_prefix` |
| PhoneNumberUtilTest#testMaybeStripInternationalPrefix | `tests/upstream_phone_number_util.rs::maybe_strip_international_prefix` |
| PhoneNumberUtilTest#testMaybeExtractCountryCode | `tests/upstream_phone_number_util.rs::maybe_extract_country_code` |
| PhoneNumberUtilTest#testParseNationalNumber | `tests/upstream_phone_number_util.rs::parse_national_number` |
| PhoneNumberUtilTest#testParseNumberWithAlphaCharacters | `tests/upstream_phone_number_util.rs::parse_number_with_alpha_characters` |
| PhoneNumberUtilTest#testParseMaliciousInput | `tests/upstream_phone_number_util.rs::parse_malicious_input` |
| PhoneNumberUtilTest#testParseWithInternationalPrefixes | `tests/upstream_phone_number_util.rs::parse_with_international_prefixes` |
| PhoneNumberUtilTest#testParseNonAscii | `tests/upstream_phone_number_util.rs::parse_non_ascii` |
| PhoneNumberUtilTest#testParseWithLeadingZero | `tests/upstream_phone_number_util.rs::parse_with_leading_zero` |
| PhoneNumberUtilTest#testParseNationalNumberArgentina | `tests/upstream_phone_number_util.rs::parse_national_number_argentina` |
| PhoneNumberUtilTest#testParseWithXInNumber | `tests/upstream_phone_number_util.rs::parse_with_x_in_number` |
| PhoneNumberUtilTest#testParseNumbersMexico | `tests/upstream_phone_number_util.rs::parse_numbers_mexico` |
| PhoneNumberUtilTest#testFailedParseOnInvalidNumbers | `tests/upstream_phone_number_util.rs::failed_parse_on_invalid_numbers` |
| PhoneNumberUtilTest#testParseNumbersWithPlusWithNoRegion | `tests/upstream_phone_number_util.rs::parse_numbers_with_plus_with_no_region` |
| PhoneNumberUtilTest#testParseNumberTooShortIfNationalPrefixStripped | `tests/upstream_phone_number_util.rs::parse_number_too_short_if_national_prefix_stripped` |
| PhoneNumberUtilTest#testParseExtensions | `tests/upstream_phone_number_util.rs::parse_extensions` |
| PhoneNumberUtilTest#testParseHandlesLongExtensionsWithExplicitLabels | `tests/upstream_phone_number_util.rs::parse_handles_long_extensions_with_explicit_labels` |
| PhoneNumberUtilTest#testParseHandlesLongExtensionsWithAutoDiallingLabels | `tests/upstream_phone_number_util.rs::parse_handles_long_extensions_with_auto_dialling_labels` |
| PhoneNumberUtilTest#testParseHandlesShortExtensionsWithAmbiguousChar | `tests/upstream_phone_number_util.rs::parse_handles_short_extensions_with_ambiguous_char` |
| PhoneNumberUtilTest#testParseHandlesShortExtensionsWhenNotSureOfLabel | `tests/upstream_phone_number_util.rs::parse_handles_short_extensions_when_not_sure_of_label` |
| PhoneNumberUtilTest#testParseAndKeepRaw | `tests/upstream_phone_number_util.rs::parse_and_keep_raw` |
| PhoneNumberUtilTest#testParseItalianLeadingZeros | `tests/upstream_phone_number_util.rs::parse_italian_leading_zeros` |
| PhoneNumberUtilTest#testParseWithPhoneContext | `tests/upstream_phone_number_util.rs::parse_with_phone_context` |
| PhoneNumberUtilTest#testCountryWithNoNumberDesc | `tests/upstream_phone_number_util.rs::country_with_no_number_desc` |
| PhoneNumberUtilTest#testUnknownCountryCallingCode | `tests/upstream_phone_number_util.rs::unknown_country_calling_code` |
| PhoneNumberUtilTest#testCanBeInternationallyDialled | `tests/upstream_phone_number_util.rs::can_be_internationally_dialled` |
| PhoneNumberUtilTest#testIsAlphaNumber | `tests/upstream_phone_number_util.rs::is_alpha_number` |
| PhoneNumberUtilTest#testIsMobileNumberPortableRegion | `tests/upstream_phone_number_util.rs::is_mobile_number_portable_region` |
| PhoneNumberUtilTest#testGetMetadataForRegionForNonGeoEntity_shouldBeNull | `tests/upstream_phone_number_util.rs::get_metadata_for_region_for_non_geo_entity_should_be_null` |
| PhoneNumberUtilTest#testGetMetadataForRegionForUnknownRegion_shouldBeNull | `tests/upstream_phone_number_util.rs::get_metadata_for_region_for_unknown_region_should_be_null` |
| PhoneNumberUtilTest#testGetMetadataForNonGeographicalRegionForGeoRegion_shouldBeNull | `tests/upstream_phone_number_util.rs::get_metadata_for_non_geographical_region_for_geo_region_should_be_null` |
| PhoneNumberUtilTest#testGetMetadataForRegionForMissingMetadata | `tests/upstream_phone_number_util.rs::get_metadata_for_region_for_missing_metadata` |
| PhoneNumberUtilTest#testGetMetadataForNonGeographicalRegionForMissingMetadata | `tests/upstream_phone_number_util.rs::get_metadata_for_non_geographical_region_for_missing_metadata` |
| PhonenumberTest#testEqualSimpleNumber | `tests/upstream_phonenumber.rs::equal_simple_number` |
| PhonenumberTest#testEqualWithItalianLeadingZeroSetToDefault | `tests/upstream_phonenumber.rs::equal_with_italian_leading_zero_set_to_default` |
| PhonenumberTest#testEqualWithCountryCodeSourceSet | `tests/upstream_phonenumber.rs::equal_with_country_code_source_set` |
| PhonenumberTest#testNonEqualWithItalianLeadingZeroSetToTrue | `tests/upstream_phonenumber.rs::non_equal_with_italian_leading_zero_set_to_true` |
| PhonenumberTest#testNonEqualWithDifferingRawInput | `tests/upstream_phonenumber.rs::non_equal_with_differing_raw_input` |
| PhonenumberTest#testNonEqualWithPreferredDomesticCarrierCodeSetToDefault | `tests/upstream_phonenumber.rs::non_equal_with_preferred_domestic_carrier_code_set_to_default` |
| PhonenumberTest#testEqualWithPreferredDomesticCarrierCodeSetToDefault | `tests/upstream_phonenumber.rs::equal_with_preferred_domestic_carrier_code_set_to_default` |

## Out of scope

| Upstream test | Why |
|---|---|
| PhoneNumberUtilTest#testFormatOutOfCountryCallingNumber | `formatOutOfCountryCallingNumber` is not ported (Datafaker does not call it) |
| PhoneNumberUtilTest#testFormatOutOfCountryWithInvalidRegion | same |
| PhoneNumberUtilTest#testFormatOutOfCountryWithPreferredIntlPrefix | same |
| PhoneNumberUtilTest#testFormatOutOfCountryKeepingAlphaChars | `formatOutOfCountryKeepingAlphaChars` is not ported |
| PhoneNumberUtilTest#testFormatNumberForMobileDialing | `formatNumberForMobileDialing` is not ported |
| PhoneNumberUtilTest#testFormatInOriginalFormat | `formatInOriginalFormat` is not ported |
| PhoneNumberUtilTest#testIsNumberMatchMatches | `isNumberMatch` is not ported |
| PhoneNumberUtilTest#testIsNumberMatchShortMatchIfDiffNumLeadingZeros | same |
| PhoneNumberUtilTest#testIsNumberMatchAcceptsProtoDefaultsAsMatch | same |
| PhoneNumberUtilTest#testIsNumberMatchMatchesDiffLeadingZerosIfItalianLeadingZeroFalse | same |
| PhoneNumberUtilTest#testIsNumberMatchIgnoresSomeFields | same |
| PhoneNumberUtilTest#testIsNumberMatchNonMatches | same |
| PhoneNumberUtilTest#testIsNumberMatchNsnMatches | same |
| PhoneNumberUtilTest#testIsNumberMatchShortNsnMatches | same |
| ExampleNumbersTest#testShortNumbersValidAndCorrectCost | `ShortNumberInfo` and the short-number metadata are not ported |
| ExampleNumbersTest#testEmergency | same |
| ExampleNumbersTest#testCarrierSpecificShortNumbers | same |
| ExampleNumbersTest#testSmsServiceShortNumbers | same |
| AsYouTypeFormatterTest (33 methods) | `AsYouTypeFormatter` is not ported |
| PhoneNumberMatcherTest (46), PhoneNumberMatchTest (2) | `findNumbers`/`PhoneNumberMatcher` is not ported |
| ShortNumberInfoTest (28) | `ShortNumberInfo` is not ported |
| metadata/, internal/ test packages | metadata loading through `MetadataLoader`/class path, `RegexCache` internals, the XML metadata builder: replaced by the embedded serialized files |
