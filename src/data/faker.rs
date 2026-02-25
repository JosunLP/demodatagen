/// Procedural fake data generator.
///
/// Generates realistic-looking names, emails, addresses, phone numbers,
/// dates, and other data types without relying on AI. All data is produced
/// deterministically given the same RNG state.

use rand::Rng;

/// Pool of common first names.
const FIRST_NAMES: &[&str] = &[
    "James", "Mary", "Robert", "Patricia", "John", "Jennifer", "Michael", "Linda",
    "David", "Elizabeth", "William", "Barbara", "Richard", "Susan", "Joseph", "Jessica",
    "Thomas", "Sarah", "Christopher", "Karen", "Charles", "Lisa", "Daniel", "Nancy",
    "Matthew", "Betty", "Anthony", "Margaret", "Mark", "Sandra", "Donald", "Ashley",
    "Steven", "Dorothy", "Paul", "Kimberly", "Andrew", "Emily", "Joshua", "Donna",
    "Kenneth", "Michelle", "Kevin", "Carol", "Brian", "Amanda", "George", "Melissa",
    "Timothy", "Deborah", "Ronald", "Stephanie", "Edward", "Rebecca", "Jason", "Sharon",
    "Jeffrey", "Laura", "Ryan", "Cynthia", "Jacob", "Kathleen", "Gary", "Amy",
    "Nicholas", "Angela", "Eric", "Shirley", "Jonathan", "Anna", "Stephen", "Brenda",
    "Larry", "Pamela", "Justin", "Emma", "Scott", "Nicole", "Brandon", "Helen",
    "Benjamin", "Samantha", "Samuel", "Katherine", "Raymond", "Christine", "Gregory", "Debra",
    "Frank", "Rachel", "Alexander", "Carolyn", "Patrick", "Janet", "Jack", "Catherine",
];

/// Pool of common last names.
const LAST_NAMES: &[&str] = &[
    "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis",
    "Rodriguez", "Martinez", "Hernandez", "Lopez", "Gonzalez", "Wilson", "Anderson",
    "Thomas", "Taylor", "Moore", "Jackson", "Martin", "Lee", "Perez", "Thompson",
    "White", "Harris", "Sanchez", "Clark", "Ramirez", "Lewis", "Robinson", "Walker",
    "Young", "Allen", "King", "Wright", "Scott", "Torres", "Nguyen", "Hill",
    "Flores", "Green", "Adams", "Nelson", "Baker", "Hall", "Rivera", "Campbell",
    "Mitchell", "Carter", "Roberts", "Gomez", "Phillips", "Evans", "Turner", "Diaz",
    "Parker", "Cruz", "Edwards", "Collins", "Reyes", "Stewart", "Morris", "Morales",
    "Murphy", "Cook", "Rogers", "Gutierrez", "Ortiz", "Morgan", "Cooper", "Peterson",
    "Bailey", "Reed", "Kelly", "Howard", "Ramos", "Kim", "Cox", "Ward",
    "Richardson", "Watson", "Brooks", "Chavez", "Wood", "James", "Bennett", "Gray",
    "Mendoza", "Ruiz", "Hughes", "Price", "Alvarez", "Castillo", "Sanders", "Patel",
];

/// Pool of email domains.
const EMAIL_DOMAINS: &[&str] = &[
    "gmail.com", "yahoo.com", "outlook.com", "hotmail.com", "protonmail.com",
    "icloud.com", "mail.com", "aol.com", "zoho.com", "fastmail.com",
    "example.com", "company.org", "business.net", "enterprise.io", "startup.co",
];

/// Pool of street names.
const STREET_NAMES: &[&str] = &[
    "Main St", "Oak Ave", "Maple Dr", "Cedar Ln", "Pine Rd",
    "Elm St", "Washington Blvd", "Park Ave", "Lake Dr", "Hill Rd",
    "River Rd", "Forest Ave", "Sunset Blvd", "Broadway", "Church St",
    "Spring St", "Highland Ave", "Valley Rd", "Meadow Ln", "Willow Dr",
];

/// Pool of city names.
const CITIES: &[&str] = &[
    "New York", "Los Angeles", "Chicago", "Houston", "Phoenix",
    "Philadelphia", "San Antonio", "San Diego", "Dallas", "San Jose",
    "Austin", "Jacksonville", "Fort Worth", "Columbus", "Indianapolis",
    "Charlotte", "Seattle", "Denver", "Washington", "Nashville",
    "Portland", "Memphis", "Louisville", "Baltimore", "Milwaukee",
];

/// Pool of US state abbreviations.
const STATES: &[&str] = &[
    "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA",
    "HI", "ID", "IL", "IN", "IA", "KS", "KY", "LA", "ME", "MD",
    "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ",
    "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC",
    "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV", "WI", "WY",
];

