//! Typed schema engine.
//!
//! Parses a compact schema string into a list of [`FieldSpec`]s and produces
//! typed [`Record`]s of [`FieldValue`]s. Producing *typed* values (rather than
//! re-parsing strings) lets every structured format — JSON, YAML, TOML, SQL,
//! CSV, XML — emit correctly-typed output from a single source of truth.
//!
//! # Schema syntax
//!
//! A schema is a comma-separated list of `name:typespec` fields. A `typespec`
//! is a base type with optional arguments and an optional nullability suffix:
//!
//! ```text
//! id:sequence                 # 1,2,3,… per row
//! id:sequence(100)            # start at 100
//! age:int(18..65)             # bounded integer
//! score:float(0..1)           # bounded float
//! status:enum(new,active,gone) # pick one
//! country:const(DE)           # constant
//! tags:array(word,3)          # array of 3 words
//! phone:phone?                # ~10% chance of null
//! note:sentence?0.5           # 50% chance of null
//! ```
//!
//! Unknown base types fall back to a generic word string so a typo never
//! aborts generation.
use crate::data::{Locale, faker, lorem};
use rand::{Rng, RngExt};

/// A single generated, typed value.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    /// SQL `NULL` / JSON `null` / absent.
    Null,
    /// Boolean.
    Bool(bool),
    /// 64-bit signed integer.
    Int(i64),
    /// 64-bit float.
    Float(f64),
    /// UTF-8 string.
    Str(String),
    /// Homogeneous list of values.
    Array(Vec<FieldValue>),
}

impl FieldValue {
    /// Returns `true` if this value is [`FieldValue::Null`].
    pub fn is_null(&self) -> bool {
        matches!(self, FieldValue::Null)
    }

    /// Converts to a `serde_json::Value` for JSON/JSONL/YAML output.
    pub fn to_json(&self) -> serde_json::Value {
        use serde_json::Value;
        match self {
            FieldValue::Null => Value::Null,
            FieldValue::Bool(b) => Value::Bool(*b),
            FieldValue::Int(i) => Value::from(*i),
            FieldValue::Float(f) => Value::from(*f),
            FieldValue::Str(s) => Value::String(s.clone()),
            FieldValue::Array(items) => Value::Array(items.iter().map(|v| v.to_json()).collect()),
        }
    }

    /// Renders as a flat string for delimited formats (CSV/TSV) and XML text.
    /// `Null` becomes an empty string; arrays join with `; `.
    pub fn to_flat_string(&self) -> String {
        match self {
            FieldValue::Null => String::new(),
            FieldValue::Bool(b) => b.to_string(),
            FieldValue::Int(i) => i.to_string(),
            FieldValue::Float(f) => f.to_string(),
            FieldValue::Str(s) => s.clone(),
            FieldValue::Array(items) => items
                .iter()
                .map(|v| v.to_flat_string())
                .collect::<Vec<_>>()
                .join("; "),
        }
    }

    /// Renders as a SQL literal (`NULL`, numbers bare, strings single-quoted
    /// with `'` doubled).
    pub fn to_sql_literal(&self) -> String {
        match self {
            FieldValue::Null => "NULL".to_string(),
            FieldValue::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
            FieldValue::Int(i) => i.to_string(),
            FieldValue::Float(f) => f.to_string(),
            FieldValue::Str(s) => format!("'{}'", s.replace('\'', "''")),
            FieldValue::Array(items) => {
                let joined = items
                    .iter()
                    .map(|v| v.to_flat_string())
                    .collect::<Vec<_>>()
                    .join(",");
                format!("'{}'", joined.replace('\'', "''"))
            }
        }
    }
}

/// A generated record: an ordered list of `(field_name, value)` pairs.
pub type Record = Vec<(String, FieldValue)>;

/// Parsed arguments attached to a scalar field type.
#[derive(Debug, Clone, PartialEq)]
enum Args {
    None,
    IntRange(i64, i64),
    FloatRange(f64, f64),
    Count(usize),
}

/// How a single field is produced.
#[derive(Debug, Clone, PartialEq)]
enum FieldKind {
    /// A named base type with optional arguments.
    Scalar { base: String, args: Args },
    /// One value chosen at random from a fixed set.
    Enum(Vec<String>),
    /// A constant literal value.
    Const(String),
    /// A per-row incrementing integer starting at the given value.
    Sequence(i64),
    /// An array of `count` (or random) elements of a kind.
    Array {
        elem: Box<FieldKind>,
        count: Option<usize>,
    },
}

/// A fully parsed field: a name, how to generate it, and a null probability.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldSpec {
    /// Output field/column name.
    pub name: String,
    kind: FieldKind,
    null_prob: f64,
}

impl FieldSpec {
    /// Returns a representative SQL column type for this field.
    pub fn sql_type(&self) -> &'static str {
        sql_type_for_kind(&self.kind)
    }
}

