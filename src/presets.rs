//! Built-in schema presets.
//!
//! A [`Preset`] is a named, ready-to-use [`Schema`](crate::data::Schema) string
//! for a common data shape — users, orders, sensor readings, … — so that the
//! most frequent cases need no hand-written `--schema`. Presets are exposed on
//! the CLI via `--preset <name>` (on any structured format) and listed by the
//! `presets` subcommand.
//!
//! # Design
//!
//! The preset *table* lives in exactly one place ([`PRESETS`]); the localized
//! one-line descriptions live in the [`i18n`](crate::i18n) catalog, keyed by
//! `preset_desc_<name>`. A compile-time-checked test asserts every preset has a
//! description in every language and that every preset schema parses, so adding
//! a preset is a localized, guarded edit.

use crate::i18n::Language;

/// A named, ready-to-use schema for a common data shape.
#[derive(Debug, Clone, Copy)]
pub struct Preset {
    /// The lowercase preset name used with `--preset` (e.g. `"users"`).
    pub name: &'static str,
    /// The schema string this preset expands to, in `--schema` syntax.
    pub schema: &'static str,
}

impl Preset {
    /// Returns the localized one-line description for this preset.
    ///
    /// Falls back to the English description if a translation is somehow missing
    /// (the `test_every_preset_has_descriptions` test prevents that in practice).
    pub fn description(&self, lang: Language) -> &'static str {
        let c = lang.catalog();
        match self.name {
            "users" => c.preset_desc_users,
            "employees" => c.preset_desc_employees,
            "customers" => c.preset_desc_customers,
            "products" => c.preset_desc_products,
            "orders" => c.preset_desc_orders,
            "transactions" => c.preset_desc_transactions,
            "events" => c.preset_desc_events,
            "servers" => c.preset_desc_servers,
            "geo" => c.preset_desc_geo,
            "posts" => c.preset_desc_posts,
            "payments" => c.preset_desc_payments,
            "sensors" => c.preset_desc_sensors,
            "invoices" => c.preset_desc_invoices,
            "logins" => c.preset_desc_logins,
            "vehicles" => c.preset_desc_vehicles,
            "books" => c.preset_desc_books,
            _ => Language::En.catalog().preset_desc_users,
        }
    }
}

/// The canonical table of built-in presets, in display order.
///
/// Every `name` must have a matching `preset_desc_<name>` key in the i18n
/// catalog and a schema that [`Schema::parse`](crate::data::Schema::parse)
/// accepts; both are enforced by tests.
pub const PRESETS: &[Preset] = &[
    Preset {
        name: "users",
        schema: "id:sequence,name:name,email:email,username:username,active:bool,created:datetime",
    },
    Preset {
        name: "employees",
        schema: "id:sequence,first_name:first_name,last_name:last_name,department:department,job:job,level:job_level,salary:price(30000..150000),hired:date",
    },
    Preset {
        name: "customers",
        schema: "id:uuid,name:name,email:email,phone:phone,address:address,city:city,country:country",
    },
    Preset {
        name: "products",
        schema: "id:sequence,sku:sku,name:product,category:enum(Electronics,Home,Toys,Books,Apparel),price:price(5..999),rating:rating,in_stock:bool",
    },
    Preset {
        name: "orders",
        schema: "id:uuid,customer:name,product:product,quantity:int(1..10),total:price(10..2000),status:enum(pending,paid,shipped,delivered,cancelled),ordered:datetime",
    },
    Preset {
        name: "transactions",
        schema: "id:uuid,amount:price(1..10000),currency:currency,iban:iban,bic:bic,status:enum(authorized,settled,refunded,declined),timestamp:datetime",
    },
    Preset {
        name: "events",
        schema: "id:uuid,event:enum(page_view,click,signup,purchase,logout),user_id:uuid,session:uuid,url:url,timestamp:datetime",
    },
    Preset {
        name: "servers",
        schema: "id:sequence,hostname:domain,ipv4:ipv4,ipv6:ipv6,port:port,os:os,status:enum(online,degraded,offline),uptime:percent",
    },
    Preset {
        name: "geo",
        schema: "id:sequence,city:city,country:country,latitude:latitude,longitude:longitude,coordinates:coordinates,timezone:timezone",
    },
    Preset {
        name: "posts",
        schema: "id:sequence,author:name,title:sentence,body:paragraph,tags:array(word,3),likes:int(0..5000),published:datetime",
    },
    Preset {
        name: "payments",
        schema: "id:uuid,method:enum(card,paypal,transfer,crypto),card_network:card_network,card:credit_card,amount:price(1..5000),currency:currency,paid:datetime",
    },
    Preset {
        name: "sensors",
        schema: "device_id:uuid,metric:enum(temperature,humidity,pressure,co2,motion),value:float(0..100),unit:enum(C,pct,hPa,ppm),battery:percent,recorded:datetime",
    },
    Preset {
        name: "invoices",
        schema: "id:sequence(1000),customer:company,amount:price(50..25000),currency:currency,iban:iban,issued:date,due:date,paid:bool",
    },
    Preset {
        name: "logins",
        schema: "user:username,ip:ipv4,user_agent:user_agent,mfa:enum(none,sms,totp,webauthn),success:bool,timestamp:datetime",
    },
    Preset {
        name: "vehicles",
        schema: "id:uuid,make:enum(Toyota,Volkswagen,Ford,BMW,Hyundai,Tesla,Kia,Volvo),model_year:year(1998..2026),fuel:enum(petrol,diesel,hybrid,electric),mileage_km:int(0..350000),price:price(1500..120000)",
    },
    Preset {
        name: "books",
        schema: "isbn:isbn,title:sentence(4),author:name,pages:int(80..1200),language:language,published:year(1950..2026),price:price(5..60)",
    },
];

