// Records differential vectors from libphonenumber 9.0.41 (see scripts/record.sh and tests/vectors.rs).
//
//   java Vectors corpus        prints the input rows, `region TAB input`, built from the metadata itself: for every
//                              supported region (sorted) and every type with an example number, the example in
//                              national digits, NATIONAL, INTERNATIONAL, E164 and RFC3966 renderings, with a
//                              leading 0, one digit short, one digit long, an extension; then random digit strings
//                              from new Random(293) per region; then fixed odd inputs; then `@` queries.
//   java Vectors record        reads `region TAB input` rows (`#` and blank lines pass through) and appends what
//                              libphonenumber answers, one TAB-separated column each.
//
// Columns of a parse row: parse(input, region) as PhoneNumber.toString() or `ERR <ErrorType>: <message>`, then (when
// it parsed) parseAndKeepRawInput's toString, isValidNumber, isValidNumberForRegion(region), getNumberType,
// getRegionCodeForNumber, isPossibleNumberWithReason, format NATIONAL, INTERNATIONAL, E164, RFC3966,
// getLengthOfGeographicalAreaCode, getLengthOfNationalDestinationCode, canBeInternationallyDialled.
// An `@query` row answers one PhoneNumberUtil query; the region column is its argument. In output, `\` TAB and
// newline are escaped as `\\` `\t` `\n`.
import com.google.i18n.phonenumbers.NumberParseException;
import com.google.i18n.phonenumbers.PhoneNumberUtil;
import com.google.i18n.phonenumbers.PhoneNumberUtil.PhoneNumberFormat;
import com.google.i18n.phonenumbers.PhoneNumberUtil.PhoneNumberType;
import com.google.i18n.phonenumbers.Phonenumber.PhoneNumber;
import java.io.*;
import java.nio.charset.StandardCharsets;
import java.util.*;

public class Vectors {
  static final PhoneNumberUtil U = PhoneNumberUtil.getInstance();

