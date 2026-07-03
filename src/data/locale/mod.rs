//! Locale support for the faker engine.
//!
//! A [`Locale`] selects the pools of names, places, and company parts used when
//! generating realistic fake data. Locale-agnostic data (UUIDs, IP addresses,
//! colors, …) is unaffected.
//!
//! # Adding a locale
//!
//! 1. Create `src/data/locale/<id>.rs` exposing a single
//!    `pub static <ID>: LocaleData = LocaleData { … };`.
//! 2. Declare the module below (`mod <id>;`).
//! 3. Add one line to the `define_locales!` table.
//!
//! The macro generates the [`Locale`] enum, the `id → variant` parser, the data
//! lookup, and the `all()` catalogue from that single source of truth, so the
//! three steps above are all that is ever required.

mod cs_cz;
mod da_dk;
mod de_de;
mod en_gb;
mod en_us;
mod es_es;
mod fi_fi;
mod fr_fr;
mod it_it;
mod ja_jp;
mod nb_no;
mod nl_nl;
mod pl_pl;
mod pt_br;
mod sv_se;
mod tr_tr;

/// A bundle of static data pools backing a single locale.
///
/// Every locale provides the same fields so that the faker functions in
/// [`crate::data::faker`] stay locale-agnostic — they only ever read from a
/// `LocaleData`, never branch on a specific country.
pub struct LocaleData {
    /// Pool of first / given names.
    pub first_names: &'static [&'static str],
    /// Pool of last / family names.
    pub last_names: &'static [&'static str],
    /// Pool of street names (already including the local street-type word).
    pub streets: &'static [&'static str],
    /// Pool of city names.
    pub cities: &'static [&'static str],
    /// Pool of region / state / province names.
    pub states: &'static [&'static str],
    /// Country name (in the locale's own language) for full addresses.
    pub country: &'static str,
    /// ISO 3166-1 alpha-2 country code.
    pub country_code: &'static str,
    /// Phone country dialing prefix (e.g. `+1`).
    pub phone_prefix: &'static str,
    /// Company name prefixes.
    pub company_prefixes: &'static [&'static str],
    /// Company legal-form suffixes.
    pub company_suffixes: &'static [&'static str],
    /// Whether the house number precedes the street name (`true` for US/UK/FR
    /// style "12 Main St"; `false` for DE/IT/ES style "Hauptstraße 12").
    pub street_number_first: bool,
}

/// Generates the [`Locale`] enum and all its lookup tables from one declaration.
macro_rules! define_locales {
    (
        $( $variant:ident {
            id: $id:literal,
            aliases: [ $( $alias:literal ),* $(,)? ],
            label: $label:literal,
            data: $data:path $(,)?
        } ),+ $(,)?
    ) => {
        /// Supported locales for fake-data generation.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Locale {
            $(
                #[doc = $label]
                $variant,
            )+
        }

        impl Locale {
            /// Returns the static data pools for this locale.
            pub fn data(&self) -> &'static LocaleData {
                match self {
                    $( Locale::$variant => &$data, )+
                }
            }

            /// Returns the canonical lowercase identifier (e.g. `"en_us"`).
            pub fn as_str(&self) -> &'static str {
                match self {
                    $( Locale::$variant => $id, )+
                }
            }

            /// Returns a human-readable label (e.g. `"English (United States)"`).
            pub fn label(&self) -> &'static str {
                match self {
                    $( Locale::$variant => $label, )+
                }
            }

            /// Returns every supported variant, in declaration order.
            pub fn variants() -> &'static [Locale] {
                &[ $( Locale::$variant ),+ ]
            }

            /// Returns all canonical locale identifiers, for help text and the
            /// `list` command.
            pub fn all() -> &'static [&'static str] {
                &[ $( $id ),+ ]
            }
        }

        impl ::std::str::FromStr for Locale {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.trim().to_lowercase().replace('-', "_").as_str() {
                    $( $id $( | $alias )* => Ok(Locale::$variant), )+
                    other => Err(format!(
                        "Unknown locale: '{other}'. Valid: {}",
                        Locale::all().join(", ")
                    )),
                }
            }
        }
    };
}

