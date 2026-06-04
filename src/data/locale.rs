//! Locale support for the faker engine.
//!
//! A [`Locale`] selects the pools of names, places, and company parts used when
//! generating realistic fake data. Locale-agnostic data (UUIDs, IP addresses,
//! colors, …) is unaffected.

/// Supported locales for fake data generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Locale {
    /// United States English (default).
    #[default]
    EnUs,
    /// German (Germany).
    DeDe,
}

impl Locale {
    /// Returns the static data pools for this locale.
    pub fn data(&self) -> &'static LocaleData {
        match self {
            Locale::EnUs => &EN_US,
            Locale::DeDe => &DE_DE,
        }
    }

    /// Returns the canonical lowercase identifier (e.g. `"en_us"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Locale::EnUs => "en_us",
            Locale::DeDe => "de_de",
        }
    }

    /// Returns all locale identifiers, for help text and the `list` command.
    pub fn all() -> &'static [&'static str] {
        &["en_us", "de_de"]
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Locale {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "en" | "en_us" | "us" => Ok(Locale::EnUs),
            "de" | "de_de" | "german" => Ok(Locale::DeDe),
            _ => Err(format!(
                "Unknown locale: '{s}'. Valid: {}",
                Locale::all().join(", ")
            )),
        }
    }
}

/// A bundle of static data pools backing a single locale.
pub struct LocaleData {
    /// Pool of first / given names.
    pub first_names: &'static [&'static str],
    /// Pool of last / family names.
    pub last_names: &'static [&'static str],
    /// Pool of street names (already including the type suffix).
    pub streets: &'static [&'static str],
    /// Pool of city names.
    pub cities: &'static [&'static str],
    /// Pool of region / state codes or names.
    pub states: &'static [&'static str],
    /// Country name for full addresses.
    pub country: &'static str,
    /// ISO 3166-1 alpha-2 country code.
    pub country_code: &'static str,
    /// Phone country dialing prefix (e.g. `+1`).
    pub phone_prefix: &'static str,
    /// Company name prefixes.
    pub company_prefixes: &'static [&'static str],
    /// Company legal-form suffixes.
    pub company_suffixes: &'static [&'static str],
}

