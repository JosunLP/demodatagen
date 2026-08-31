//! Procedural fake-data generators.
//!
//! Generates realistic-looking names, emails, addresses, phone numbers, dates,
//! identifiers, and dozens of other data types without any AI or network
//! access. Every generator is deterministic given the same RNG state, and
//! locale-sensitive generators consult [`Locale`] for region-appropriate pools.
//!
//! The higher-level schema engine in [`crate::data::schema`] builds on these
//! primitives to produce typed records.
use crate::data::Locale;
use rand::{Rng, RngExt};

// ---------------------------------------------------------------------------
// Locale-aware people & places
// ---------------------------------------------------------------------------

/// Picks a random (copied) element from a non-empty slice.
fn pick<R: Rng, T: Copy>(rng: &mut R, items: &[T]) -> T {
    items[rng.random_range(0..items.len())]
}

/// Generates a random first name for the locale.
pub fn first_name<R: Rng>(rng: &mut R, locale: Locale) -> &'static str {
    pick(rng, locale.data().first_names)
}

/// Generates a random last name for the locale.
pub fn last_name<R: Rng>(rng: &mut R, locale: Locale) -> &'static str {
    pick(rng, locale.data().last_names)
}

/// Generates a full name (first + last) for the locale.
pub fn full_name<R: Rng>(rng: &mut R, locale: Locale) -> String {
    format!("{} {}", first_name(rng, locale), last_name(rng, locale))
}

/// Generates a gender label.
pub fn gender<R: Rng>(rng: &mut R) -> &'static str {
    pick(rng, &["male", "female", "non-binary", "other"])
}

/// Generates a login username from name parts.
pub fn username<R: Rng>(rng: &mut R, locale: Locale) -> String {
    let first = ascii_slug(first_name(rng, locale));
    let last = ascii_slug(last_name(rng, locale));
    match rng.random_range(0..4) {
        0 => format!("{first}.{last}"),
        1 => format!("{first}{last}{}", rng.random_range(1..99u32)),
        2 => format!("{}{last}", first.chars().next().unwrap_or('x')),
        _ => format!("{first}_{last}"),
    }
}

/// Generates a realistic email address for the locale.
pub fn email<R: Rng>(rng: &mut R, locale: Locale) -> String {
    let first = ascii_slug(first_name(rng, locale));
    let last = ascii_slug(last_name(rng, locale));
    let domain = pick(rng, EMAIL_DOMAINS);
    let separator = if rng.random_bool(0.5) { "." } else { "_" };
    if rng.random_bool(0.6) {
        format!("{first}{separator}{last}@{domain}")
    } else {
        format!(
            "{first}{separator}{last}{}@{domain}",
            rng.random_range(1..999u32)
        )
    }
}

/// Generates a random password-like string.
pub fn password<R: Rng>(rng: &mut R) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%&*";
    let len = rng.random_range(10..=16);
    (0..len).map(|_| pick(rng, CHARS) as char).collect()
}

/// Generates a phone number with the locale's dialing prefix.
pub fn phone<R: Rng>(rng: &mut R, locale: Locale) -> String {
    let prefix = locale.data().phone_prefix;
    let area: u32 = rng.random_range(100..999);
    let mid: u32 = rng.random_range(100..999);
    let line: u32 = rng.random_range(1000..9999);
    format!("{prefix} {area} {mid} {line}")
}

/// Generates a street address line (number + street name).
///
/// The number/street order follows the locale's
/// [`street_number_first`](crate::data::locale::LocaleData::street_number_first)
/// convention (e.g. "12 Main St" vs "Hauptstraße 12").
pub fn street<R: Rng>(rng: &mut R, locale: Locale) -> String {
    let number: u32 = rng.random_range(1..9999);
    let street = pick(rng, locale.data().streets);
    if locale.data().street_number_first {
        format!("{number} {street}")
    } else {
        format!("{street} {number}")
    }
}

/// Returns a random city for the locale.
pub fn city<R: Rng>(rng: &mut R, locale: Locale) -> &'static str {
    pick(rng, locale.data().cities)
}

/// Returns a random state / region for the locale.
pub fn state<R: Rng>(rng: &mut R, locale: Locale) -> &'static str {
    pick(rng, locale.data().states)
}

/// Returns the locale's country name.
pub fn country(locale: Locale) -> &'static str {
    locale.data().country
}

/// Returns the locale's ISO country code.
pub fn country_code(locale: Locale) -> &'static str {
    locale.data().country_code
}