fn sql_type_for_kind(kind: &FieldKind) -> &'static str {
    match kind {
        FieldKind::Sequence(_) => "INTEGER",
        FieldKind::Const(_) | FieldKind::Enum(_) | FieldKind::Array { .. } => "TEXT",
        FieldKind::Scalar { base, .. } => match base.as_str() {
            "int" | "integer" | "number" | "year" | "age" | "timestamp" | "unix" | "port"
            | "http_status" | "status_code" | "statuscode" => "INTEGER",
            "float" | "decimal" | "double" | "price" | "amount" | "money" | "latitude" | "lat"
            | "longitude" | "lng" | "lon" | "percent" | "percentage" | "rating" | "stars" => "REAL",
            "bool" | "boolean" => "BOOLEAN",
            _ => "TEXT",
        },
    }
}

/// A parsed schema: an ordered list of field specifications.
#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    /// The parsed field specifications, in declaration order.
    pub fields: Vec<FieldSpec>,
}

impl Schema {
    /// Parses a schema string. Returns an error only for structurally invalid
    /// input (e.g. an empty schema or a malformed range); unknown type names
    /// are tolerated and produce generic strings.
    pub fn parse(input: &str) -> Result<Schema, String> {
        let mut fields = Vec::new();
        for raw in split_top_level(input) {
            let raw = raw.trim();
            if raw.is_empty() {
                continue;
            }
            let (name, typespec) = match raw.split_once(':') {
                Some((n, t)) => (n.trim().to_string(), t.trim()),
                None => (raw.to_string(), "string"),
            };
            if name.is_empty() {
                return Err(format!("Field with empty name in schema: '{raw}'"));
            }
            let (kind, null_prob) = parse_typespec(typespec)?;
            fields.push(FieldSpec {
                name,
                kind,
                null_prob,
            });
        }
        Ok(Schema { fields })
    }

    /// Returns `true` if the schema has no fields.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Returns the ordered field names.
    pub fn field_names(&self) -> Vec<&str> {
        self.fields.iter().map(|f| f.name.as_str()).collect()
    }

    /// Generates a single record. `index` drives the `sequence` field kind.
    pub fn generate_record<R: Rng>(&self, rng: &mut R, locale: Locale, index: usize) -> Record {
        self.fields
            .iter()
            .map(|f| {
                let value = if f.null_prob > 0.0 && rng.random_bool(f.null_prob.clamp(0.0, 1.0)) {
                    FieldValue::Null
                } else {
                    eval_kind(&f.kind, rng, locale, index)
                };
                (f.name.clone(), value)
            })
            .collect()
    }

    /// Generates `rows` records.
    pub fn generate_records<R: Rng>(
        &self,
        rng: &mut R,
        locale: Locale,
        rows: usize,
    ) -> Vec<Record> {
        (0..rows)
            .map(|i| self.generate_record(rng, locale, i))
            .collect()
    }

    /// Returns the base names of any unrecognized scalar field types in this
    /// schema (recursing into arrays), in declaration order.
    ///
    /// Unknown types still generate output — a generic word — so this never
    /// blocks generation; the CLI uses it only to surface a "did you mean …"
    /// hint, helping users catch typos like `emial` for `email`.
    pub fn unknown_field_types(&self) -> Vec<String> {
        let mut out = Vec::new();
        for f in &self.fields {
            collect_unknown(&f.kind, &mut out);
        }
        out
    }
}

/// Recursively collects unrecognized scalar base names from a [`FieldKind`].
fn collect_unknown(kind: &FieldKind, out: &mut Vec<String>) {
    match kind {
        FieldKind::Scalar { base, .. } if !is_known_type(base) => out.push(base.clone()),
        FieldKind::Array { elem, .. } => collect_unknown(elem, out),
        _ => {}
    }
}

/// Returns `true` if `base` is a scalar type name the engine recognizes.
pub fn is_known_type(base: &str) -> bool {
    let b = base.trim().to_lowercase();
    KNOWN_TYPE_NAMES.contains(&b.as_str())
}

/// Suggests the closest known type name to `unknown`, if one is near enough to
/// be a plausible typo (Levenshtein distance within a small, length-aware
/// threshold). Returns `None` when nothing is close.
pub fn suggest_type(unknown: &str) -> Option<&'static str> {
    let u = unknown.trim().to_lowercase();
    // A blank or already-valid type needs no suggestion.
    if u.is_empty() || is_known_type(&u) {
        return None;
    }
    KNOWN_TYPE_NAMES
        .iter()
        .copied()
        .map(|t| (t, levenshtein(&u, t)))
        .filter(|(t, d)| {
            // Length-aware tolerance: enough to catch transpositions and a
            // double typo on a word, without suggesting unrelated names.
            let n = u.len().max(t.len());
            let threshold = if n <= 2 {
                1
            } else if n <= 9 {
                2
            } else {
                3
            };
            *d > 0 && *d <= threshold
        })
        // Prefer the smallest edit distance; break ties toward the candidate
        // closest in length to the input (a better proxy for "intended word").
        .min_by_key(|(t, d)| {
            let len_diff = (t.len() as i64 - u.len() as i64).unsigned_abs();
            (*d, len_diff, t.len())
        })
        .map(|(t, _)| t)
}

