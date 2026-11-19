#!/bin/sh
# Records tests/vectors/phonenumbers.tsv from libphonenumber 9.0.41 through scripts/Vectors.java.
#   scripts/record.sh            re-records the existing rows (keeps the region and input columns)
#   scripts/record.sh --corpus   rebuilds the rows from the metadata first (see Vectors.java)
# Needs a JDK and ~/.m2/repository/com/googlecode/libphonenumber/libphonenumber/9.0.41/libphonenumber-9.0.41.jar
# (mvn dependency:get -Dartifact=com.googlecode.libphonenumber:libphonenumber:9.0.41 fetches it).
set -eu
cd "$(dirname "$0")/.."
CP="$HOME/.m2/repository/com/googlecode/libphonenumber/libphonenumber/9.0.41/libphonenumber-9.0.41.jar"
OUT=tests/vectors/phonenumbers.tsv
DIR=$(mktemp -d)
javac -nowarn -d "$DIR" -cp "$CP" scripts/Vectors.java
if [ "${1:-}" = "--corpus" ]; then
  java -cp "$CP:$DIR" Vectors corpus > "$OUT"
fi
cut -f1,2 "$OUT" | java -cp "$CP:$DIR" Vectors record > "$OUT.new" 2> "$DIR/stderr"
mv "$OUT.new" "$OUT"
rm -rf "$DIR"