/// Generates a postal code in a format plausible for the locale's country.
///
/// Falls back to a generic five-digit code for countries without a special
/// case. Codes are illustrative only — they are not guaranteed to be assigned.
pub fn zipcode<R: Rng>(rng: &mut R, locale: Locale) -> String {
    let upper = |rng: &mut R| (b'A' + rng.random_range(0..26u8)) as char;
    match locale.data().country_code {
        // "1234 AB"
        "NL" => format!(
            "{:04} {}{}",
            rng.random_range(1000..9999u32),
            upper(rng),
            upper(rng)
        ),
        // "123 45"
        "SE" => format!(
            "{:03} {:02}",
            rng.random_range(100..999u32),
            rng.random_range(0..99u32)
        ),
        // "12-345"
        "PL" => format!(
            "{:02}-{:03}",
            rng.random_range(0..99u32),
            rng.random_range(0..999u32)
        ),
        // "SW1 9AA"
        "GB" => format!(
            "{}{}{} {}{}{}",
            upper(rng),
            upper(rng),
            rng.random_range(1..99u32),
            rng.random_range(1..9u32),
            upper(rng),
            upper(rng)
        ),
        // "12345-678"
        "BR" => format!(
            "{:05}-{:03}",
            rng.random_range(1000..99999u32),
            rng.random_range(0..999u32)
        ),
        // "123-4567"
        "JP" => format!(
            "{:03}-{:04}",
            rng.random_range(100..999u32),
            rng.random_range(0..9999u32)
        ),
        // "123 45"
        "CZ" => format!(
            "{:03} {:02}",
            rng.random_range(100..999u32),
            rng.random_range(0..99u32)
        ),
        // Four-digit codes.
        "DK" | "NO" => format!("{:04}", rng.random_range(1000..9999u32)),
        // Generic five-digit (US, DE, FR, ES, IT, FI, TR, …)
        _ => format!("{:05}", rng.random_range(1000..99999u32)),
    }
}

/// Generates a full street address: street, city, region, postcode, country.
///
/// The component order follows the locale's house-number convention: countries
/// that write the number first (US/UK/FR) use "street, city, region zip";
/// number-last countries (DE/IT/ES/…) use "street, zip city, region".
pub fn address<R: Rng>(rng: &mut R, locale: Locale) -> String {
    let s = street(rng, locale);
    let city = city(rng, locale);
    let zip = zipcode(rng, locale);
    let state = state(rng, locale);
    if locale.data().street_number_first {
        format!("{s}, {city}, {state} {zip}")
    } else {
        format!("{s}, {zip} {city}, {state}")
    }
}

/// Generates a company name for the locale.
pub fn company<R: Rng>(rng: &mut R, locale: Locale) -> String {
    let d = locale.data();
    format!(
        "{} {}",
        pick(rng, d.company_prefixes),
        pick(rng, d.company_suffixes)
    )
}

// ---------------------------------------------------------------------------
// Business & commerce
// ---------------------------------------------------------------------------

/// Returns a random job title.
pub fn job_title<R: Rng>(rng: &mut R) -> &'static str {
    pick(rng, JOB_TITLES)
}

/// Returns a random department name.
pub fn department<R: Rng>(rng: &mut R) -> &'static str {
    pick(rng, DEPARTMENTS)
}

/// Generates a product name.
pub fn product<R: Rng>(rng: &mut R) -> String {
    format!(
        "{} {}",
        pick(rng, PRODUCT_ADJECTIVES),
        pick(rng, PRODUCT_NOUNS)
    )
}

/// Generates a stock-keeping unit code.
pub fn sku<R: Rng>(rng: &mut R) -> String {
    let letters: String = (0..3)
        .map(|_| (b'A' + rng.random_range(0..26u8)) as char)
        .collect();
    format!("{letters}-{:05}", rng.random_range(0..99999u32))
}

/// Generates a price within `[min, max]`, rounded to 2 decimals.
pub fn price<R: Rng>(rng: &mut R, min: f64, max: f64) -> f64 {
    float(rng, min, max)
}

/// Returns a three-letter ISO 4217 currency code.
pub fn currency_code<R: Rng>(rng: &mut R) -> &'static str {
    pick(
        rng,
        &[
            "USD", "EUR", "GBP", "JPY", "CHF", "CAD", "AUD", "CNY", "SEK", "NOK",
        ],
    )
}

/// Generates an IBAN-like string (not check-digit valid, demo only).
pub fn iban<R: Rng>(rng: &mut R, locale: Locale) -> String {
    let cc = locale.data().country_code;
    let check: u32 = rng.random_range(10..99);
    let bban: String = (0..18)
        .map(|_| (b'0' + rng.random_range(0..10u8)) as char)
        .collect();
    format!("{cc}{check}{bban}")
}

/// Computes the Luhn check digit for a sequence of payload digits (each 0–9).
///
/// The returned digit, appended to `payload`, makes the whole number pass the
/// Luhn (mod-10) checksum used by credit cards and IMEIs. Doubling starts at the
/// rightmost payload digit, which becomes position 1 once the check digit is
/// appended.
fn luhn_check_digit(payload: &[u8]) -> u8 {
    let mut sum = 0u32;
    for (i, d) in payload.iter().rev().enumerate() {
        let mut v = *d as u32;
        if i % 2 == 0 {
            v *= 2;
            if v > 9 {
                v -= 9;
            }
        }
        sum += v;
    }
    ((10 - (sum % 10)) % 10) as u8
}