define_locales! {
    EnUs { id: "en_us", aliases: ["en", "us", "english"], label: "English (United States)", data: en_us::EN_US },
    EnGb { id: "en_gb", aliases: ["gb", "uk", "en_uk"],    label: "English (United Kingdom)", data: en_gb::EN_GB },
    DeDe { id: "de_de", aliases: ["de", "german"],         label: "German (Germany)",         data: de_de::DE_DE },
    FrFr { id: "fr_fr", aliases: ["fr", "french"],         label: "French (France)",          data: fr_fr::FR_FR },
    EsEs { id: "es_es", aliases: ["es", "spanish"],        label: "Spanish (Spain)",          data: es_es::ES_ES },
    ItIt { id: "it_it", aliases: ["it", "italian"],        label: "Italian (Italy)",          data: it_it::IT_IT },
    PtBr { id: "pt_br", aliases: ["pt", "br", "brazil"],   label: "Portuguese (Brazil)",      data: pt_br::PT_BR },
    NlNl { id: "nl_nl", aliases: ["nl", "dutch"],          label: "Dutch (Netherlands)",      data: nl_nl::NL_NL },
    PlPl { id: "pl_pl", aliases: ["pl", "polish"],         label: "Polish (Poland)",          data: pl_pl::PL_PL },
    SvSe { id: "sv_se", aliases: ["sv", "se", "swedish"],  label: "Swedish (Sweden)",         data: sv_se::SV_SE },
    DaDk { id: "da_dk", aliases: ["da", "dk", "danish"],   label: "Danish (Denmark)",         data: da_dk::DA_DK },
    NbNo { id: "nb_no", aliases: ["nb", "no", "norwegian"], label: "Norwegian Bokmål (Norway)", data: nb_no::NB_NO },
    FiFi { id: "fi_fi", aliases: ["fi", "finnish"],        label: "Finnish (Finland)",        data: fi_fi::FI_FI },
    CsCz { id: "cs_cz", aliases: ["cs", "cz", "czech"],    label: "Czech (Czechia)",          data: cs_cz::CS_CZ },
    TrTr { id: "tr_tr", aliases: ["tr", "turkish"],        label: "Turkish (Türkiye)",        data: tr_tr::TR_TR },
    JaJp { id: "ja_jp", aliases: ["ja", "jp", "japanese"], label: "Japanese (Japan)",         data: ja_jp::JA_JP },
}

impl Default for Locale {
    /// United States English is the default locale.
    fn default() -> Self {
        Locale::EnUs
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_parse() {
        assert_eq!("en".parse::<Locale>().unwrap(), Locale::EnUs);
        assert_eq!("de_DE".parse::<Locale>().unwrap(), Locale::DeDe);
        assert_eq!("DE-DE".parse::<Locale>().unwrap(), Locale::DeDe);
        assert_eq!("fr".parse::<Locale>().unwrap(), Locale::FrFr);
        assert_eq!("pt_br".parse::<Locale>().unwrap(), Locale::PtBr);
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
    fn test_variants_match_all() {
        assert_eq!(Locale::variants().len(), Locale::all().len());
        for (loc, id) in Locale::variants().iter().zip(Locale::all()) {
            assert_eq!(&loc.as_str(), id);
        }
    }

    #[test]
    fn test_every_locale_has_a_label() {
        for loc in Locale::variants() {
            assert!(!loc.label().is_empty());
            assert!(loc.label().contains('('));
        }
    }

    #[test]
    fn test_locale_data_non_empty() {
        for loc in Locale::variants() {
            let d = loc.data();
            assert!(!d.first_names.is_empty(), "{loc} first_names empty");
            assert!(!d.last_names.is_empty(), "{loc} last_names empty");
            assert!(!d.cities.is_empty(), "{loc} cities empty");
            assert!(!d.streets.is_empty(), "{loc} streets empty");
            assert!(!d.states.is_empty(), "{loc} states empty");
            assert!(
                !d.company_prefixes.is_empty(),
                "{loc} company_prefixes empty"
            );
            assert!(
                !d.company_suffixes.is_empty(),
                "{loc} company_suffixes empty"
            );
            assert!(!d.country.is_empty(), "{loc} country empty");
            assert_eq!(d.country_code.len(), 2, "{loc} country_code not alpha-2");
            assert!(d.phone_prefix.starts_with('+'), "{loc} phone_prefix");
        }
    }

    #[test]
    fn test_default_locale_is_en() {
        assert_eq!(Locale::default(), Locale::EnUs);
    }

    #[test]
    fn test_locale_pools_have_no_duplicates() {
        use std::collections::HashSet;
        let assert_unique = |loc: &Locale, field: &str, pool: &[&str]| {
            let unique: HashSet<&&str> = pool.iter().collect();
            assert_eq!(
                unique.len(),
                pool.len(),
                "{loc} {field} contains duplicates"
            );
        };
        for loc in Locale::variants() {
            let d = loc.data();
            assert_unique(loc, "first_names", d.first_names);
            assert_unique(loc, "last_names", d.last_names);
            assert_unique(loc, "streets", d.streets);
            assert_unique(loc, "cities", d.cities);
            assert_unique(loc, "states", d.states);
            assert_unique(loc, "company_prefixes", d.company_prefixes);
            assert_unique(loc, "company_suffixes", d.company_suffixes);
        }
    }
}