/// Computes the Levenshtein edit distance between two strings.
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];
    for (i, &ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

/// Splits a schema on top-level commas, ignoring commas inside parentheses.
fn split_top_level(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    for ch in input.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                out.push(std::mem::take(&mut current));
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        out.push(current);
    }
    out
}

/// Parses a `typespec` into a [`FieldKind`] and a null probability.
fn parse_typespec(spec: &str) -> Result<(FieldKind, f64), String> {
    let spec = spec.trim();
    // Extract a trailing nullability marker: `?` or `?<prob>`, which must come
    // after any closing parenthesis.
    let (body, null_prob) = match spec.rfind('?') {
        Some(pos) if pos >= spec.rfind(')').map(|p| p + 1).unwrap_or(0) => {
            let rest = spec[pos + 1..].trim();
            let prob = if rest.is_empty() {
                0.1
            } else {
                let p = rest
                    .parse::<f64>()
                    .map_err(|_| format!("Invalid null probability '{rest}' in '{spec}'"))?;
                if !(0.0..=1.0).contains(&p) {
                    return Err(format!(
                        "Null probability must be between 0.0 and 1.0, got '{rest}' in '{spec}'"
                    ));
                }
                p
            };
            (spec[..pos].trim(), prob)
        }
        _ => (spec, 0.0),
    };

    // Split base name from a parenthesised argument string.
    let (base, argstr) = match body.find('(') {
        Some(open) => {
            let close = body
                .rfind(')')
                .ok_or_else(|| format!("Unclosed '(' in typespec '{body}'"))?;
            if close < open {
                return Err(format!("Mismatched parentheses in typespec '{body}'"));
            }
            (body[..open].trim(), Some(body[open + 1..close].trim()))
        }
        None => (body, None),
    };

    let base_lc = base.to_lowercase();
    let kind = match base_lc.as_str() {
        "enum" | "choice" | "oneof" => {
            let opts: Vec<String> = argstr
                .map(|a| {
                    split_top_level(a)
                        .into_iter()
                        .map(|s| s.trim().to_string())
                        .collect()
                })
                .unwrap_or_default();
            if opts.is_empty() {
                return Err(format!("enum requires at least one option: '{spec}'"));
            }
            FieldKind::Enum(opts)
        }
        "const" | "constant" | "fixed" | "literal" => {
            FieldKind::Const(argstr.unwrap_or("").to_string())
        }
        "sequence" | "seq" | "autoincrement" | "serial" => {
            let start = match argstr {
                Some(a) if !a.is_empty() => a
                    .parse::<i64>()
                    .map_err(|_| format!("sequence start must be an integer: '{spec}'"))?,
                _ => 1,
            };
            FieldKind::Sequence(start)
        }
        "array" | "list" => {
            let inner = argstr.unwrap_or("string");
            let parts = split_top_level(inner);
            let elem_spec = parts.first().map(|s| s.trim()).unwrap_or("string");
            let count = match parts.get(1) {
                Some(c) => Some(
                    c.trim()
                        .parse::<usize>()
                        .map_err(|_| format!("array count must be an integer: '{spec}'"))?,
                ),
                None => None,
            };
            let (elem_kind, _) = parse_typespec(elem_spec)?;
            FieldKind::Array {
                elem: Box::new(elem_kind),
                count,
            }
        }
        _ => FieldKind::Scalar {
            base: base_lc,
            args: parse_args(argstr)?,
        },
    };

    Ok((kind, null_prob))
}

/// Parses scalar arguments: a numeric range `a..b` or a single count.
fn parse_args(argstr: Option<&str>) -> Result<Args, String> {
    let Some(a) = argstr.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(Args::None);
    };
    if let Some((lo, hi)) = a.split_once("..") {
        let lo = lo.trim();
        let hi = hi.trim();
        let is_float = lo.contains('.') || hi.contains('.');
        if is_float {
            let lo: f64 = lo
                .parse()
                .map_err(|_| format!("Invalid range bound '{lo}'"))?;
            let hi: f64 = hi
                .parse()
                .map_err(|_| format!("Invalid range bound '{hi}'"))?;
            Ok(Args::FloatRange(lo, hi))
        } else {
            let lo: i64 = lo
                .parse()
                .map_err(|_| format!("Invalid range bound '{lo}'"))?;
            let hi: i64 = hi
                .parse()
                .map_err(|_| format!("Invalid range bound '{hi}'"))?;
            Ok(Args::IntRange(lo, hi))
        }
    } else if let Ok(n) = a.parse::<usize>() {
        Ok(Args::Count(n))
    } else {
        // A non-numeric argument on a scalar is ignored gracefully.
        Ok(Args::None)
    }
}