/// Generates a credit-card-like number (Luhn-valid 16 digits).
pub fn credit_card<R: Rng>(rng: &mut R) -> String {
    let mut digits: Vec<u8> = (0..15).map(|_| rng.random_range(0..10u8)).collect();
    digits.push(luhn_check_digit(&digits));
    digits
        .chunks(4)
        .map(|c| c.iter().map(|d| (b'0' + d) as char).collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Returns a random credit-card network name (Visa, Mastercard, …).
pub fn card_network<R: Rng>(rng: &mut R) -> &'static str {
    pick(
        rng,
        &[
            "Visa",
            "Mastercard",
            "American Express",
            "Discover",
            "JCB",
            "Diners Club",
            "UnionPay",
            "Maestro",
        ],
    )
}

/// Generates a BIC / SWIFT business identifier code (8 or 11 chars, demo only).
///
/// Format: 4-letter bank code, the locale's 2-letter country code, a 2-char
/// location code, and — 50% of the time — a 3-char branch code.
pub fn bic<R: Rng>(rng: &mut R, locale: Locale) -> String {
    let alnum = |rng: &mut R| {
        let n = rng.random_range(0..36u8);
        if n < 26 {
            (b'A' + n) as char
        } else {
            (b'0' + (n - 26)) as char
        }
    };
    let upper = |rng: &mut R| (b'A' + rng.random_range(0..26u8)) as char;
    let bank: String = (0..4).map(|_| upper(rng)).collect();
    let cc = locale.data().country_code;
    let location: String = (0..2).map(|_| alnum(rng)).collect();
    if rng.random_bool(0.5) {
        let branch: String = (0..3).map(|_| alnum(rng)).collect();
        format!("{bank}{cc}{location}{branch}")
    } else {
        format!("{bank}{cc}{location}")
    }
}

/// Generates an EAN-13 barcode number with a valid check digit (demo only).
pub fn ean13<R: Rng>(rng: &mut R) -> String {
    let data: Vec<u8> = (0..12).map(|_| rng.random_range(0..10u8)).collect();
    // EAN-13: odd 1-indexed positions weight 1, even positions weight 3.
    let sum: u32 = data
        .iter()
        .enumerate()
        .map(|(i, d)| *d as u32 * if i % 2 == 0 { 1 } else { 3 })
        .sum();
    let check = ((10 - (sum % 10)) % 10) as u8;
    data.iter()
        .chain(std::iter::once(&check))
        .map(|d| (b'0' + d) as char)
        .collect()
}

/// Generates a 15-digit IMEI device identifier (Luhn-valid, demo only).
pub fn imei<R: Rng>(rng: &mut R) -> String {
    let mut digits: Vec<u8> = (0..14).map(|_| rng.random_range(0..10u8)).collect();
    digits.push(luhn_check_digit(&digits));
    digits.iter().map(|d| (b'0' + d) as char).collect()
}

/// Generates an ISBN-13 string (demo, valid prefix/format).
pub fn isbn<R: Rng>(rng: &mut R) -> String {
    let body: String = (0..9)
        .map(|_| (b'0' + rng.random_range(0..10u8)) as char)
        .collect();
    format!("978-{}-{}-{}", &body[0..1], &body[1..5], &body[5..9])
}

// ---------------------------------------------------------------------------
// Internet & identifiers
// ---------------------------------------------------------------------------

/// Generates a registrable domain name.
pub fn domain<R: Rng>(rng: &mut R) -> String {
    let name = ascii_slug(pick(rng, EN_COMPANY_WORDS));
    format!("{name}{}", pick(rng, TLDS))
}

/// Generates a URL.
pub fn url<R: Rng>(rng: &mut R, _locale: Locale) -> String {
    let protocol = if rng.random_bool(0.85) {
        "https"
    } else {
        "http"
    };
    let path = if rng.random_bool(0.5) {
        format!("/{}", slug(rng))
    } else {
        String::new()
    };
    format!("{protocol}://www.{}{path}", domain(rng))
}

/// Generates a hyphenated lowercase slug of 2-4 words.
pub fn slug<R: Rng>(rng: &mut R) -> String {
    let n = rng.random_range(2..=4);
    (0..n)
        .map(|_| crate::data::lorem::word(rng))
        .collect::<Vec<_>>()
        .join("-")
}

/// Generates a random IPv4 address.
pub fn ipv4<R: Rng>(rng: &mut R) -> String {
    format!(
        "{}.{}.{}.{}",
        rng.random_range(1..255u8),
        rng.random_range(0..255u8),
        rng.random_range(0..255u8),
        rng.random_range(1..255u8)
    )
}

/// Generates a random IPv6 address.
pub fn ipv6<R: Rng>(rng: &mut R) -> String {
    (0..8)
        .map(|_| format!("{:04x}", rng.random_range(0..=0xffffu16)))
        .collect::<Vec<_>>()
        .join(":")
}

/// Generates a random MAC address.
pub fn mac_address<R: Rng>(rng: &mut R) -> String {
    (0..6)
        .map(|_| format!("{:02x}", rng.random_range(0..=0xffu8)))
        .collect::<Vec<_>>()
        .join(":")
}

/// Generates a random UUID v4 string.
pub fn uuid<R: Rng>(rng: &mut R) -> String {
    let bytes: [u8; 16] = rng.random();
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u16::from_be_bytes([bytes[4], bytes[5]]),
        u16::from_be_bytes([bytes[6], bytes[7]]) & 0x0FFF,
        (u16::from_be_bytes([bytes[8], bytes[9]]) & 0x3FFF) | 0x8000,
        u64::from_be_bytes([
            0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        ])
    )
}

/// Returns a random user-agent string.
pub fn user_agent<R: Rng>(rng: &mut R) -> &'static str {
    pick(rng, USER_AGENTS)
}

// ---------------------------------------------------------------------------
// Identifiers, tokens & web
// ---------------------------------------------------------------------------

/// Returns a common MIME / content type string.
pub fn mime_type<R: Rng>(rng: &mut R) -> &'static str {
    pick(rng, MIME_TYPES)
}