/// Pool of company name parts.
const COMPANY_PREFIXES: &[&str] = &[
    "Acme", "Global", "Apex", "Prime", "Nova", "Quantum", "Stellar", "Vertex",
    "Nexus", "Synergy", "Pinnacle", "Vanguard", "Catalyst", "Fusion", "Horizon",
    "Atlas", "Genesis", "Titan", "Summit", "Core", "Infinity", "Omega", "Alpha",
];

/// Pool of company suffixes.
const COMPANY_SUFFIXES: &[&str] = &[
    "Inc", "LLC", "Corp", "Ltd", "Group", "Solutions", "Technologies", "Systems",
    "Industries", "Enterprises", "Services", "Partners", "Associates", "Consulting",
    "International", "Holdings", "Dynamics", "Labs", "Digital", "Analytics",
];

/// Pool of TLDs for URLs.
const TLDS: &[&str] = &[
    ".com", ".org", ".net", ".io", ".co", ".dev", ".app", ".tech",
];

/// Generates a random first name.
pub fn first_name<R: Rng>(rng: &mut R) -> &'static str {
    FIRST_NAMES[rng.gen_range(0..FIRST_NAMES.len())]
}

/// Generates a random last name.
pub fn last_name<R: Rng>(rng: &mut R) -> &'static str {
    LAST_NAMES[rng.gen_range(0..LAST_NAMES.len())]
}

/// Generates a full name (first + last).
pub fn full_name<R: Rng>(rng: &mut R) -> String {
    format!("{} {}", first_name(rng), last_name(rng))
}

/// Generates a realistic email address.
pub fn email<R: Rng>(rng: &mut R) -> String {
    let first = first_name(rng).to_lowercase();
    let last = last_name(rng).to_lowercase();
    let domain = EMAIL_DOMAINS[rng.gen_range(0..EMAIL_DOMAINS.len())];
    let separator = if rng.gen_bool(0.5) { "." } else { "_" };
    let suffix: u32 = rng.gen_range(1..999);

    if rng.gen_bool(0.6) {
        format!("{first}{separator}{last}@{domain}")
    } else {
        format!("{first}{separator}{last}{suffix}@{domain}")
    }
}

/// Generates an integer in the given range.
pub fn integer<R: Rng>(rng: &mut R, min: i64, max: i64) -> i64 {
    rng.gen_range(min..=max)
}

/// Generates a floating point number in the given range with 2 decimal places.
pub fn float<R: Rng>(rng: &mut R, min: f64, max: f64) -> f64 {
    let val: f64 = rng.gen_range(min..max);
    (val * 100.0).round() / 100.0
}

/// Generates a boolean value.
pub fn boolean<R: Rng>(rng: &mut R) -> bool {
    rng.gen_bool(0.5)
}