  public static void main(String[] a) throws Exception {
    PrintStream out = new PrintStream(new FileOutputStream(FileDescriptor.out), true, StandardCharsets.UTF_8);
    if (a.length > 0 && a[0].equals("corpus")) {
      corpus(out);
      return;
    }
    BufferedReader in = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8));
    String line;
    while ((line = in.readLine()) != null) {
      if (line.startsWith("#") || line.isBlank()) { out.println(line); continue; }
      String[] cols = line.split("\t", -1);
      String region = unescape(cols[0]);
      String input = unescape(cols[1]);
      List<String> results = input.startsWith("@") ? query(input, region) : parseRow(input, region);
      StringBuilder sb = new StringBuilder(cols[0]).append('\t').append(cols[1]);
      for (String r : results) sb.append('\t').append(escape(r));
      out.println(sb);
    }
  }

  static List<String> parseRow(String input, String region) {
    List<String> r = new ArrayList<>();
    PhoneNumber n;
    try {
      n = U.parse(input, region.equals("null") ? null : region);
    } catch (NumberParseException e) {
      r.add("ERR " + e.getErrorType() + ": " + e.getMessage());
      return r;
    }
    r.add(n.toString());
    try {
      r.add(U.parseAndKeepRawInput(input, region.equals("null") ? null : region).toString());
    } catch (NumberParseException e) {
      r.add("ERR " + e.getErrorType() + ": " + e.getMessage());
    }
    r.add(String.valueOf(U.isValidNumber(n)));
    r.add(String.valueOf(U.isValidNumberForRegion(n, region)));
    r.add(String.valueOf(U.getNumberType(n)));
    r.add(String.valueOf(U.getRegionCodeForNumber(n)));
    r.add(String.valueOf(U.isPossibleNumberWithReason(n)));
    for (PhoneNumberFormat f : PhoneNumberFormat.values()) r.add(U.format(n, f));
    r.add(String.valueOf(U.getLengthOfGeographicalAreaCode(n)));
    r.add(String.valueOf(U.getLengthOfNationalDestinationCode(n)));
    r.add(String.valueOf(U.canBeInternationallyDialled(n)));
    return r;
  }

  static List<String> query(String q, String arg) {
    List<String> r = new ArrayList<>();
    switch (q) {
      case "@supportedRegions" -> r.add(String.join(",", U.getSupportedRegions()));
      case "@globalNetworkCallingCodes" -> r.add(U.getSupportedGlobalNetworkCallingCodes().toString());
      case "@callingCodes" -> r.add(U.getSupportedCallingCodes().toString());
      case "@regionInfo" -> {
        r.add(String.valueOf(U.getCountryCodeForRegion(arg)));
        r.add(String.valueOf(U.getSupportedTypesForRegion(arg)));
        r.add(String.valueOf(U.getNddPrefixForRegion(arg, false)));
        r.add(String.valueOf(U.getNddPrefixForRegion(arg, true)));
        r.add(String.valueOf(U.isNANPACountry(arg)));
        r.add(String.valueOf(U.isMobileNumberPortableRegion(arg)));
        r.add(String.valueOf(U.getExampleNumber(arg)));
        r.add(String.valueOf(U.getInvalidExampleNumber(arg)));
      }
      case "@callingCodeInfo" -> {
        int code = Integer.parseInt(arg);
        r.add(U.getRegionCodeForCountryCode(code));
        r.add(String.valueOf(U.getRegionCodesForCountryCode(code)));
        r.add(String.valueOf(U.getSupportedTypesForNonGeoEntity(code)));
        r.add(String.valueOf(U.getExampleNumberForNonGeoEntity(code)));
      }
      case "@exampleForType" -> r.add(String.valueOf(U.getExampleNumberForType(PhoneNumberType.valueOf(arg))));
      default -> throw new IllegalArgumentException(q);
    }
    return r;
  }

  static void corpus(PrintStream out) {
    out.println("# Recorded from libphonenumber 9.0.41 by scripts/record.sh (scripts/Vectors.java has the columns).");
    List<String> regions = new ArrayList<>(U.getSupportedRegions());
    Collections.sort(regions);
    for (String region : regions) {
      for (PhoneNumberType type : PhoneNumberType.values()) {
        PhoneNumber example = U.getExampleNumberForType(region, type);
        if (example == null) continue;
        String nsn = U.getNationalSignificantNumber(example);
        Set<String> inputs = new LinkedHashSet<>();
        inputs.add(nsn);
        for (PhoneNumberFormat f : PhoneNumberFormat.values()) inputs.add(U.format(example, f));
        inputs.add("0" + nsn);
        inputs.add(nsn.substring(0, nsn.length() - 1));
        inputs.add(nsn + "5");
        inputs.add(U.format(example, PhoneNumberFormat.NATIONAL) + " ext. 123");
        for (String input : inputs) out.println(escape(region) + "\t" + escape(input));
      }
    }
    Random random = new Random(293);
    for (String region : regions) {
      for (int i = 0; i < 3; i++) {
        int length = 4 + random.nextInt(12);
        StringBuilder sb = new StringBuilder();
        for (int j = 0; j < length; j++) sb.append((char) ('0' + random.nextInt(10)));
        out.println(region + "\t" + sb);
      }
    }
    String[][] odd = {
      {"US", "1-800-SIX-FLAG"}, {"US", "1800 six-flag"}, {"US", "Tel: +1 (650) 253-0000 x123"},
      {"US", "(530) 583-6985 x302/x2303"}, {"US", "650 253 0000 ext 4567"}, {"US", "650 253 0000,,456"},
      {"US", "650 253 0000;123"}, {"US", "+1 650 253 0000 extension 12345"}, {"US", "011 44 20 7031 3000"},
      {"GB", "00 1 650 253 0000"}, {"GB", "+44 (0) 20 7031 3000"}, {"GB", "020 7031 3000#"},
      {"DE", "+49 (0)30 1234567"}, {"DE", "0049 30 1234567"}, {"IT", "+39 02 3661 8300"}, {"IT", "02 3661 8300"},
      {"AR", "+54 9 11 1234 5678"}, {"AR", "011 15 1234 5678"}, {"MX", "044 33 1234 5678"}, {"MX", "+52 1 33 1234 5678"},
      {"BR", "0 xx 21 2345 6789"}, {"BR", "(21) 2345-6789"}, {"RU", "8 495 123 45 67"}, {"JP", "03-1234-5678"},
      {"KR", "010-1234-5678"}, {"CN", "010 1234 5678"}, {"IN", "+91 98765 43210"}, {"AU", "0011 1 650 253 0000"},
      {"ZZ", "+800 1234 5678"}, {"ZZ", "+979 123 456 789"}, {"ZZ", "1234567"}, {"null", "+44 20 7031 3000"},
      {"US", "tel:253-0000;phone-context=www.google.com"}, {"US", "tel:+1-650-253-0000;isub=12345"},
      {"US", "tel:253-0000;phone-context=+1-650"}, {"US", "tel:253-0000;phone-context="},
      {"US", "+１ ６５０ ２５３ ００００"}, {"US", "٠١٢٣٤٥٦٧٨٩"}, {"US", "123"}, {"US", "12"}, {"US", "1"},
      {"US", ""}, {"US", "+"}, {"US", "+++1 650 253 0000"}, {"US", "abc"}, {"US", "0000000000"},
      {"US", "+0 123 456 7890"}, {"US", "+999 123 456"}, {"US", "1234567890123456789"}, {"FR", "+33 1 23 45 67 89"},
      {"FR", "01 23 45 67 89"}, {"FR", "0033 1 23 45 67 89"}, {"EE", "+372 5123 4567"}, {"MD", "+373 22 123 456"},
      {"MD", "022 123 456"}, {"NZ", "+64 3 331 6005"}, {"NZ", "03-331 6005"}, {"SG", "6123 4567"}, {"HU", "06 1 234 5678"},
    };
    for (String[] o : odd) out.println(escape(o[0]) + "\t" + escape(o[1]));
    out.println("-\t@supportedRegions");
    out.println("-\t@globalNetworkCallingCodes");
    out.println("-\t@callingCodes");
    for (PhoneNumberType type : PhoneNumberType.values()) out.println(type + "\t@exampleForType");
    for (String region : regions) out.println(region + "\t@regionInfo");
    out.println("ZZ\t@regionInfo");
    out.println("001\t@regionInfo");
    List<Integer> codes = new ArrayList<>(U.getSupportedCallingCodes());
    Collections.sort(codes);
    for (int code : codes) out.println(code + "\t@callingCodeInfo");
    out.println("0\t@callingCodeInfo");
    out.println("999\t@callingCodeInfo");
  }

  static String escape(String s) {
    return s.replace("\\", "\\\\").replace("\t", "\\t").replace("\n", "\\n");
  }

  static String unescape(String s) {
    StringBuilder sb = new StringBuilder();
    for (int i = 0; i < s.length(); i++) {
      char c = s.charAt(i);
      if (c == '\\' && i + 1 < s.length()) {
        char n = s.charAt(++i);
        sb.append(n == 't' ? '\t' : n == 'n' ? '\n' : n);
      } else {
        sb.append(c);
      }
    }
    return sb.toString();
  }
}