/// Generates a filename with a plausible extension (e.g. `lorem_ipsum.pdf`).
pub fn filename<R: Rng>(rng: &mut R) -> String {
    let stem = (0..rng.random_range(1..=3))
        .map(|_| crate::data::lorem::word(rng))
        .collect::<Vec<_>>()
        .join("_");
    format!("{stem}.{}", pick(rng, FILE_EXTENSIONS))
}

/// Generates a semantic-version string (`MAJOR.MINOR.PATCH`).
pub fn semver<R: Rng>(rng: &mut R) -> String {
    format!(
        "{}.{}.{}",
        rng.random_range(0..10u32),
        rng.random_range(0..30u32),
        rng.random_range(0..50u32)
    )
}

/// Generates a social-media style hashtag (e.g. `#LoremIpsum`).
pub fn hashtag<R: Rng>(rng: &mut R) -> String {
    let mut tag = String::from("#");
    for _ in 0..rng.random_range(1..=2) {
        let word = crate::data::lorem::word(rng);
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            tag.extend(first.to_uppercase());
            tag.push_str(chars.as_str());
        }
    }
    tag
}

/// Generates a base64-like token of 24 characters.
pub fn base64_token<R: Rng>(rng: &mut R) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    (0..24).map(|_| pick(rng, CHARS) as char).collect()
}

/// Generates a lowercase hexadecimal token of `len` (at least 1) characters.
pub fn hex_token<R: Rng>(rng: &mut R, len: usize) -> String {
    (0..len.max(1))
        .map(|_| char::from_digit(rng.random_range(0..16u32), 16).unwrap())
        .collect()
}

/// Generates a US-style Social Security Number (synthetic — not a real SSN).
pub fn ssn<R: Rng>(rng: &mut R) -> String {
    format!(
        "{:03}-{:02}-{:04}",
        rng.random_range(100..900u32),
        rng.random_range(10..99u32),
        rng.random_range(1..9999u32)
    )
}

/// Returns a currency symbol.
pub fn currency_symbol<R: Rng>(rng: &mut R) -> &'static str {
    pick(rng, &["$", "€", "£", "¥", "₹", "₽", "₣", "¢", "₩", "R$"])
}

/// Generates a percentage value in `[0, 100]`, rounded to one decimal.
pub fn percent<R: Rng>(rng: &mut R) -> f64 {
    (rng.random_range(0.0..=100.0f64) * 10.0).round() / 10.0
}

/// Generates a 1.0–5.0 star rating, rounded to one decimal.
pub fn rating<R: Rng>(rng: &mut R) -> f64 {
    (rng.random_range(1.0..=5.0f64) * 10.0).round() / 10.0
}

/// Generates a TCP/UDP port in the registered/dynamic range (1024–65535).
pub fn port<R: Rng>(rng: &mut R) -> i64 {
    rng.random_range(1024..=65535) as i64
}

// ---------------------------------------------------------------------------
// Misc descriptive
// ---------------------------------------------------------------------------

/// Returns a random color name.
pub fn color_name<R: Rng>(rng: &mut R) -> &'static str {
    pick(rng, COLORS)
}

/// Generates a random `#RRGGBB` hex color.
pub fn hex_color<R: Rng>(rng: &mut R) -> String {
    format!("#{:06X}", rng.random_range(0..=0xFFFFFFu32))
}

/// Returns a random IETF language tag.
pub fn language<R: Rng>(rng: &mut R) -> &'static str {
    pick(
        rng,
        &["en", "de", "fr", "es", "it", "pt", "nl", "pl", "ja", "zh"],
    )
}

/// Returns a random IANA timezone name.
pub fn timezone<R: Rng>(rng: &mut R) -> &'static str {
    pick(
        rng,
        &[
            "UTC",
            "Europe/Berlin",
            "Europe/London",
            "America/New_York",
            "America/Los_Angeles",
            "Asia/Tokyo",
            "Australia/Sydney",
        ],
    )
}

/// Returns a single emoji.
pub fn emoji<R: Rng>(rng: &mut R) -> &'static str {
    pick(
        rng,
        &[
            "😀", "🎉", "🚀", "🔥", "💡", "📦", "✅", "⚡", "🌟", "🐳", "🦀", "📊",
        ],
    )
}

// ---------------------------------------------------------------------------
// Web, tech & geo
// ---------------------------------------------------------------------------

/// Returns a random HTTP request method.
pub fn http_method<R: Rng>(rng: &mut R) -> &'static str {
    pick(
        rng,
        &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"],
    )
}

/// Returns a plausible HTTP status code, weighted toward common ones.
pub fn http_status<R: Rng>(rng: &mut R) -> i64 {
    pick(
        rng,
        &[
            200, 200, 200, 201, 204, 301, 302, 304, 400, 401, 403, 404, 404, 409, 422, 429, 500,
            502, 503,
        ],
    )
}

/// Returns a random operating-system name.
pub fn os_name<R: Rng>(rng: &mut R) -> &'static str {
    pick(
        rng,
        &[
            "Windows 11",
            "Windows 10",
            "macOS 14",
            "macOS 13",
            "Ubuntu 24.04",
            "Debian 12",
            "Fedora 40",
            "Arch Linux",
            "Android 14",
            "iOS 17",
        ],
    )
}

/// Returns a random web-browser name.
pub fn browser<R: Rng>(rng: &mut R) -> &'static str {
    pick(
        rng,
        &[
            "Chrome", "Firefox", "Safari", "Edge", "Opera", "Brave", "Vivaldi",
        ],
    )
}

