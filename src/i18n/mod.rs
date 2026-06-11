//! Internationalization (i18n) for all user-facing CLI output.
//!
//! `demodatagen` separates two orthogonal notions of "locale":
//!
//! - **Data locale** ([`crate::data::Locale`], `--locale`) selects the *content*
//!   of generated data — German names, Brazilian cities, French companies, ….
//! - **Interface language** ([`Language`], `--lang`) selects the language of the
//!   *program's own messages* — progress, summaries, errors, the `list` output.
//!
//! Both default to English and are independent: you can generate German test
//! data while reading Spanish progress messages, or vice versa.
//!
//! # Design
//!
//! Every translatable string lives in exactly one place — the [`catalog!`]
//! invocation below — as a template with `{placeholder}` slots. The macro turns
//! that single table into a [`Catalog`] struct plus one immutable instance per
//! language, so adding a message (or a language) is a localized, compiler-checked
//! edit: forget a translation and the build fails.
//!
//! # Usage
//!
//! ```
//! use demodatagen::i18n::{fill, Language};
//!
//! let lang = Language::En;
//! let msg = fill(lang.catalog().update_up_to_date, &[("version", "0.4.0".into())]);
//! assert_eq!(msg, "demodatagen is up to date (v0.4.0).");
//! ```
//!
//! Internally the [`tr!`](crate::i18n::tr) macro makes this terser:
//! `tr!(lang, update_up_to_date, "version" => "0.4.0")`.

