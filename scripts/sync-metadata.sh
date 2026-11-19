#!/bin/sh
# Copies libphonenumber 9.0.41's Java runtime metadata into resources/, verbatim:
#   resources/data/PhoneNumberMetadataProto_*            the serialized PhoneMetadataCollection of every region and
#                                                        non-geographical calling code, as the JVM loads them
#   resources/CountryCodeToRegionCodeMap.java            the calling-code-to-regions map (build.rs reads the Java)
#   resources/testing/PhoneNumberMetadataProtoForTesting_*, resources/testing/CountryCodeToRegionCodeMapForTesting.java
#                                                        the test metadata upstream's unit tests run against
# build.rs embeds resources/ into the crate, so it builds without the upstream submodule; tests/resources.rs fails
# when the copy and upstream differ.
set -eu
cd "$(dirname "$0")/.."
JAVA=upstream/libphonenumber/java/libphonenumber
rm -rf resources
mkdir -p resources/data resources/testing
cp "$JAVA"/src/com/google/i18n/phonenumbers/data/PhoneNumberMetadataProto_* resources/data/
cp "$JAVA"/src/com/google/i18n/phonenumbers/CountryCodeToRegionCodeMap.java resources/
cp "$JAVA"/test/com/google/i18n/phonenumbers/data/PhoneNumberMetadataProtoForTesting_* resources/testing/
cp "$JAVA"/test/com/google/i18n/phonenumbers/CountryCodeToRegionCodeMapForTesting.java resources/testing/