/// Returns a random device category.
pub fn device<R: Rng>(rng: &mut R) -> &'static str {
    pick(
        rng,
        &[
            "Desktop", "Laptop", "Mobile", "Tablet", "Smart TV", "Wearable",
        ],
    )
}

/// Generates a work email of the form `first.last@<company-domain>`.
pub fn company_email<R: Rng>(rng: &mut R, locale: Locale) -> String {
    let first = ascii_slug(first_name(rng, locale));
    let last = ascii_slug(last_name(rng, locale));
    format!("{first}.{last}@{}", domain(rng))
}

/// Returns a random seniority / job level.
pub fn job_level<R: Rng>(rng: &mut R) -> &'static str {
    pick(
        rng,
        &[
            "Intern",
            "Junior",
            "Mid-level",
            "Senior",
            "Staff",
            "Lead",
            "Principal",
            "Director",
        ],
    )
}

/// Generates a human-readable file size such as `4.20 MB` or `512.00 KB`.
pub fn file_size<R: Rng>(rng: &mut R) -> String {
    let unit = pick(rng, &["B", "KB", "MB", "GB"]);
    let value: f64 = match unit {
        "B" => rng.random_range(1.0..1024.0),
        _ => rng.random_range(1.0..1000.0),
    };
    format!("{value:.2} {unit}")
}

/// Generates a `"latitude,longitude"` coordinate pair.
pub fn coordinates<R: Rng>(rng: &mut R) -> String {
    format!("{},{}", latitude(rng), longitude(rng))
}

/// Picks a major international airport's IATA code.
pub fn airport<R: Rng>(rng: &mut R) -> &'static str {
    pick(
        rng,
        &[
            "ATL", "PEK", "LHR", "HND", "ORD", "LAX", "CDG", "DFW", "FRA", "HKG", "DXB", "AMS",
            "MAD", "BKK", "JFK", "SIN", "CAN", "NRT", "IST", "SYD", "ICN", "DEN", "SFO", "BCN",
            "YYZ", "MUC", "GRU", "MEX", "ZRH", "VIE", "CPH", "OSL", "HEL", "PRG", "WAW", "ARN",
        ],
    )
}

/// Generates a flight number: a two-letter airline designator plus 1–4 digits.
pub fn flight<R: Rng>(rng: &mut R) -> String {
    let airline = pick(
        rng,
        &[
            "LH", "BA", "AF", "KL", "UA", "AA", "DL", "EK", "QF", "SK", "AY", "TK", "JL", "NH",
            "LX", "OS", "IB", "AZ",
        ],
    );
    format!("{airline}{}", rng.random_range(1..=4999u32))
}

/// Generates a 17-character VIN from the legal alphabet (no I, O, or Q).
///
/// The check digit (position 9) is not computed, so VINs are illustrative
/// only — like the IBANs and credit-card numbers, they are shaped correctly
/// but intentionally not valid identifiers.
pub fn vin<R: Rng>(rng: &mut R) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPRSTUVWXYZ0123456789";
    (0..17).map(|_| pick(rng, ALPHABET) as char).collect()
}

// ---------------------------------------------------------------------------
// Numeric & temporal
// ---------------------------------------------------------------------------

/// Generates an integer in the inclusive range `[min, max]`.
pub fn integer<R: Rng>(rng: &mut R, min: i64, max: i64) -> i64 {
    if min >= max {
        return min;
    }
    rng.random_range(min..=max)
}

/// Generates a float in `[min, max)` rounded to 2 decimals.
pub fn float<R: Rng>(rng: &mut R, min: f64, max: f64) -> f64 {
    if min >= max {
        return (min * 100.0).round() / 100.0;
    }
    let val: f64 = rng.random_range(min..max);
    (val * 100.0).round() / 100.0
}

/// Generates a latitude in `[-90, 90]`.
pub fn latitude<R: Rng>(rng: &mut R) -> f64 {
    let v: f64 = rng.random_range(-90.0..=90.0);
    (v * 1e6).round() / 1e6
}

/// Generates a longitude in `[-180, 180]`.
pub fn longitude<R: Rng>(rng: &mut R) -> f64 {
    let v: f64 = rng.random_range(-180.0..=180.0);
    (v * 1e6).round() / 1e6
}

/// Generates a boolean value.
pub fn boolean<R: Rng>(rng: &mut R) -> bool {
    rng.random_bool(0.5)
}

/// Generates a random date string in `YYYY-MM-DD` format (years 1970-2025).
pub fn date<R: Rng>(rng: &mut R) -> String {
    let year = rng.random_range(1970..=2025);
    let month = rng.random_range(1..=12u32);
    let max_day = days_in_month(year, month);
    let day = rng.random_range(1..=max_day);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Number of days in a given month, accounting for leap years.
fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Generates a time string in `HH:MM:SS` format.
pub fn time<R: Rng>(rng: &mut R) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        rng.random_range(0..24u32),
        rng.random_range(0..60u32),
        rng.random_range(0..60u32)
    )
}

/// Generates an ISO 8601 datetime string with a trailing `Z`.
pub fn datetime<R: Rng>(rng: &mut R) -> String {
    format!("{}T{}Z", date(rng), time(rng))
}

/// Generates a plausible Unix timestamp (seconds).
pub fn unix_timestamp<R: Rng>(rng: &mut R) -> i64 {
    rng.random_range(0..1_900_000_000i64)
}