/// Evaluates a [`FieldKind`] into a [`FieldValue`].
fn eval_kind<R: Rng>(kind: &FieldKind, rng: &mut R, locale: Locale, index: usize) -> FieldValue {
    match kind {
        FieldKind::Const(v) => FieldValue::Str(v.clone()),
        FieldKind::Enum(opts) => FieldValue::Str(opts[rng.random_range(0..opts.len())].clone()),
        FieldKind::Sequence(start) => FieldValue::Int(start.wrapping_add(index as i64)),
        FieldKind::Array { elem, count } => {
            let n = count.unwrap_or_else(|| rng.random_range(1..=5));
            FieldValue::Array((0..n).map(|i| eval_kind(elem, rng, locale, i)).collect())
        }
        FieldKind::Scalar { base, args } => eval_scalar(base, args, rng, locale),
    }
}

/// Evaluates a named scalar base type with its arguments.
fn eval_scalar<R: Rng>(base: &str, args: &Args, rng: &mut R, locale: Locale) -> FieldValue {
    use FieldValue as V;
    let int_range = |def_lo: i64, def_hi: i64| match args {
        Args::IntRange(lo, hi) => (*lo, *hi),
        Args::FloatRange(lo, hi) => (*lo as i64, *hi as i64),
        _ => (def_lo, def_hi),
    };
    let float_range = |def_lo: f64, def_hi: f64| match args {
        Args::FloatRange(lo, hi) => (*lo, *hi),
        Args::IntRange(lo, hi) => (*lo as f64, *hi as f64),
        _ => (def_lo, def_hi),
    };
    let count = |def: usize| match args {
        Args::Count(n) => *n,
        _ => def,
    };

    match base {
        "int" | "integer" | "number" => {
            let (lo, hi) = int_range(0, 10_000);
            V::Int(faker::integer(rng, lo, hi))
        }
        "age" => {
            let (lo, hi) = int_range(18, 90);
            V::Int(faker::integer(rng, lo, hi))
        }
        "year" => {
            let (lo, hi) = int_range(1970, 2025);
            V::Int(faker::integer(rng, lo, hi))
        }
        "float" | "decimal" | "double" => {
            let (lo, hi) = float_range(0.0, 10_000.0);
            V::Float(faker::float(rng, lo, hi))
        }
        "price" | "amount" | "money" => {
            let (lo, hi) = float_range(0.0, 1_000.0);
            V::Float(faker::price(rng, lo, hi))
        }
        "latitude" | "lat" => V::Float(faker::latitude(rng)),
        "longitude" | "lng" | "lon" => V::Float(faker::longitude(rng)),
        "bool" | "boolean" => V::Bool(faker::boolean(rng)),
        "timestamp" | "unix" => V::Int(faker::unix_timestamp(rng)),

        "string" | "name" | "full_name" | "fullname" => V::Str(faker::full_name(rng, locale)),
        "first_name" | "firstname" | "given_name" => V::Str(faker::first_name(rng, locale).into()),
        "last_name" | "lastname" | "surname" | "family_name" => {
            V::Str(faker::last_name(rng, locale).into())
        }
        "username" | "user" | "login" => V::Str(faker::username(rng, locale)),
        "email" | "mail" => V::Str(faker::email(rng, locale)),
        "password" | "pass" => V::Str(faker::password(rng)),
        "phone" | "telephone" | "tel" => V::Str(faker::phone(rng, locale)),
        "street" => V::Str(faker::street(rng, locale)),
        "address" => V::Str(faker::address(rng, locale)),
        "city" => V::Str(faker::city(rng, locale).into()),
        "state" | "region" | "province" => V::Str(faker::state(rng, locale).into()),
        "zipcode" | "zip" | "postcode" | "postal_code" => V::Str(faker::zipcode(rng, locale)),
        "country" => V::Str(faker::country(locale).into()),
        "country_code" => V::Str(faker::country_code(locale).into()),
        "company" | "organization" | "org" => V::Str(faker::company(rng, locale)),
        "job" | "job_title" | "title" | "position" => V::Str(faker::job_title(rng).into()),
        "department" | "dept" => V::Str(faker::department(rng).into()),
        "product" => V::Str(faker::product(rng)),
        "sku" => V::Str(faker::sku(rng)),
        "currency" | "currency_code" => V::Str(faker::currency_code(rng).into()),
        "iban" => V::Str(faker::iban(rng, locale)),
        "credit_card" | "creditcard" | "cc" => V::Str(faker::credit_card(rng)),
        "isbn" => V::Str(faker::isbn(rng)),
        "url" | "uri" | "link" => V::Str(faker::url(rng, locale)),
        "domain" | "hostname" => V::Str(faker::domain(rng)),
        "slug" => V::Str(faker::slug(rng)),
        "ipv4" | "ip" => V::Str(faker::ipv4(rng)),
        "ipv6" => V::Str(faker::ipv6(rng)),
        "mac" | "mac_address" => V::Str(faker::mac_address(rng)),
        "uuid" | "guid" => V::Str(faker::uuid(rng)),
        "user_agent" | "useragent" => V::Str(faker::user_agent(rng).into()),
        "color" | "colour" => V::Str(faker::color_name(rng).into()),
        "hex_color" | "hexcolor" => V::Str(faker::hex_color(rng)),
        "language" | "lang" => V::Str(faker::language(rng).into()),
        "timezone" | "tz" => V::Str(faker::timezone(rng).into()),
        "emoji" => V::Str(faker::emoji(rng).into()),
        "gender" | "sex" => V::Str(faker::gender(rng).into()),

        "date" => V::Str(faker::date(rng)),
        "time" => V::Str(faker::time(rng)),
        "datetime" => V::Str(faker::datetime(rng)),
        "weekday" | "day" => V::Str(faker::weekday(rng).into()),
        "month" => V::Str(faker::month(rng).into()),

        "percent" | "percentage" => V::Float(faker::percent(rng)),
        "rating" | "stars" => V::Float(faker::rating(rng)),
        "port" => V::Int(faker::port(rng)),
        "ssn" => V::Str(faker::ssn(rng)),
        "currency_symbol" | "currency_sign" => V::Str(faker::currency_symbol(rng).into()),
        "mime_type" | "mime" | "content_type" => V::Str(faker::mime_type(rng).into()),
        "filename" | "file" | "file_name" => V::Str(faker::filename(rng)),
        "semver" | "version" => V::Str(faker::semver(rng)),
        "hashtag" => V::Str(faker::hashtag(rng)),
        "base64" | "token" => V::Str(faker::base64_token(rng)),
        "hex" => V::Str(faker::hex_token(rng, count(16))),

        "bic" | "swift" => V::Str(faker::bic(rng, locale)),
        "ean" | "ean13" | "barcode" => V::Str(faker::ean13(rng)),
        "imei" => V::Str(faker::imei(rng)),
        "card_network" | "card_type" | "cc_type" => V::Str(faker::card_network(rng).into()),
        "company_email" | "work_email" | "business_email" => {
            V::Str(faker::company_email(rng, locale))
        }
        "job_level" | "seniority" => V::Str(faker::job_level(rng).into()),
        "http_method" | "method" | "verb" => V::Str(faker::http_method(rng).into()),
        "http_status" | "status_code" | "statuscode" => V::Int(faker::http_status(rng)),
        "os" | "operating_system" => V::Str(faker::os_name(rng).into()),
        "browser" => V::Str(faker::browser(rng).into()),
        "device" | "device_type" => V::Str(faker::device(rng).into()),
        "file_size" | "filesize" => V::Str(faker::file_size(rng)),
        "coordinates" | "coords" | "latlng" | "geo" => V::Str(faker::coordinates(rng)),
        "airport" | "iata" => V::Str(faker::airport(rng).into()),
        "flight" | "flight_number" => V::Str(faker::flight(rng)),
        "vin" => V::Str(faker::vin(rng)),

        "word" => V::Str(lorem::word(rng).into()),
        "words" => V::Str(lorem::words(rng, count(3))),
        "sentence" => V::Str(lorem::sentence(rng, count(0))),
        "paragraph" | "text" => V::Str(lorem::paragraph(rng)),

        // Unknown type: degrade gracefully to a generic word.
        _ => V::Str(lorem::word(rng).into()),
    }
}