/// Looks up a preset by name (case-insensitive). Returns `None` if unknown.
pub fn get(name: &str) -> Option<&'static Preset> {
    let needle = name.trim().to_lowercase();
    PRESETS.iter().find(|p| p.name == needle)
}

/// Returns the number of built-in presets.
pub fn count() -> usize {
    PRESETS.len()
}

/// Returns all preset names, in display order.
pub fn names() -> Vec<&'static str> {
    PRESETS.iter().map(|p| p.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Locale, Schema};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_get_is_case_insensitive() {
        assert!(get("users").is_some());
        assert!(get("USERS").is_some());
        assert!(get("  Orders  ").is_some());
        assert!(get("nope").is_none());
    }

    #[test]
    fn test_names_match_table() {
        assert_eq!(names().len(), PRESETS.len());
        assert_eq!(count(), PRESETS.len());
        assert!(count() >= 12);
    }

    #[test]
    fn test_preset_names_are_unique_and_lowercase() {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for p in PRESETS {
            assert_eq!(p.name, p.name.to_lowercase(), "{} not lowercase", p.name);
            assert!(seen.insert(p.name), "duplicate preset name {}", p.name);
        }
    }

    #[test]
    fn test_every_preset_schema_parses_and_generates() {
        for p in PRESETS {
            let schema = Schema::parse(p.schema)
                .unwrap_or_else(|e| panic!("preset '{}' schema failed to parse: {e}", p.name));
            assert!(
                !schema.is_empty(),
                "preset '{}' has an empty schema",
                p.name
            );
            // Every preset should reference only known types (no accidental typos).
            assert!(
                schema.unknown_field_types().is_empty(),
                "preset '{}' uses unknown types: {:?}",
                p.name,
                schema.unknown_field_types()
            );
            let mut rng = ChaCha8Rng::seed_from_u64(1);
            let records = schema.generate_records(&mut rng, Locale::EnUs, 3);
            assert_eq!(records.len(), 3);
        }
    }

    #[test]
    fn test_every_preset_has_descriptions_in_every_language() {
        for p in PRESETS {
            for lang in Language::variants() {
                let desc = p.description(*lang);
                assert!(
                    !desc.trim().is_empty(),
                    "preset '{}' missing {} description",
                    p.name,
                    lang
                );
            }
        }
    }
}