/// Generates the [`Catalog`] struct and one `const` instance per language from a
/// single declaration. Each entry lists every supported language, so a missing
/// translation is a compile error rather than a silent English fallback.
macro_rules! catalog {
    (
        $( $(#[$doc:meta])* $key:ident {
            en: $en:literal,
            de: $de:literal,
            fr: $fr:literal,
            es: $es:literal $(,)?
        } ),+ $(,)?
    ) => {
        /// An immutable bundle of every user-facing message template for one
        /// [`Language`]. Templates may contain `{name}` placeholders, filled in
        /// at call sites via [`fill`] / [`tr!`](crate::i18n::tr).
        #[derive(Debug, Clone, Copy)]
        #[non_exhaustive]
        pub struct Catalog {
            $( $(#[$doc])* pub $key: &'static str, )+
        }

        const EN_CATALOG: Catalog = Catalog { $( $key: $en, )+ };
        const DE_CATALOG: Catalog = Catalog { $( $key: $de, )+ };
        const FR_CATALOG: Catalog = Catalog { $( $key: $fr, )+ };
        const ES_CATALOG: Catalog = Catalog { $( $key: $es, )+ };
    };
}

catalog! {
    /// One-line product tagline. Placeholders: `{formats}`, `{locales}`.
    tagline {
        en: "Realistic demo files in {formats} formats and {locales} locales — offline, deterministic, dependency-free.",
        de: "Realistische Demo-Dateien in {formats} Formaten und {locales} Sprachräumen — offline, deterministisch, ohne externe Dienste.",
        fr: "Des fichiers de démonstration réalistes dans {formats} formats et {locales} locales — hors ligne, déterministe, sans dépendances.",
        es: "Archivos de demostración realistas en {formats} formatos y {locales} locales — sin conexión, deterministas y sin dependencias.",
    },
    /// Compact line printed when a generation run starts.
    /// Placeholders: `{count}`, `{format}`, `{dir}`.
    generating_header {
        en: "Generating {count} × {format} → {dir}",
        de: "Erzeuge {count} × {format} → {dir}",
        fr: "Génération de {count} × {format} → {dir}",
        es: "Generando {count} × {format} → {dir}",
    },
    /// Progress-bar message. Placeholder: `{format}`.
    progress_message {
        en: "writing {format}",
        de: "schreibe {format}",
        fr: "écriture de {format}",
        es: "escribiendo {format}",
    },
    /// Progress-bar finish word.
    progress_done {
        en: "done",
        de: "fertig",
        fr: "terminé",
        es: "listo",
    },
    /// Success summary. Placeholders: `{count}`, `{bytes}`, `{elapsed}`.
    summary_success {
        en: "Generated {count} file(s) · {bytes} · {elapsed}",
        de: "{count} Datei(en) erzeugt · {bytes} · {elapsed}",
        fr: "{count} fichier(s) généré(s) · {bytes} · {elapsed}",
        es: "{count} archivo(s) generado(s) · {bytes} · {elapsed}",
    },
    /// Output-location line. Placeholder: `{dir}`.
    summary_location {
        en: "Output directory: {dir}",
        de: "Ausgabeverzeichnis: {dir}",
        fr: "Répertoire de sortie : {dir}",
        es: "Directorio de salida: {dir}",
    },
    /// Partial-success summary. Placeholders: `{ok}`, `{total}`, `{errors}`.
    summary_partial {
        en: "Generated {ok}/{total} file(s) — {errors} error(s)",
        de: "{ok}/{total} Datei(en) erzeugt — {errors} Fehler",
        fr: "{ok}/{total} fichier(s) généré(s) — {errors} erreur(s)",
        es: "{ok}/{total} archivo(s) generado(s) — {errors} error(es)",
    },
    /// Full-failure message. Placeholders: `{total}`, `{error}`.
    summary_failed {
        en: "All {total} file(s) failed to generate. First error: {error}",
        de: "Alle {total} Datei(en) konnten nicht erzeugt werden. Erster Fehler: {error}",
        fr: "Les {total} fichier(s) n'ont pas pu être générés. Première erreur : {error}",
        es: "No se pudo generar ninguno de los {total} archivo(s). Primer error: {error}",
    },
    /// Unknown output format. Placeholder: `{format}`.
    err_unknown_format {
        en: "Unknown format: {format}",
        de: "Unbekanntes Format: {format}",
        fr: "Format inconnu : {format}",
        es: "Formato desconocido: {format}",
    },
    /// Generation-failed prefix. Placeholder: `{error}`.
    err_generation_failed {
        en: "Generation failed: {error}",
        de: "Erzeugung fehlgeschlagen: {error}",
        fr: "Échec de la génération : {error}",
        es: "Error en la generación: {error}",
    },
    /// Warning when an enum-like argument is invalid and a default is used.
    /// Placeholders: `{error}`, `{default}`.
    warn_fallback {
        en: "{error} Falling back to the default '{default}'.",
        de: "{error} Verwende stattdessen den Standardwert '{default}'.",
        fr: "{error} Utilisation de la valeur par défaut « {default} ».",
        es: "{error} Se usará el valor predeterminado '{default}'.",
    },
    /// Update check started. Placeholder: `{version}`.
    update_checking {
        en: "Checking for updates (current: v{version})…",
        de: "Suche nach Updates (aktuell: v{version})…",
        fr: "Recherche de mises à jour (actuelle : v{version})…",
        es: "Buscando actualizaciones (actual: v{version})…",
    },
    /// Newer version available. Placeholders: `{current}`, `{latest}`.
    update_available {
        en: "Update available: v{current} → v{latest}",
        de: "Update verfügbar: v{current} → v{latest}",
        fr: "Mise à jour disponible : v{current} → v{latest}",
        es: "Actualización disponible: v{current} → v{latest}",
    },
    /// How to apply an update. Placeholder: `{url}`.
    update_hint {
        en: "Run `demodatagen update` to upgrade, or download from {url}",
        de: "Führe `demodatagen update` aus, oder lade es von {url} herunter",
        fr: "Lancez « demodatagen update » pour mettre à jour, ou téléchargez depuis {url}",
        es: "Ejecuta `demodatagen update` para actualizar, o descárgalo desde {url}",
    },
    /// Already current. Placeholder: `{version}`.
    update_up_to_date {
        en: "demodatagen is up to date (v{version}).",
        de: "demodatagen ist aktuell (v{version}).",
        fr: "demodatagen est à jour (v{version}).",
        es: "demodatagen está actualizado (v{version}).",
    },
    /// Could not query releases. Placeholder: `{url}`.
    update_unknown {
        en: "Could not determine the latest version. Check {url} manually.",
        de: "Die neueste Version konnte nicht ermittelt werden. Prüfe {url} manuell.",
        fr: "Impossible de déterminer la dernière version. Vérifiez {url} manuellement.",
        es: "No se pudo determinar la última versión. Comprueba {url} manualmente.",
    },
    /// Update succeeded. Placeholders: `{from}`, `{to}`.
    update_updated {
        en: "Updated demodatagen v{from} → v{to}.",
        de: "demodatagen aktualisiert: v{from} → v{to}.",
        fr: "demodatagen mis à jour : v{from} → v{to}.",
        es: "demodatagen actualizado: v{from} → v{to}.",
    },
    /// Prompt to restart after updating.
    update_restart {
        en: "Restart the command to use the new version.",
        de: "Starte den Befehl neu, um die neue Version zu verwenden.",
        fr: "Relancez la commande pour utiliser la nouvelle version.",
        es: "Reinicia el comando para usar la nueva versión.",
    },
    /// Update was a no-op. Placeholder: `{version}`.
    update_already_latest {
        en: "Already running the latest version (v{version}).",
        de: "Es läuft bereits die neueste Version (v{version}).",
        fr: "Vous utilisez déjà la dernière version (v{version}).",
        es: "Ya estás ejecutando la última versión (v{version}).",
    },
    /// Update failed. Placeholder: `{error}`.
    update_failed {
        en: "Update failed: {error}",
        de: "Update fehlgeschlagen: {error}",
        fr: "Échec de la mise à jour : {error}",
        es: "Error en la actualización: {error}",
    },
    /// Update-check failed. Placeholder: `{error}`.
    update_check_failed {
        en: "Update check failed: {error}",
        de: "Update-Prüfung fehlgeschlagen: {error}",
        fr: "Échec de la vérification des mises à jour : {error}",
        es: "Error al comprobar actualizaciones: {error}",
    },
    /// Self-update compiled out.
    update_disabled {
        en: "Self-update support is disabled in this build. Rebuild with `--features update`.",
        de: "Die Selbst-Update-Funktion ist in diesem Build deaktiviert. Neu bauen mit `--features update`.",
        fr: "La mise à jour automatique est désactivée dans cette version. Recompilez avec « --features update ».",
        es: "La autoactualización está deshabilitada en esta compilación. Recompila con `--features update`.",
    },
    /// `list` title.
    list_title {
        en: "Supported output formats",
        de: "Unterstützte Ausgabeformate",
        fr: "Formats de sortie pris en charge",
        es: "Formatos de salida compatibles",
    },
    /// `list` group header.
    group_structured {
        en: "Structured data",
        de: "Strukturierte Daten",
        fr: "Données structurées",
        es: "Datos estructurados",
    },
    /// `list` group header.
    group_text {
        en: "Text & config",
        de: "Text & Konfiguration",
        fr: "Texte & configuration",
        es: "Texto y configuración",
    },
    /// `list` group header.
    group_images {
        en: "Images",
        de: "Bilder",
        fr: "Images",
        es: "Imágenes",
    },
    /// `list` group header.
    group_av {
        en: "Audio & video",
        de: "Audio & Video",
        fr: "Audio & vidéo",
        es: "Audio y vídeo",
    },
    /// `list` group header.
    group_docs {
        en: "Documents",
        de: "Dokumente",
        fr: "Documents",
        es: "Documentos",
    },
    /// `list` group header.
    group_binary {
        en: "Binary & archives",
        de: "Binär & Archive",
        fr: "Binaire & archives",
        es: "Binarios y archivos",
    },
    /// `list` schema-types section title.
    list_schema_title {
        en: "Schema field types (use as `field:type` in --schema)",
        de: "Schema-Feldtypen (Verwendung als `feld:typ` in --schema)",
        fr: "Types de champs de schéma (à utiliser comme `champ:type` dans --schema)",
        es: "Tipos de campo del esquema (usar como `campo:tipo` en --schema)",
    },
    /// `list` data-locale section title.
    list_locales_title {
        en: "Data locales (--locale)",
        de: "Daten-Sprachräume (--locale)",
        fr: "Locales de données (--locale)",
        es: "Locales de datos (--locale)",
    },
    /// `list` interface-language section title.
    list_langs_title {
        en: "Interface languages (--lang)",
        de: "Oberflächensprachen (--lang)",
        fr: "Langues de l'interface (--lang)",
        es: "Idiomas de la interfaz (--lang)",
    },
    /// Closing tip in `list`.
    list_hint {
        en: "Tip: run any format with --help to see its options, e.g. `demodatagen json --help`.",
        de: "Tipp: Jedes Format mit --help aufrufen, um seine Optionen zu sehen, z. B. `demodatagen json --help`.",
        fr: "Astuce : lancez un format avec --help pour voir ses options, p. ex. « demodatagen json --help ».",
        es: "Consejo: ejecuta cualquier formato con --help para ver sus opciones, p. ej. `demodatagen json --help`.",
    },
}

/// The language of the program's own user-facing messages.
///
/// Independent of the [data locale](crate::data::Locale): `--lang` controls the
/// interface, `--locale` controls the generated content. English is the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Language {
    /// English (default).
    #[default]
    En,
    /// German.
    De,
    /// French.
    Fr,
    /// Spanish.
    Es,
}

impl Language {
    /// Returns the immutable message catalog for this language.
    pub fn catalog(self) -> &'static Catalog {
        match self {
            Language::En => &EN_CATALOG,
            Language::De => &DE_CATALOG,
            Language::Fr => &FR_CATALOG,
            Language::Es => &ES_CATALOG,
        }
    }

    /// Returns the canonical lowercase identifier (e.g. `"en"`).
    pub fn as_str(self) -> &'static str {
        match self {
            Language::En => "en",
            Language::De => "de",
            Language::Fr => "fr",
            Language::Es => "es",
        }
    }

    /// Returns a human-readable label (e.g. `"English"`).
    pub fn label(self) -> &'static str {
        match self {
            Language::En => "English",
            Language::De => "Deutsch",
            Language::Fr => "Français",
            Language::Es => "Español",
        }
    }

    /// Returns every supported language, in declaration order.
    pub fn variants() -> &'static [Language] {
        &[Language::En, Language::De, Language::Fr, Language::Es]
    }

    /// Returns all canonical language identifiers, for help text and `list`.
    pub fn all() -> &'static [&'static str] {
        &["en", "de", "fr", "es"]
    }

    /// Resolves the interface language from, in order of precedence:
    ///
    /// 1. an explicit `--lang` value (if it parses);
    /// 2. the `DEMODATAGEN_LANG` environment variable;
    /// 3. the standard `LC_ALL` / `LC_MESSAGES` / `LANG` / `LANGUAGE` variables;
    /// 4. English.
    ///
    /// Unparseable values are skipped rather than fatal, so a typo never aborts
    /// a run — at worst the program speaks English.
    pub fn detect(explicit: Option<&str>) -> Language {
        if let Some(code) = explicit {
            if let Ok(lang) = code.parse() {
                return lang;
            }
        }
        for var in [
            "DEMODATAGEN_LANG",
            "LC_ALL",
            "LC_MESSAGES",
            "LANG",
            "LANGUAGE",
        ] {
            if let Ok(value) = std::env::var(var) {
                // Values look like "de_DE.UTF-8", "fr_FR", or "en_US:en".
                let primary = value
                    .split([':', '.', '_', '-'])
                    .next()
                    .unwrap_or("")
                    .trim();
                if let Ok(lang) = primary.parse() {
                    return lang;
                }
            }
        }
        Language::default()
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().replace('-', "_").as_str() {
            "en" | "en_us" | "en_gb" | "english" => Ok(Language::En),
            "de" | "de_de" | "german" | "deutsch" => Ok(Language::De),
            "fr" | "fr_fr" | "french" | "francais" | "français" => Ok(Language::Fr),
            "es" | "es_es" | "spanish" | "espanol" | "español" => Ok(Language::Es),
            other => Err(format!(
                "Unknown interface language: '{other}'. Valid: {}",
                Language::all().join(", ")
            )),
        }
    }
}