/// Returns a random weekday name.
pub fn weekday<R: Rng>(rng: &mut R) -> &'static str {
    pick(
        rng,
        &[
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
        ],
    )
}

/// Returns a random month name.
pub fn month<R: Rng>(rng: &mut R) -> &'static str {
    pick(
        rng,
        &[
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ],
    )
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Converts a name to a lowercase ASCII slug so emails/usernames stay
/// ASCII-safe across every locale.
///
/// German umlauts and ligatures expand to digraphs (`ä → ae`, `ß → ss`); other
/// accented Latin letters are folded to their base letter via [`deburr`]. Any
/// remaining non-ASCII-alphanumeric character is dropped. An empty result
/// degrades to `"x"` so downstream formatting always has something to work with.
fn ascii_slug(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            'ä' | 'Ä' => out.push_str("ae"),
            'ö' | 'Ö' => out.push_str("oe"),
            'ü' | 'Ü' => out.push_str("ue"),
            'ß' => out.push_str("ss"),
            'æ' | 'Æ' => out.push_str("ae"),
            'œ' | 'Œ' => out.push_str("oe"),
            other => {
                if let Some(base) = deburr(other) {
                    out.push(base);
                } else if other.is_ascii_alphanumeric() {
                    out.push(other.to_ascii_lowercase());
                }
            }
        }
    }
    if out.is_empty() {
        out.push('x');
    }
    out
}