/// The catalogue of schema field types, grouped by theme for the `list` and
/// `presets` commands. This is the single source of truth for the *displayed*
/// type reference; the category labels are intentionally technical and kept in
/// English across interface languages.
pub const FIELD_TYPE_GROUPS: &[(&str, &[&str])] = &[
    (
        "Numeric",
        &[
            "int(min..max)",
            "float(min..max)",
            "price(min..max)",
            "age",
            "year",
            "latitude",
            "longitude",
            "percent",
            "rating",
            "port",
            "timestamp",
            "http_status",
        ],
    ),
    ("Boolean", &["bool"]),
    (
        "People",
        &[
            "name",
            "first_name",
            "last_name",
            "username",
            "gender",
            "password",
            "ssn",
            "job_level",
        ],
    ),
    (
        "Contact",
        &[
            "email",
            "company_email",
            "phone",
            "address",
            "street",
            "city",
            "state",
            "zipcode",
            "country",
            "country_code",
            "coordinates",
        ],
    ),
    (
        "Business",
        &[
            "company",
            "job",
            "department",
            "product",
            "sku",
            "currency",
            "currency_symbol",
            "iban",
            "bic",
            "credit_card",
            "card_network",
            "isbn",
            "ean",
        ],
    ),
    (
        "Internet",
        &[
            "url",
            "domain",
            "slug",
            "ipv4",
            "ipv6",
            "mac",
            "uuid",
            "user_agent",
            "http_method",
            "mime_type",
            "filename",
            "semver",
            "os",
            "browser",
            "device",
            "imei",
        ],
    ),
    (
        "Misc",
        &[
            "color",
            "hex_color",
            "language",
            "timezone",
            "emoji",
            "hashtag",
            "base64",
            "hex(n)",
            "file_size",
            "airport",
            "flight",
            "vin",
        ],
    ),
    (
        "Temporal",
        &["date", "time", "datetime", "weekday", "month"],
    ),
    ("Text", &["word", "words(n)", "sentence", "paragraph"]),
    (
        "Modifiers",
        &[
            "enum(a,b,c)",
            "const(value)",
            "sequence(start)",
            "array(type,n)",
            "type? / type?p (nullable)",
        ],
    ),
];