/// Fills a message template by replacing each `{name}` placeholder with its
/// value. Placeholders without a matching argument are left untouched, and the
/// no-argument case returns the template unchanged.
pub fn fill(template: &str, args: &[(&str, String)]) -> String {
    if args.is_empty() {
        return template.to_string();
    }
    let mut out = template.to_string();
    for (name, value) in args {
        out = out.replace(&format!("{{{name}}}"), value);
    }
    out
}

/// Ergonomic wrapper around [`fill`] keyed on a [`Language`] and a [`Catalog`]
/// field name.
///
/// ```
/// use demodatagen::i18n::{tr, Language};
/// let s = tr!(Language::En, update_up_to_date, "version" => "0.4.0");
/// assert_eq!(s, "demodatagen is up to date (v0.4.0).");
/// ```
#[macro_export]
macro_rules! tr {
    ($lang:expr, $key:ident) => {
        $crate::i18n::fill($lang.catalog().$key, &[])
    };
    ($lang:expr, $key:ident, $( $name:literal => $val:expr ),+ $(,)?) => {
        $crate::i18n::fill(
            $lang.catalog().$key,
            &[ $( ($name, ($val).to_string()) ),+ ],
        )
    };
}

pub use crate::tr;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_english() {
        assert_eq!(Language::default(), Language::En);
    }

    #[test]
    fn test_parse_variants() {
        assert_eq!("en".parse::<Language>().unwrap(), Language::En);
        assert_eq!("DE".parse::<Language>().unwrap(), Language::De);
        assert_eq!("fr-FR".parse::<Language>().unwrap(), Language::Fr);
        assert_eq!("Spanish".parse::<Language>().unwrap(), Language::Es);
        assert!("xx".parse::<Language>().is_err());
    }

    #[test]
    fn test_roundtrip() {
        for id in Language::all() {
            assert_eq!(id.parse::<Language>().unwrap().as_str(), *id);
        }
        assert_eq!(Language::variants().len(), Language::all().len());
    }

    #[test]
    fn test_detect_precedence() {
        // An explicit, valid value always wins.
        assert_eq!(Language::detect(Some("de")), Language::De);
        // An invalid explicit value is ignored (falls through to default here).
        assert_eq!(Language::detect(Some("klingon")), Language::default());
    }

    #[test]
    fn test_fill_replaces_placeholders() {
        assert_eq!(
            fill(
                "v{from} → v{to}",
                &[("from", "1".into()), ("to", "2".into())]
            ),
            "v1 → v2"
        );
        // No-arg returns the template verbatim.
        assert_eq!(fill("plain", &[]), "plain");
        // Unknown placeholders are left alone.
        assert_eq!(fill("{a}{b}", &[("a", "X".into())]), "X{b}");
    }

    #[test]
    fn test_tr_macro() {
        assert_eq!(
            tr!(Language::En, update_restart),
            "Restart the command to use the new version."
        );
        assert_eq!(
            tr!(Language::De, update_available, "current" => "1.0.0", "latest" => "1.1.0"),
            "Update verfügbar: v1.0.0 → v1.1.0"
        );
    }

    #[test]
    fn test_all_catalogs_have_no_empty_strings() {
        // Catch a forgotten translation that was left as "".
        for lang in Language::variants() {
            let c = lang.catalog();
            for field in [
                c.tagline,
                c.generating_header,
                c.progress_message,
                c.progress_done,
                c.summary_success,
                c.summary_location,
                c.summary_partial,
                c.summary_failed,
                c.err_unknown_format,
                c.err_generation_failed,
                c.warn_fallback,
                c.update_checking,
                c.update_available,
                c.update_hint,
                c.update_up_to_date,
                c.update_unknown,
                c.update_updated,
                c.update_restart,
                c.update_already_latest,
                c.update_failed,
                c.update_check_failed,
                c.update_disabled,
                c.list_title,
                c.group_structured,
                c.group_text,
                c.group_images,
                c.group_av,
                c.group_docs,
                c.group_binary,
                c.list_schema_title,
                c.list_locales_title,
                c.list_langs_title,
                c.list_hint,
            ] {
                assert!(!field.trim().is_empty(), "{lang} has an empty message");
            }
        }
    }
}