static EN_US: LocaleData = LocaleData {
    first_names: &[
        "James",
        "Mary",
        "Robert",
        "Patricia",
        "John",
        "Jennifer",
        "Michael",
        "Linda",
        "David",
        "Elizabeth",
        "William",
        "Barbara",
        "Richard",
        "Susan",
        "Joseph",
        "Jessica",
        "Thomas",
        "Sarah",
        "Christopher",
        "Karen",
        "Charles",
        "Lisa",
        "Daniel",
        "Nancy",
        "Matthew",
        "Betty",
        "Anthony",
        "Margaret",
        "Mark",
        "Sandra",
        "Donald",
        "Ashley",
        "Steven",
        "Dorothy",
        "Paul",
        "Kimberly",
        "Andrew",
        "Emily",
        "Joshua",
        "Donna",
        "Kenneth",
        "Michelle",
        "Kevin",
        "Carol",
        "Brian",
        "Amanda",
        "George",
        "Melissa",
        "Timothy",
        "Deborah",
        "Olivia",
        "Noah",
        "Liam",
        "Emma",
        "Ava",
        "Sophia",
        "Isabella",
        "Mia",
        "Ethan",
        "Mason",
    ],
    last_names: &[
        "Smith",
        "Johnson",
        "Williams",
        "Brown",
        "Jones",
        "Garcia",
        "Miller",
        "Davis",
        "Rodriguez",
        "Martinez",
        "Hernandez",
        "Lopez",
        "Gonzalez",
        "Wilson",
        "Anderson",
        "Thomas",
        "Taylor",
        "Moore",
        "Jackson",
        "Martin",
        "Lee",
        "Perez",
        "Thompson",
        "White",
        "Harris",
        "Sanchez",
        "Clark",
        "Ramirez",
        "Lewis",
        "Robinson",
        "Walker",
        "Young",
        "Allen",
        "King",
        "Wright",
        "Scott",
        "Torres",
        "Nguyen",
        "Hill",
        "Flores",
        "Green",
        "Adams",
        "Nelson",
        "Baker",
        "Hall",
        "Rivera",
        "Campbell",
        "Mitchell",
        "Carter",
        "Roberts",
    ],
    streets: &[
        "Main St",
        "Oak Ave",
        "Maple Dr",
        "Cedar Ln",
        "Pine Rd",
        "Elm St",
        "Washington Blvd",
        "Park Ave",
        "Lake Dr",
        "Hill Rd",
        "River Rd",
        "Forest Ave",
        "Sunset Blvd",
        "Broadway",
        "Church St",
        "Spring St",
        "Highland Ave",
        "Valley Rd",
        "Meadow Ln",
        "Willow Dr",
    ],
    cities: &[
        "New York",
        "Los Angeles",
        "Chicago",
        "Houston",
        "Phoenix",
        "Philadelphia",
        "San Antonio",
        "San Diego",
        "Dallas",
        "San Jose",
        "Austin",
        "Jacksonville",
        "Fort Worth",
        "Columbus",
        "Indianapolis",
        "Charlotte",
        "Seattle",
        "Denver",
        "Washington",
        "Nashville",
        "Portland",
        "Memphis",
        "Louisville",
        "Baltimore",
        "Milwaukee",
    ],
    states: &[
        "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA",
        "KS", "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ",
        "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT",
        "VA", "WA", "WV", "WI", "WY",
    ],
    country: "United States",
    country_code: "US",
    phone_prefix: "+1",
    company_prefixes: &[
        "Acme", "Global", "Apex", "Prime", "Nova", "Quantum", "Stellar", "Vertex", "Nexus",
        "Synergy", "Pinnacle", "Vanguard", "Catalyst", "Fusion", "Horizon", "Atlas", "Genesis",
        "Titan", "Summit", "Core", "Infinity", "Omega", "Alpha",
    ],
    company_suffixes: &[
        "Inc",
        "LLC",
        "Corp",
        "Ltd",
        "Group",
        "Solutions",
        "Technologies",
        "Systems",
        "Industries",
        "Enterprises",
        "Services",
        "Partners",
        "Associates",
        "Consulting",
        "International",
        "Holdings",
        "Dynamics",
        "Labs",
        "Digital",
        "Analytics",
    ],
};