/// Every base type name and alias the schema engine recognizes, used by
/// [`is_known_type`] and [`suggest_type`].
///
/// Keep this in sync with the match arms in `eval_scalar` and the modifier
/// keywords in `parse_typespec`; the `test_known_types_cover_catalogue` test
/// guards against the catalogue drifting ahead of this list.
pub const KNOWN_TYPE_NAMES: &[&str] = &[
    // Numeric
    "int",
    "integer",
    "number",
    "age",
    "year",
    "float",
    "decimal",
    "double",
    "price",
    "amount",
    "money",
    "latitude",
    "lat",
    "longitude",
    "lng",
    "lon",
    "bool",
    "boolean",
    "timestamp",
    "unix",
    "percent",
    "percentage",
    "rating",
    "stars",
    "port",
    "http_status",
    "status_code",
    "statuscode",
    // People
    "string",
    "name",
    "full_name",
    "fullname",
    "first_name",
    "firstname",
    "given_name",
    "last_name",
    "lastname",
    "surname",
    "family_name",
    "username",
    "user",
    "login",
    "password",
    "pass",
    "gender",
    "sex",
    "ssn",
    "job_level",
    "seniority",
    // Contact
    "email",
    "mail",
    "company_email",
    "work_email",
    "business_email",
    "phone",
    "telephone",
    "tel",
    "street",
    "address",
    "city",
    "state",
    "region",
    "province",
    "zipcode",
    "zip",
    "postcode",
    "postal_code",
    "country",
    "country_code",
    "coordinates",
    "coords",
    "latlng",
    "geo",
    // Business
    "company",
    "organization",
    "org",
    "job",
    "job_title",
    "title",
    "position",
    "department",
    "dept",
    "product",
    "sku",
    "currency",
    "currency_code",
    "currency_symbol",
    "currency_sign",
    "iban",
    "bic",
    "swift",
    "credit_card",
    "creditcard",
    "cc",
    "card_network",
    "card_type",
    "cc_type",
    "isbn",
    "ean",
    "ean13",
    "barcode",
    "imei",
    // Internet
    "url",
    "uri",
    "link",
    "domain",
    "hostname",
    "slug",
    "ipv4",
    "ip",
    "ipv6",
    "mac",
    "mac_address",
    "uuid",
    "guid",
    "user_agent",
    "useragent",
    "http_method",
    "method",
    "verb",
    "mime_type",
    "mime",
    "content_type",
    "filename",
    "file",
    "file_name",
    "semver",
    "version",
    "os",
    "operating_system",
    "browser",
    "device",
    "device_type",
    // Misc
    "color",
    "colour",
    "hex_color",
    "hexcolor",
    "language",
    "lang",
    "timezone",
    "tz",
    "emoji",
    "hashtag",
    "base64",
    "token",
    "hex",
    "file_size",
    "filesize",
    "airport",
    "iata",
    "flight",
    "flight_number",
    "vin",
    // Temporal
    "date",
    "time",
    "datetime",
    "weekday",
    "day",
    "month",
    // Text
    "word",
    "words",
    "sentence",
    "paragraph",
    "text",
    // Modifiers
    "enum",
    "choice",
    "oneof",
    "const",
    "constant",
    "fixed",
    "literal",
    "sequence",
    "seq",
    "autoincrement",
    "serial",
    "array",
    "list",
];

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(7)
    }

    #[test]
    fn test_parse_basic_fields() {
        let s = Schema::parse("id:int,name:name,email:email").unwrap();
        assert_eq!(s.fields.len(), 3);
        assert_eq!(s.field_names(), vec!["id", "name", "email"]);
    }

    #[test]
    fn test_parse_without_colon_defaults_to_string() {
        let s = Schema::parse("foo").unwrap();
        assert_eq!(s.fields.len(), 1);
        let rec = s.generate_record(&mut rng(), Locale::EnUs, 0);
        assert!(matches!(rec[0].1, FieldValue::Str(_)));
    }

    #[test]
    fn test_int_range() {
        let s = Schema::parse("n:int(5..9)").unwrap();
        let mut r = rng();
        for _ in 0..100 {
            let rec = s.generate_record(&mut r, Locale::EnUs, 0);
            match rec[0].1 {
                FieldValue::Int(v) => assert!((5..=9).contains(&v)),
                _ => panic!("expected int"),
            }
        }
    }

    #[test]
    fn test_float_range() {
        let s = Schema::parse("x:float(0.0..1.0)").unwrap();
        let mut r = rng();
        for _ in 0..100 {
            let rec = s.generate_record(&mut r, Locale::EnUs, 0);
            match rec[0].1 {
                FieldValue::Float(v) => assert!((0.0..=1.0).contains(&v)),
                _ => panic!("expected float"),
            }
        }
    }

    #[test]
    fn test_enum_picks_from_set() {
        let s = Schema::parse("status:enum(new,active,closed)").unwrap();
        let mut r = rng();
        for _ in 0..50 {
            let rec = s.generate_record(&mut r, Locale::EnUs, 0);
            match &rec[0].1 {
                FieldValue::Str(v) => assert!(["new", "active", "closed"].contains(&v.as_str())),
                _ => panic!("expected str"),
            }
        }
    }

    #[test]
    fn test_const() {
        let s = Schema::parse("kind:const(widget)").unwrap();
        let rec = s.generate_record(&mut rng(), Locale::EnUs, 0);
        assert_eq!(rec[0].1, FieldValue::Str("widget".into()));
    }

    #[test]
    fn test_sequence() {
        let s = Schema::parse("id:sequence(100)").unwrap();
        let mut r = rng();
        let r0 = s.generate_record(&mut r, Locale::EnUs, 0);
        let r5 = s.generate_record(&mut r, Locale::EnUs, 5);
        assert_eq!(r0[0].1, FieldValue::Int(100));
        assert_eq!(r5[0].1, FieldValue::Int(105));
    }

    #[test]
    fn test_array() {
        let s = Schema::parse("tags:array(word,3)").unwrap();
        let rec = s.generate_record(&mut rng(), Locale::EnUs, 0);
        match &rec[0].1 {
            FieldValue::Array(items) => assert_eq!(items.len(), 3),
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn test_nullable_always_null() {
        let s = Schema::parse("x:int?1.0").unwrap();
        let rec = s.generate_record(&mut rng(), Locale::EnUs, 0);
        assert!(rec[0].1.is_null());
    }

    #[test]
    fn test_nullable_never_null() {
        let s = Schema::parse("x:int?0.0").unwrap();
        let rec = s.generate_record(&mut rng(), Locale::EnUs, 0);
        assert!(!rec[0].1.is_null());
    }

    #[test]
    fn test_split_respects_parens() {
        let parts = split_top_level("a:enum(x,y,z),b:int(1..2)");
        assert_eq!(parts.len(), 2);
    }

    #[test]
    fn test_invalid_range_errors() {
        assert!(Schema::parse("n:int(a..b)").is_err());
    }

    #[test]
    fn test_new_scalar_types() {
        let s = Schema::parse(
            "p:percent,r:rating,port:port,id:ssn,sym:currency_symbol,m:mime_type,f:filename,v:semver,h:hashtag,t:base64,x:hex(8)",
        )
        .unwrap();
        let rec = s.generate_record(&mut rng(), Locale::EnUs, 0);
        // Numeric kinds.
        assert!(matches!(rec[0].1, FieldValue::Float(p) if (0.0..=100.0).contains(&p)));
        assert!(matches!(rec[1].1, FieldValue::Float(r) if (1.0..=5.0).contains(&r)));
        assert!(matches!(rec[2].1, FieldValue::Int(p) if (1024..=65535).contains(&p)));
        // String kinds with structural checks.
        if let FieldValue::Str(ssn) = &rec[3].1 {
            assert_eq!(ssn.split('-').count(), 3);
        } else {
            panic!("ssn should be a string");
        }
        if let FieldValue::Str(ver) = &rec[7].1 {
            assert_eq!(ver.split('.').count(), 3);
        } else {
            panic!("semver should be a string");
        }
        if let FieldValue::Str(tag) = &rec[8].1 {
            assert!(tag.starts_with('#'));
        } else {
            panic!("hashtag should be a string");
        }
        if let FieldValue::Str(hex) = &rec[10].1 {
            assert_eq!(hex.len(), 8);
            assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
        } else {
            panic!("hex should be a string");
        }
    }

    #[test]
    fn test_sql_type_inference() {
        let s = Schema::parse("a:int,b:float,c:bool,d:name,e:sequence,f:http_status").unwrap();
        assert_eq!(s.fields[0].sql_type(), "INTEGER");
        assert_eq!(s.fields[1].sql_type(), "REAL");
        assert_eq!(s.fields[2].sql_type(), "BOOLEAN");
        assert_eq!(s.fields[3].sql_type(), "TEXT");
        assert_eq!(s.fields[4].sql_type(), "INTEGER");
        assert_eq!(s.fields[5].sql_type(), "INTEGER");
    }

    #[test]
    fn test_new_web_and_geo_types() {
        let s = Schema::parse(
            "m:http_method,s:http_status,o:os,b:browser,d:device,bic:bic,e:ean,i:imei,co:coordinates,fs:file_size,ce:company_email,jl:job_level,cn:card_network",
        )
        .unwrap();
        let rec = s.generate_record(&mut rng(), Locale::EnUs, 0);
        // http_status is numeric, everything else is a string.
        assert!(matches!(rec[1].1, FieldValue::Int(c) if (100..=599).contains(&c)));
        for (i, (_, v)) in rec.iter().enumerate() {
            if i == 1 {
                continue;
            }
            assert!(
                matches!(v, FieldValue::Str(_)),
                "field {i} should be a string"
            );
        }
    }

    #[test]
    fn test_is_known_type() {
        assert!(is_known_type("email"));
        assert!(is_known_type("Email")); // case-insensitive
        assert!(is_known_type("http_status"));
        assert!(is_known_type("enum"));
        assert!(!is_known_type("emial"));
        assert!(!is_known_type("totally_bogus"));
    }

    #[test]
    fn test_suggest_type() {
        assert_eq!(suggest_type("emial"), Some("email"));
        assert_eq!(suggest_type("nmae"), Some("name"));
        assert_eq!(suggest_type("uuidd"), Some("uuid"));
        // A known type suggests nothing (distance 0 is filtered out).
        assert_eq!(suggest_type("email"), None);
        // Nonsense far from any type yields no suggestion.
        assert_eq!(suggest_type("xyzzyqwerty"), None);
    }

    #[test]
    fn test_unknown_field_types() {
        let s = Schema::parse("a:email,b:emial,c:int,d:array(boguss,2)").unwrap();
        let unknown = s.unknown_field_types();
        assert_eq!(unknown, vec!["emial".to_string(), "boguss".to_string()]);
    }

    #[test]
    fn test_known_types_cover_catalogue() {
        // Every displayed catalogue type must be a recognized base name, so the
        // `list` reference can never advertise a type the engine doesn't know.
        for (_, types) in FIELD_TYPE_GROUPS {
            for entry in *types {
                let base: String = entry
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                if base == "type" {
                    continue; // the `type? / type?p (nullable)` pseudo-entry
                }
                assert!(
                    is_known_type(&base),
                    "catalogue type '{base}' (from '{entry}') is not in KNOWN_TYPE_NAMES"
                );
            }
        }
    }

    #[test]
    fn test_field_value_serialization() {
        assert_eq!(FieldValue::Null.to_flat_string(), "");
        assert_eq!(FieldValue::Null.to_sql_literal(), "NULL");
        assert_eq!(FieldValue::Int(3).to_json(), serde_json::json!(3));
        assert_eq!(
            FieldValue::Str("O'Brien".into()).to_sql_literal(),
            "'O''Brien'"
        );
        assert_eq!(FieldValue::Bool(true).to_sql_literal(), "TRUE");
    }

    #[test]
    fn test_deterministic_records() {
        let s = Schema::parse("id:sequence,name:name,age:int(18..65)").unwrap();
        let a = s.generate_records(&mut rng(), Locale::EnUs, 10);
        let b = s.generate_records(&mut rng(), Locale::EnUs, 10);
        assert_eq!(a, b);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    /// A representative vocabulary of valid type specs.
    const TYPE_VOCAB: &[&str] = &[
        "int",
        "int(1..100)",
        "float(0.0..1.0)",
        "bool",
        "name",
        "email",
        "uuid",
        "date",
        "datetime",
        "sequence",
        "sequence(50)",
        "enum(a,b,c)",
        "const(fixed)",
        "array(word,3)",
        "phone?0.3",
        "price(1..9)",
        "city",
        "ipv4",
        "percent",
        "rating",
        "port",
        "semver",
        "mime_type",
        "filename",
        "ssn",
        "hex(8)",
    ];

    proptest! {
        /// Parsing arbitrary input must never panic.
        #[test]
        fn parse_never_panics(s in ".{0,80}") {
            let _ = Schema::parse(&s);
        }

        /// Generation always yields `rows` records, each with one value per
        /// declared field, for any combination of valid types.
        #[test]
        fn generation_shape_is_consistent(
            types in prop::collection::vec(prop::sample::select(TYPE_VOCAB), 1..6),
            rows in 0usize..15,
            seed in any::<u64>(),
        ) {
            let schema_str = types
                .iter()
                .enumerate()
                .map(|(i, t)| format!("f{i}:{t}"))
                .collect::<Vec<_>>()
                .join(",");
            let schema = Schema::parse(&schema_str).unwrap();
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let records = schema.generate_records(&mut rng, Locale::EnUs, rows);
            prop_assert_eq!(records.len(), rows);
            for record in &records {
                prop_assert_eq!(record.len(), types.len());
            }
        }

        /// The same seed must produce byte-identical records.
        #[test]
        fn generation_is_deterministic(seed in any::<u64>(), rows in 0usize..12) {
            let schema = Schema::parse(
                "id:sequence,name:name,age:int(1..100),active:bool,tags:array(word,2),v:enum(a,b)",
            )
            .unwrap();
            let mut r1 = ChaCha8Rng::seed_from_u64(seed);
            let mut r2 = ChaCha8Rng::seed_from_u64(seed);
            prop_assert_eq!(
                schema.generate_records(&mut r1, Locale::EnUs, rows),
                schema.generate_records(&mut r2, Locale::EnUs, rows)
            );
        }

        /// Every supported locale generates without panicking for every
        /// vocabulary type.
        #[test]
        fn all_locales_generate(idx in 0usize..TYPE_VOCAB.len(), seed in any::<u64>()) {
            let schema = Schema::parse(&format!("f:{}", TYPE_VOCAB[idx])).unwrap();
            for locale in Locale::variants() {
                let mut rng = ChaCha8Rng::seed_from_u64(seed);
                let recs = schema.generate_records(&mut rng, *locale, 3);
                prop_assert_eq!(recs.len(), 3);
            }
        }
    }
}