/// Folds a single accented Latin letter to its lowercase ASCII base, covering
/// the diacritics used by every supported locale (French, Spanish, Italian,
/// Portuguese, Dutch, Polish, Swedish, …). Returns `None` for characters that
/// have no single-letter ASCII equivalent.
fn deburr(c: char) -> Option<char> {
    Some(match c {
        'à' | 'á' | 'â' | 'ã' | 'å' | 'ą' | 'À' | 'Á' | 'Â' | 'Ã' | 'Å' | 'Ą' => 'a',
        'ç' | 'ć' | 'č' | 'Ç' | 'Ć' | 'Č' => 'c',
        'ď' | 'Ď' => 'd',
        'è' | 'é' | 'ê' | 'ë' | 'ę' | 'ě' | 'È' | 'É' | 'Ê' | 'Ë' | 'Ę' | 'Ě' => 'e',
        'ì' | 'í' | 'î' | 'ï' | 'Ì' | 'Í' | 'Î' | 'Ï' => 'i',
        'ł' | 'Ł' => 'l',
        'ñ' | 'ń' | 'Ñ' | 'Ń' => 'n',
        'ò' | 'ó' | 'ô' | 'õ' | 'ø' | 'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ø' => 'o',
        'ř' | 'Ř' => 'r',
        'ś' | 'š' | 'ş' | 'Ś' | 'Š' | 'Ş' => 's',
        'ť' | 'Ť' => 't',
        'ù' | 'ú' | 'û' | 'Ù' | 'Ú' | 'Û' => 'u',
        'ý' | 'ÿ' | 'Ý' | 'Ÿ' => 'y',
        'ż' | 'ź' | 'ž' | 'Ż' | 'Ź' | 'Ž' => 'z',
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// Static pools (locale-agnostic)
// ---------------------------------------------------------------------------

const EMAIL_DOMAINS: &[&str] = &[
    "gmail.com",
    "yahoo.com",
    "outlook.com",
    "hotmail.com",
    "protonmail.com",
    "icloud.com",
    "mail.com",
    "aol.com",
    "zoho.com",
    "fastmail.com",
    "example.com",
    "company.org",
];

const TLDS: &[&str] = &[
    ".com", ".org", ".net", ".io", ".co", ".dev", ".app", ".tech", ".de",
];

const EN_COMPANY_WORDS: &[&str] = &[
    "acme",
    "globex",
    "initech",
    "umbrella",
    "stark",
    "wayne",
    "wonka",
    "hooli",
    "pied",
    "soylent",
    "vandelay",
    "nakatomi",
    "cyberdyne",
    "tyrell",
    "aperture",
    "blackmesa",
    "octan",
    "gringotts",
];

const JOB_TITLES: &[&str] = &[
    "Software Engineer",
    "Product Manager",
    "Data Scientist",
    "UX Designer",
    "DevOps Engineer",
    "QA Analyst",
    "Sales Representative",
    "Marketing Manager",
    "HR Specialist",
    "Accountant",
    "Project Manager",
    "Systems Administrator",
    "Business Analyst",
    "Technical Writer",
    "Support Engineer",
    "CTO",
    "CEO",
    "Office Manager",
    "Recruiter",
    "Consultant",
];

const DEPARTMENTS: &[&str] = &[
    "Engineering",
    "Sales",
    "Marketing",
    "Finance",
    "Human Resources",
    "Operations",
    "Support",
    "Legal",
    "Research",
    "Product",
    "Design",
    "IT",
    "Procurement",
    "Logistics",
];

const PRODUCT_ADJECTIVES: &[&str] = &[
    "Ergonomic",
    "Sleek",
    "Rustic",
    "Handcrafted",
    "Refined",
    "Intelligent",
    "Gorgeous",
    "Compact",
    "Premium",
    "Lightweight",
    "Durable",
    "Modular",
    "Wireless",
    "Eco",
    "Smart",
];

const PRODUCT_NOUNS: &[&str] = &[
    "Chair",
    "Keyboard",
    "Lamp",
    "Bottle",
    "Backpack",
    "Headphones",
    "Monitor",
    "Mouse",
    "Desk",
    "Speaker",
    "Charger",
    "Notebook",
    "Camera",
    "Watch",
    "Bicycle",
];

const COLORS: &[&str] = &[
    "Red",
    "Orange",
    "Yellow",
    "Green",
    "Blue",
    "Indigo",
    "Violet",
    "Black",
    "White",
    "Gray",
    "Cyan",
    "Magenta",
    "Teal",
    "Maroon",
    "Navy",
    "Olive",
    "Coral",
    "Turquoise",
];

const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64; rv:121.0) Gecko/20100101 Firefox/121.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148",
    "curl/8.4.0",
];

const MIME_TYPES: &[&str] = &[
    "application/json",
    "application/xml",
    "application/pdf",
    "application/zip",
    "application/octet-stream",
    "application/javascript",
    "application/vnd.ms-excel",
    "text/html",
    "text/plain",
    "text/csv",
    "text/markdown",
    "image/png",
    "image/jpeg",
    "image/svg+xml",
    "audio/mpeg",
    "video/mp4",
];

const FILE_EXTENSIONS: &[&str] = &[
    "txt", "pdf", "csv", "json", "xml", "png", "jpg", "zip", "docx", "xlsx", "mp4", "mp3", "log",
    "md", "html", "svg",
];

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(42)
    }

    #[test]
    fn test_full_name_contains_space() {
        assert!(full_name(&mut rng(), Locale::EnUs).contains(' '));
    }

    #[test]
    fn test_locale_affects_names() {
        // The German pool should eventually produce a name with an umlaut or
        // a clearly German surname; check that the two locales differ across a
        // sample (statistically near-certain for distinct pools).
        let mut a = ChaCha8Rng::seed_from_u64(1);
        let mut b = ChaCha8Rng::seed_from_u64(1);
        let en: Vec<_> = (0..20).map(|_| full_name(&mut a, Locale::EnUs)).collect();
        let de: Vec<_> = (0..20).map(|_| full_name(&mut b, Locale::DeDe)).collect();
        assert_ne!(en, de);
    }

    #[test]
    fn test_email_is_ascii_for_german_locale() {
        let mut r = rng();
        for _ in 0..50 {
            let e = email(&mut r, Locale::DeDe);
            assert!(e.is_ascii(), "email not ASCII: {e}");
            assert!(e.contains('@'));
        }
    }

    #[test]
    fn test_credit_card_passes_luhn() {
        let mut r = rng();
        for _ in 0..20 {
            let cc = credit_card(&mut r);
            let digits: Vec<u32> = cc
                .chars()
                .filter(|c| c.is_ascii_digit())
                .map(|c| c.to_digit(10).unwrap())
                .collect();
            assert_eq!(digits.len(), 16);
            let mut sum = 0u32;
            for (i, d) in digits.iter().rev().enumerate() {
                let mut v = *d;
                if i % 2 == 1 {
                    v *= 2;
                    if v > 9 {
                        v -= 9;
                    }
                }
                sum += v;
            }
            assert_eq!(sum % 10, 0, "Luhn check failed for {cc}");
        }
    }

    #[test]
    fn test_uuid_format() {
        let u = uuid(&mut rng());
        let parts: Vec<&str> = u.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[2].chars().next(), Some('4'));
    }

    #[test]
    fn test_ipv6_format() {
        let ip = ipv6(&mut rng());
        assert_eq!(ip.split(':').count(), 8);
    }

    #[test]
    fn test_mac_format() {
        let m = mac_address(&mut rng());
        assert_eq!(m.split(':').count(), 6);
    }

    #[test]
    fn test_hex_color_format() {
        let c = hex_color(&mut rng());
        assert_eq!(c.len(), 7);
        assert!(c.starts_with('#'));
    }

    #[test]
    fn test_date_valid_ranges() {
        let mut r = rng();
        for _ in 0..200 {
            let d = date(&mut r);
            assert_eq!(d.len(), 10);
            let m: u32 = d[5..7].parse().unwrap();
            let day: u32 = d[8..10].parse().unwrap();
            assert!((1..=12).contains(&m));
            assert!((1..=31).contains(&day));
        }
    }

    #[test]
    fn test_integer_range_respected() {
        let mut r = rng();
        for _ in 0..100 {
            let v = integer(&mut r, 10, 20);
            assert!((10..=20).contains(&v));
        }
    }

    #[test]
    fn test_integer_degenerate_range() {
        assert_eq!(integer(&mut rng(), 5, 5), 5);
        assert_eq!(integer(&mut rng(), 9, 3), 9);
    }

    #[test]
    fn test_latitude_longitude_bounds() {
        let mut r = rng();
        for _ in 0..100 {
            assert!((-90.0..=90.0).contains(&latitude(&mut r)));
            assert!((-180.0..=180.0).contains(&longitude(&mut r)));
        }
    }

    #[test]
    fn test_ascii_slug_transliterates() {
        // German digraph expansion.
        assert_eq!(ascii_slug("Müller"), "mueller");
        assert_eq!(ascii_slug("Weiß"), "weiss");
        assert_eq!(ascii_slug("Öztürk"), "oeztuerk");
        // Generic Latin diacritic folding.
        assert_eq!(ascii_slug("Łukasz"), "lukasz");
        assert_eq!(ascii_slug("São"), "sao");
        assert_eq!(ascii_slug("Niño"), "nino");
        assert_eq!(ascii_slug("Sjöberg"), "sjoeberg"); // ö → oe even outside German names
        assert_eq!(ascii_slug("François"), "francois");
        // Non-alphanumeric is dropped; empty degrades to "x".
        assert_eq!(ascii_slug("---"), "x");
    }

    #[test]
    fn test_email_is_ascii_for_every_locale() {
        let mut r = rng();
        for locale in Locale::variants() {
            for _ in 0..30 {
                let e = email(&mut r, *locale);
                assert!(e.is_ascii(), "{locale} produced non-ASCII email: {e}");
                assert!(e.contains('@'));
            }
        }
    }

    #[test]
    fn test_deterministic() {
        assert_eq!(
            full_name(&mut rng(), Locale::EnUs),
            full_name(&mut rng(), Locale::EnUs)
        );
        assert_eq!(uuid(&mut rng()), uuid(&mut rng()));
    }

    /// Validates a sequence of digit chars against the Luhn (mod-10) checksum.
    fn luhn_valid(s: &str) -> bool {
        let digits: Vec<u32> = s
            .chars()
            .filter(|c| c.is_ascii_digit())
            .map(|c| c.to_digit(10).unwrap())
            .collect();
        let mut sum = 0u32;
        for (i, d) in digits.iter().rev().enumerate() {
            let mut v = *d;
            if i % 2 == 1 {
                v *= 2;
                if v > 9 {
                    v -= 9;
                }
            }
            sum += v;
        }
        sum.is_multiple_of(10)
    }

    #[test]
    fn test_imei_is_15_digits_and_luhn_valid() {
        let mut r = rng();
        for _ in 0..50 {
            let imei = imei(&mut r);
            assert_eq!(imei.len(), 15, "imei not 15 digits: {imei}");
            assert!(imei.chars().all(|c| c.is_ascii_digit()));
            assert!(luhn_valid(&imei), "imei failed Luhn: {imei}");
        }
    }

    #[test]
    fn test_ean13_check_digit_valid() {
        let mut r = rng();
        for _ in 0..50 {
            let ean = ean13(&mut r);
            assert_eq!(ean.len(), 13);
            let d: Vec<u32> = ean.chars().map(|c| c.to_digit(10).unwrap()).collect();
            let sum: u32 = d[..12]
                .iter()
                .enumerate()
                .map(|(i, x)| x * if i % 2 == 0 { 1 } else { 3 })
                .sum();
            let check = (10 - (sum % 10)) % 10;
            assert_eq!(check, d[12], "EAN-13 check digit invalid: {ean}");
        }
    }

    #[test]
    fn test_bic_length_and_country() {
        let mut r = rng();
        for _ in 0..50 {
            let bic = bic(&mut r, Locale::DeDe);
            assert!(bic.len() == 8 || bic.len() == 11, "bad BIC length: {bic}");
            assert_eq!(&bic[4..6], "DE", "BIC country segment wrong: {bic}");
            assert!(bic.chars().all(|c| c.is_ascii_alphanumeric()));
        }
    }

    #[test]
    fn test_http_status_in_known_set() {
        let mut r = rng();
        for _ in 0..100 {
            let s = http_status(&mut r);
            assert!((100..=599).contains(&s), "implausible status: {s}");
        }
    }

    #[test]
    fn test_coordinates_parse_into_bounds() {
        let mut r = rng();
        for _ in 0..50 {
            let c = coordinates(&mut r);
            let (lat, lng) = c.split_once(',').expect("coordinates need a comma");
            let lat: f64 = lat.parse().unwrap();
            let lng: f64 = lng.parse().unwrap();
            assert!((-90.0..=90.0).contains(&lat));
            assert!((-180.0..=180.0).contains(&lng));
        }
    }

    #[test]
    fn test_file_size_has_unit() {
        let mut r = rng();
        for _ in 0..30 {
            let fs = file_size(&mut r);
            assert!(
                ["B", "KB", "MB", "GB"].iter().any(|u| fs.ends_with(u)),
                "no unit: {fs}"
            );
        }
    }

    #[test]
    fn test_company_email_is_ascii() {
        let mut r = rng();
        for locale in Locale::variants() {
            for _ in 0..20 {
                let e = company_email(&mut r, *locale);
                assert!(e.is_ascii(), "{locale} company_email not ASCII: {e}");
                assert!(e.contains('@') && e.contains('.'));
            }
        }
    }
    #[test]
    fn test_airport_flight_vin_shapes() {
        let mut r = rng();
        for _ in 0..50 {
            let a = airport(&mut r);
            assert_eq!(a.len(), 3);
            assert!(a.chars().all(|c| c.is_ascii_uppercase()));

            let f = flight(&mut r);
            assert!(f.len() >= 3 && f.len() <= 6, "bad flight: {f}");
            assert!(f[..2].chars().all(|c| c.is_ascii_uppercase()));
            assert!(f[2..].chars().all(|c| c.is_ascii_digit()));

            let v = vin(&mut r);
            assert_eq!(v.len(), 17);
            assert!(
                !v.contains('I') && !v.contains('O') && !v.contains('Q'),
                "VIN uses forbidden letters: {v}"
            );
        }
    }
}