static DE_DE: LocaleData = LocaleData {
    first_names: &[
        "Maximilian",
        "Alexander",
        "Paul",
        "Leon",
        "Lukas",
        "Felix",
        "Jonas",
        "David",
        "Elias",
        "Noah",
        "Ben",
        "Finn",
        "Luis",
        "Henry",
        "Emil",
        "Marie",
        "Sophie",
        "Maria",
        "Emma",
        "Hannah",
        "Mia",
        "Anna",
        "Lena",
        "Lea",
        "Laura",
        "Lina",
        "Johanna",
        "Clara",
        "Ida",
        "Frieda",
        "Klaus",
        "Peter",
        "Hans",
        "Michael",
        "Thomas",
        "Andreas",
        "Stefan",
        "Wolfgang",
        "Jürgen",
        "Petra",
        "Sabine",
        "Ursula",
        "Monika",
        "Renate",
        "Karin",
        "Ingrid",
        "Gabriele",
        "Birgit",
        "Claudia",
    ],
    last_names: &[
        "Müller",
        "Schmidt",
        "Schneider",
        "Fischer",
        "Weber",
        "Meyer",
        "Wagner",
        "Becker",
        "Schulz",
        "Hoffmann",
        "Schäfer",
        "Koch",
        "Bauer",
        "Richter",
        "Klein",
        "Wolf",
        "Schröder",
        "Neumann",
        "Schwarz",
        "Zimmermann",
        "Braun",
        "Krüger",
        "Hofmann",
        "Hartmann",
        "Lange",
        "Schmitt",
        "Werner",
        "Schmitz",
        "Krause",
        "Meier",
        "Lehmann",
        "Schmid",
        "Schulze",
        "Maier",
        "Köhler",
        "Herrmann",
        "König",
        "Walter",
        "Mayer",
        "Huber",
        "Kaiser",
        "Fuchs",
        "Peters",
        "Lang",
        "Scholz",
        "Möller",
        "Weiß",
        "Jung",
        "Hahn",
        "Vogel",
    ],
    streets: &[
        "Hauptstraße",
        "Bahnhofstraße",
        "Gartenstraße",
        "Dorfstraße",
        "Schulstraße",
        "Bergstraße",
        "Lindenstraße",
        "Kirchstraße",
        "Waldstraße",
        "Ringstraße",
        "Goethestraße",
        "Schillerstraße",
        "Mozartstraße",
        "Beethovenstraße",
        "Wiesenweg",
        "Birkenweg",
        "Amselweg",
        "Marktplatz",
        "Rosenweg",
        "Talstraße",
    ],
    cities: &[
        "Berlin",
        "Hamburg",
        "München",
        "Köln",
        "Frankfurt",
        "Stuttgart",
        "Düsseldorf",
        "Leipzig",
        "Dortmund",
        "Essen",
        "Bremen",
        "Dresden",
        "Hannover",
        "Nürnberg",
        "Duisburg",
        "Bochum",
        "Wuppertal",
        "Bielefeld",
        "Bonn",
        "Münster",
        "Karlsruhe",
        "Mannheim",
        "Augsburg",
        "Wiesbaden",
        "Mönchengladbach",
    ],
    states: &[
        "Baden-Württemberg",
        "Bayern",
        "Berlin",
        "Brandenburg",
        "Bremen",
        "Hamburg",
        "Hessen",
        "Mecklenburg-Vorpommern",
        "Niedersachsen",
        "Nordrhein-Westfalen",
        "Rheinland-Pfalz",
        "Saarland",
        "Sachsen",
        "Sachsen-Anhalt",
        "Schleswig-Holstein",
        "Thüringen",
    ],
    country: "Deutschland",
    country_code: "DE",
    phone_prefix: "+49",
    company_prefixes: &[
        "Adler",
        "Alpen",
        "Donau",
        "Rhein",
        "Nord",
        "Süd",
        "Bavaria",
        "Hanse",
        "Kontinental",
        "Deutsche",
        "Europa",
        "Stern",
        "Berg",
        "Tal",
        "Wald",
        "See",
        "Brücke",
        "Anker",
        "Falke",
        "Phönix",
        "Zenit",
        "Vega",
    ],
    company_suffixes: &[
        "GmbH",
        "AG",
        "KG",
        "GmbH & Co. KG",
        "OHG",
        "SE",
        "e.K.",
        "Werke",
        "Gruppe",
        "Systeme",
        "Technik",
        "Industrie",
        "Handel",
        "Dienste",
        "Partner",
        "Beratung",
        "Logistik",
        "Digital",
    ],
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_parse() {
        assert_eq!("en".parse::<Locale>().unwrap(), Locale::EnUs);
        assert_eq!("de_DE".parse::<Locale>().unwrap(), Locale::DeDe);
        assert_eq!("DE-DE".parse::<Locale>().unwrap(), Locale::DeDe);
        assert!("xx".parse::<Locale>().is_err());
    }

    #[test]
    fn test_locale_roundtrip() {
        for id in Locale::all() {
            let loc: Locale = id.parse().unwrap();
            assert_eq!(&loc.as_str(), id);
        }
    }

    #[test]
    fn test_locale_data_non_empty() {
        for loc in [Locale::EnUs, Locale::DeDe] {
            let d = loc.data();
            assert!(!d.first_names.is_empty());
            assert!(!d.last_names.is_empty());
            assert!(!d.cities.is_empty());
            assert!(!d.streets.is_empty());
            assert!(!d.states.is_empty());
            assert!(!d.company_prefixes.is_empty());
            assert!(!d.company_suffixes.is_empty());
            assert!(!d.country.is_empty());
        }
    }

    #[test]
    fn test_default_locale_is_en() {
        assert_eq!(Locale::default(), Locale::EnUs);
    }
}