/// Generates a random date string in YYYY-MM-DD format.
pub fn date<R: Rng>(rng: &mut R) -> String {
    let year = rng.gen_range(1970..=2025);
    let month = rng.gen_range(1..=12u32);
    let max_day = match month {
        2 => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    let day = rng.gen_range(1..=max_day);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Generates a random datetime string in ISO 8601 format.
pub fn datetime<R: Rng>(rng: &mut R) -> String {
    let d = date(rng);
    let hour = rng.gen_range(0..24u32);
    let minute = rng.gen_range(0..60u32);
    let second = rng.gen_range(0..60u32);
    format!("{d}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Generates a random phone number.
pub fn phone<R: Rng>(rng: &mut R) -> String {
    let area: u32 = rng.gen_range(200..999);
    let prefix: u32 = rng.gen_range(200..999);
    let line: u32 = rng.gen_range(1000..9999);
    format!("({area}) {prefix}-{line}")
}

/// Generates a random street address.
pub fn address<R: Rng>(rng: &mut R) -> String {
    let number: u32 = rng.gen_range(1..9999);
    let street = STREET_NAMES[rng.gen_range(0..STREET_NAMES.len())];
    let city = CITIES[rng.gen_range(0..CITIES.len())];
    let state = STATES[rng.gen_range(0..STATES.len())];
    let zip: u32 = rng.gen_range(10000..99999);
    format!("{number} {street}, {city}, {state} {zip}")
}

/// Generates a random company name.
pub fn company<R: Rng>(rng: &mut R) -> String {
    let prefix = COMPANY_PREFIXES[rng.gen_range(0..COMPANY_PREFIXES.len())];
    let suffix = COMPANY_SUFFIXES[rng.gen_range(0..COMPANY_SUFFIXES.len())];
    format!("{prefix} {suffix}")
}

/// Generates a random URL.
pub fn url<R: Rng>(rng: &mut R) -> String {
    let company_name = COMPANY_PREFIXES[rng.gen_range(0..COMPANY_PREFIXES.len())].to_lowercase();
    let tld = TLDS[rng.gen_range(0..TLDS.len())];
    let protocol = if rng.gen_bool(0.8) { "https" } else { "http" };
    format!("{protocol}://www.{company_name}{tld}")
}

/// Generates a random UUID v4 string.
pub fn uuid<R: Rng>(rng: &mut R) -> String {
    let bytes: [u8; 16] = rng.gen();
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u16::from_be_bytes([bytes[4], bytes[5]]),
        u16::from_be_bytes([bytes[6], bytes[7]]) & 0x0FFF,
        (u16::from_be_bytes([bytes[8], bytes[9]]) & 0x3FFF) | 0x8000,
        u64::from_be_bytes([0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]])
    )
}

/// Generates a random IPv4 address.
pub fn ipv4<R: Rng>(rng: &mut R) -> String {
    format!(
        "{}.{}.{}.{}",
        rng.gen_range(1..255u8),
        rng.gen_range(0..255u8),
        rng.gen_range(0..255u8),
        rng.gen_range(1..255u8)
    )
}

/// Generates a random string value for a given schema field type.
///
/// Supported types: `string`, `int`, `float`, `bool`, `email`, `date`,
/// `datetime`, `phone`, `address`, `company`, `url`, `uuid`, `ipv4`, `name`.
pub fn value_for_type<R: Rng>(rng: &mut R, field_type: &str) -> String {
    match field_type {
        "string" | "name" => full_name(rng),
        "int" | "integer" => integer(rng, 0, 10000).to_string(),
        "float" | "decimal" => float(rng, 0.0, 10000.0).to_string(),
        "bool" | "boolean" => boolean(rng).to_string(),
        "email" => email(rng),
        "date" => date(rng),
        "datetime" => datetime(rng),
        "phone" => phone(rng),
        "address" => address(rng),
        "company" => company(rng),
        "url" => url(rng),
        "uuid" => uuid(rng),
        "ipv4" | "ip" => ipv4(rng),
        _ => format!("unknown_type_{}", rng.gen_range(0..1000u32)),
    }
}

/// Parses a schema string like `"name:string,age:int,email:email"` into
/// field name / type pairs.
///
/// Returns a vector of `(field_name, field_type)` tuples.
pub fn parse_schema(schema: &str) -> Vec<(String, String)> {
    schema
        .split(',')
        .filter_map(|field| {
            let parts: Vec<&str> = field.trim().splitn(2, ':').collect();
            if parts.len() == 2 {
                Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn test_rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(42)
    }

    #[test]
    fn test_first_name_deterministic() {
        let mut rng = test_rng();
        let name1 = first_name(&mut rng);
        let mut rng2 = test_rng();
        let name2 = first_name(&mut rng2);
        assert_eq!(name1, name2);
    }

    #[test]
    fn test_full_name_contains_space() {
        let mut rng = test_rng();
        let name = full_name(&mut rng);
        assert!(name.contains(' '));
    }

    #[test]
    fn test_email_contains_at() {
        let mut rng = test_rng();
        let e = email(&mut rng);
        assert!(e.contains('@'));
        assert!(e.contains('.'));
    }

    #[test]
    fn test_date_format() {
        let mut rng = test_rng();
        let d = date(&mut rng);
        assert_eq!(d.len(), 10);
        assert_eq!(&d[4..5], "-");
        assert_eq!(&d[7..8], "-");
    }

    #[test]
    fn test_phone_format() {
        let mut rng = test_rng();
        let p = phone(&mut rng);
        assert!(p.starts_with('('));
        assert!(p.contains(')'));
        assert!(p.contains('-'));
    }

    #[test]
    fn test_parse_schema() {
        let schema = "name:string,age:int,email:email,active:bool";
        let fields = parse_schema(schema);
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], ("name".to_string(), "string".to_string()));
        assert_eq!(fields[1], ("age".to_string(), "int".to_string()));
        assert_eq!(fields[2], ("email".to_string(), "email".to_string()));
        assert_eq!(fields[3], ("active".to_string(), "bool".to_string()));
    }

    #[test]
    fn test_parse_schema_with_spaces() {
        let schema = " name : string , age : int ";
        let fields = parse_schema(schema);
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].0, "name");
        assert_eq!(fields[0].1, "string");
    }

    #[test]
    fn test_value_for_type_all_types() {
        let mut rng = test_rng();
        // Verify all known types produce non-empty strings
        let types = [
            "string", "int", "float", "bool", "email", "date", "datetime",
            "phone", "address", "company", "url", "uuid", "ipv4", "name",
        ];
        for t in types {
            let val = value_for_type(&mut rng, t);
            assert!(!val.is_empty(), "Type '{t}' produced empty value");
        }
    }

    #[test]
    fn test_uuid_format() {
        let mut rng = test_rng();
        let u = uuid(&mut rng);
        let parts: Vec<&str> = u.split('-').collect();
        assert_eq!(parts.len(), 5);
    }

    #[test]
    fn test_ipv4_format() {
        let mut rng = test_rng();
        let ip = ipv4(&mut rng);
        let octets: Vec<&str> = ip.split('.').collect();
        assert_eq!(octets.len(), 4);
    }
}
