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
//! data while reading Spanish progress messages, or vice versa. The interface is
//! translated into **nine languages** — one for every language family covered by
//! the data locales: English, German, French, Spanish, Italian, Portuguese,
//! Dutch, Polish, and Swedish.
//!
//! # Design
//!
//! Every translatable string lives in exactly one place — the `catalog!`
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
//! let msg = fill(lang.catalog().update_up_to_date, &[("version", "0.5.0".into())]);
//! assert_eq!(msg, "demodatagen is up to date (v0.5.0).");
//! ```
//!
//! Internally the [`tr!`](crate::i18n::tr) macro makes this terser:
//! `tr!(lang, update_up_to_date, "version" => "0.5.0")`.

/// Generates the [`Catalog`] struct and one `const` instance per language from a
/// single declaration. Each entry lists every supported language, so a missing
/// translation is a compile error rather than a silent English fallback.
macro_rules! catalog {
    (
        $( $(#[$doc:meta])* $key:ident {
            en: $en:literal,
            de: $de:literal,
            fr: $fr:literal,
            es: $es:literal,
            it: $it:literal,
            pt: $pt:literal,
            nl: $nl:literal,
            pl: $pl:literal,
            sv: $sv:literal $(,)?
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

        impl Catalog {
            /// Returns every message template in this catalog, in declaration
            /// order. Used by tooling and the completeness test so that adding a
            /// key never requires touching a hand-maintained list.
            pub fn fields(&self) -> Vec<&'static str> {
                vec![ $( self.$key ),+ ]
            }
        }

        const EN_CATALOG: Catalog = Catalog { $( $key: $en, )+ };
        const DE_CATALOG: Catalog = Catalog { $( $key: $de, )+ };
        const FR_CATALOG: Catalog = Catalog { $( $key: $fr, )+ };
        const ES_CATALOG: Catalog = Catalog { $( $key: $es, )+ };
        const IT_CATALOG: Catalog = Catalog { $( $key: $it, )+ };
        const PT_CATALOG: Catalog = Catalog { $( $key: $pt, )+ };
        const NL_CATALOG: Catalog = Catalog { $( $key: $nl, )+ };
        const PL_CATALOG: Catalog = Catalog { $( $key: $pl, )+ };
        const SV_CATALOG: Catalog = Catalog { $( $key: $sv, )+ };
    };
}

catalog! {
    /// Product tagline. Placeholders {formats} and {locales} are numbers. Keep the three trailing adjectives.
    tagline {
        en: "Realistic demo files in {formats} formats and {locales} locales — offline, deterministic, dependency-free.",
        de: "Realistische Demo-Dateien in {formats} Formaten und {locales} Sprachräumen — offline, deterministisch, ohne externe Dienste.",
        fr: "Fichiers de démonstration réalistes dans {formats} formats et {locales} régions linguistiques — hors ligne, déterministe, sans dépendances externes.",
        es: "Archivos de demostración realistas en {formats} formatos y {locales} regiones — sin conexión, deterministas y sin dependencias.",
        it: "File demo realistici in {formats} formati e {locales} aree linguistiche — offline, deterministici, senza dipendenze.",
        pt: "Arquivos de demonstração realistas em {formats} formatos e {locales} localidades — offline, determinístico, sem dependências externas.",
        nl: "Realistische demobestanden in {formats} formaten en {locales} taalregio's — offline, deterministisch, zonder externe diensten.",
        pl: "Realistyczne pliki demonstracyjne w {formats} formatach i {locales} regionach — offline, deterministycznie, bez zewnętrznych usług.",
        sv: "Realistiska demofiler i {formats} format och {locales} språkområden — offline, deterministiska, utan externa beroenden.",
    },
    /// Printed when a run starts. {count} number, {format} format name, {dir} path. Keep the × and arrow.
    generating_header {
        en: "Generating {count} × {format} → {dir}",
        de: "Erzeuge {count} × {format} → {dir}",
        fr: "Génération de {count} × {format} → {dir}",
        es: "Generando {count} × {format} → {dir}",
        it: "Generazione di {count} × {format} → {dir}",
        pt: "Gerando {count} × {format} → {dir}",
        nl: "Genereren van {count} × {format} → {dir}",
        pl: "Generowanie {count} × {format} → {dir}",
        sv: "Genererar {count} × {format} → {dir}",
    },
    /// Spinner/progress-bar message while writing. {format} is the format name.
    progress_message {
        en: "writing {format}",
        de: "schreibe {format}",
        fr: "écriture de {format}",
        es: "escribiendo {format}",
        it: "scrittura {format}",
        pt: "gravando {format}",
        nl: "{format} schrijven",
        pl: "zapisywanie {format}",
        sv: "skriver {format}",
    },
    /// Single word shown when the progress bar finishes.
    progress_done {
        en: "done",
        de: "fertig",
        fr: "terminé",
        es: "listo",
        it: "completato",
        pt: "concluído",
        nl: "klaar",
        pl: "gotowe",
        sv: "klar",
    },
    /// Success summary line. {count} number, {bytes} human size e.g. '1.2 MB', {elapsed} duration e.g. '3s'.
    summary_success {
        en: "Generated {count} file(s) · {bytes} · {elapsed}",
        de: "{count} Datei(en) erzeugt · {bytes} · {elapsed}",
        fr: "{count} fichier(s) généré(s) · {bytes} · {elapsed}",
        es: "Se generaron {count} archivo(s) · {bytes} · {elapsed}",
        it: "{count} file generati · {bytes} · {elapsed}",
        pt: "{count} arquivo(s) gerado(s) · {bytes} · {elapsed}",
        nl: "{count} bestand(en) gegenereerd · {bytes} · {elapsed}",
        pl: "Wygenerowano {count} plik(ów) · {bytes} · {elapsed}",
        sv: "{count} fil(er) genererade · {bytes} · {elapsed}",
    },
    /// Shows where files were written. {dir} is a path.
    summary_location {
        en: "Output directory: {dir}",
        de: "Ausgabeverzeichnis: {dir}",
        fr: "Répertoire de sortie : {dir}",
        es: "Directorio de salida: {dir}",
        it: "Directory di output: {dir}",
        pt: "Diretório de saída: {dir}",
        nl: "Uitvoermap: {dir}",
        pl: "Katalog wyjściowy: {dir}",
        sv: "Utdatakatalog: {dir}",
    },
    /// Some files failed. {ok} succeeded, {total} attempted, {errors} failed.
    summary_partial {
        en: "Generated {ok}/{total} file(s) — {errors} error(s)",
        de: "{ok}/{total} Datei(en) erzeugt — {errors} Fehler",
        fr: "{ok}/{total} fichier(s) généré(s) — {errors} erreur(s)",
        es: "Se generaron {ok}/{total} archivo(s) — {errors} error(es)",
        it: "Generati {ok}/{total} file — {errors} errori",
        pt: "{ok}/{total} arquivo(s) gerado(s) — {errors} erro(s)",
        nl: "{ok}/{total} bestand(en) gegenereerd — {errors} fout(en)",
        pl: "Wygenerowano {ok}/{total} plik(ów) — {errors} błąd(ów)",
        sv: "{ok}/{total} fil(er) genererade — {errors} fel",
    },
    /// All files failed. {total} number, {error} the first error message.
    summary_failed {
        en: "All {total} file(s) failed to generate. First error: {error}",
        de: "Alle {total} Datei(en) konnten nicht erzeugt werden. Erster Fehler: {error}",
        fr: "Échec de la génération des {total} fichier(s). Première erreur : {error}",
        es: "No se pudo generar ninguno de los {total} archivo(s). Primer error: {error}",
        it: "Impossibile generare tutti i {total} file. Primo errore: {error}",
        pt: "Falha ao gerar todos os {total} arquivo(s). Primeiro erro: {error}",
        nl: "Alle {total} bestand(en) konden niet worden gegenereerd. Eerste fout: {error}",
        pl: "Nie udało się wygenerować żadnego z {total} plik(ów). Pierwszy błąd: {error}",
        sv: "Alla {total} fil(er) kunde inte genereras. Första felet: {error}",
    },
    /// {format} is the unknown format key the user typed.
    err_unknown_format {
        en: "Unknown format: {format}",
        de: "Unbekanntes Format: {format}",
        fr: "Format inconnu : {format}",
        es: "Formato desconocido: {format}",
        it: "Formato sconosciuto: {format}",
        pt: "Formato desconhecido: {format}",
        nl: "Onbekend formaat: {format}",
        pl: "Nieznany format: {format}",
        sv: "Okänt format: {format}",
    },
    /// {error} is an error message.
    err_generation_failed {
        en: "Generation failed: {error}",
        de: "Erzeugung fehlgeschlagen: {error}",
        fr: "Échec de la génération : {error}",
        es: "Error en la generación: {error}",
        it: "Generazione non riuscita: {error}",
        pt: "Falha na geração: {error}",
        nl: "Genereren mislukt: {error}",
        pl: "Generowanie nie powiodło się: {error}",
        sv: "Genereringen misslyckades: {error}",
    },
    /// {preset} is the unknown preset name. Keep the backticked command verbatim.
    err_invalid_preset {
        en: "Unknown preset: {preset}. Run `demodatagen presets` to list the built-in presets.",
        de: "Unbekanntes Preset: {preset}. Führe `demodatagen presets` aus, um die eingebauten Presets aufzulisten.",
        fr: "Préréglage inconnu : {preset}. Exécutez `demodatagen presets` pour lister les préréglages intégrés.",
        es: "Preset desconocido: {preset}. Ejecuta `demodatagen presets` para ver los presets integrados.",
        it: "Preset sconosciuto: {preset}. Esegui `demodatagen presets` per elencare i preset integrati.",
        pt: "Predefinição desconhecida: {preset}. Execute `demodatagen presets` para listar as predefinições integradas.",
        nl: "Onbekende preset: {preset}. Voer `demodatagen presets` uit om de ingebouwde presets weer te geven.",
        pl: "Nieznany szablon: {preset}. Uruchom `demodatagen presets`, aby wyświetlić wbudowane szablony.",
        sv: "Okänd förinställning: {preset}. Kör `demodatagen presets` för att lista de inbyggda förinställningarna.",
    },
    /// An invalid enum-like value was given; we use a default. {error} the parse error, {default} the value used.
    warn_fallback {
        en: "{error} Falling back to the default '{default}'.",
        de: "{error} Verwende stattdessen den Standardwert '{default}'.",
        fr: "{error} Utilisation de la valeur par défaut « {default} » à la place.",
        es: "{error} Se usará el valor predeterminado '{default}'.",
        it: "{error} Verrà usato il valore predefinito '{default}'.",
        pt: "{error} Usando o valor padrão '{default}'.",
        nl: "{error} Standaardwaarde '{default}' wordt in plaats daarvan gebruikt.",
        pl: "{error} Używam wartości domyślnej '{default}'.",
        sv: "{error} Använder standardvärdet '{default}' istället.",
    },
    /// Schema has an unknown field type but we guessed a close one. {type} typed, {suggestion} the closest known type.
    warn_unknown_type {
        en: "Unknown schema type '{type}'; did you mean '{suggestion}'? Generating a generic word instead.",
        de: "Unbekannter Schema-Typ '{type}'; meintest du '{suggestion}'? Erzeuge stattdessen ein generisches Wort.",
        fr: "Type de schéma inconnu « {type} » ; vouliez-vous dire « {suggestion} » ? Génération d'un mot générique à la place.",
        es: "Tipo de esquema desconocido '{type}'; ¿querías decir '{suggestion}'? Se generará una palabra genérica en su lugar.",
        it: "Tipo di schema sconosciuto '{type}'; intendevi '{suggestion}'? Verrà generata invece una parola generica.",
        pt: "Tipo de esquema desconhecido '{type}'; você quis dizer '{suggestion}'? Gerando uma palavra genérica em vez disso.",
        nl: "Onbekend schematype '{type}'; bedoelde je '{suggestion}'? In plaats daarvan wordt een algemeen woord gegenereerd.",
        pl: "Nieznany typ schematu '{type}'; czy chodziło o '{suggestion}'? Generuję zamiast tego ogólne słowo.",
        sv: "Okänd schematyp '{type}'; menade du '{suggestion}'? Genererar ett generiskt ord istället.",
    },
    /// Schema has an unknown field type with no close match. {type} is what the user typed.
    warn_unknown_type_plain {
        en: "Unknown schema type '{type}'; generating a generic word instead.",
        de: "Unbekannter Schema-Typ '{type}'; erzeuge stattdessen ein generisches Wort.",
        fr: "Type de schéma inconnu « {type} » ; génération d'un mot générique à la place.",
        es: "Tipo de esquema desconocido '{type}'; se generará una palabra genérica en su lugar.",
        it: "Tipo di schema sconosciuto '{type}'; verrà generata invece una parola generica.",
        pt: "Tipo de esquema desconhecido '{type}'; gerando uma palavra genérica em vez disso.",
        nl: "Onbekend schematype '{type}'; in plaats daarvan wordt een algemeen woord gegenereerd.",
        pl: "Nieznany typ schematu '{type}'; generuję zamiast tego ogólne słowo.",
        sv: "Okänd schematyp '{type}'; genererar ett generiskt ord istället.",
    },
    /// {version} current version without leading v. Keep the ellipsis …
    update_checking {
        en: "Checking for updates (current: v{version})…",
        de: "Suche nach Updates (aktuell: v{version})…",
        fr: "Recherche de mises à jour (actuelle : v{version})…",
        es: "Buscando actualizaciones (actual: v{version})…",
        it: "Ricerca di aggiornamenti (attuale: v{version})…",
        pt: "Procurando atualizações (atual: v{version})…",
        nl: "Controleren op updates (huidig: v{version})…",
        pl: "Sprawdzanie aktualizacji (bieżąca: v{version})…",
        sv: "Söker efter uppdateringar (aktuell: v{version})…",
    },
    /// {current} and {latest} are versions. Keep the arrow.
    update_available {
        en: "Update available: v{current} → v{latest}",
        de: "Update verfügbar: v{current} → v{latest}",
        fr: "Mise à jour disponible : v{current} → v{latest}",
        es: "Actualización disponible: v{current} → v{latest}",
        it: "Aggiornamento disponibile: v{current} → v{latest}",
        pt: "Atualização disponível: v{current} → v{latest}",
        nl: "Update beschikbaar: v{current} → v{latest}",
        pl: "Dostępna aktualizacja: v{current} → v{latest}",
        sv: "Uppdatering tillgänglig: v{current} → v{latest}",
    },
    /// {url} a URL. Keep the backticked command verbatim.
    update_hint {
        en: "Run `demodatagen update` to upgrade, or download from {url}",
        de: "Führe `demodatagen update` aus, oder lade es von {url} herunter",
        fr: "Exécutez `demodatagen update` pour mettre à jour, ou téléchargez depuis {url}",
        es: "Ejecuta `demodatagen update` para actualizar, o descárgalo desde {url}",
        it: "Esegui `demodatagen update` per aggiornare, oppure scaricalo da {url}",
        pt: "Execute `demodatagen update` para atualizar, ou baixe em {url}",
        nl: "Voer `demodatagen update` uit om bij te werken, of download van {url}",
        pl: "Uruchom `demodatagen update`, aby zaktualizować, lub pobierz z {url}",
        sv: "Kör `demodatagen update` för att uppgradera, eller ladda ner från {url}",
    },
    /// {version} current version.
    update_up_to_date {
        en: "demodatagen is up to date (v{version}).",
        de: "demodatagen ist aktuell (v{version}).",
        fr: "demodatagen est à jour (v{version}).",
        es: "demodatagen está actualizado (v{version}).",
        it: "demodatagen è aggiornato (v{version}).",
        pt: "demodatagen está atualizado (v{version}).",
        nl: "demodatagen is up-to-date (v{version}).",
        pl: "demodatagen jest aktualny (v{version}).",
        sv: "demodatagen är uppdaterad (v{version}).",
    },
    /// Could not query releases. {url} a URL.
    update_unknown {
        en: "Could not determine the latest version. Check {url} manually.",
        de: "Die neueste Version konnte nicht ermittelt werden. Prüfe {url} manuell.",
        fr: "Impossible de déterminer la dernière version. Vérifiez {url} manuellement.",
        es: "No se pudo determinar la última versión. Comprueba {url} manualmente.",
        it: "Impossibile determinare l'ultima versione. Controlla {url} manualmente.",
        pt: "Não foi possível determinar a versão mais recente. Verifique {url} manualmente.",
        nl: "De nieuwste versie kon niet worden bepaald. Controleer {url} handmatig.",
        pl: "Nie udało się ustalić najnowszej wersji. Sprawdź {url} ręcznie.",
        sv: "Den senaste versionen kunde inte fastställas. Kontrollera {url} manuellt.",
    },
    /// {from} and {to} are versions.
    update_updated {
        en: "Updated demodatagen v{from} → v{to}.",
        de: "demodatagen aktualisiert: v{from} → v{to}.",
        fr: "demodatagen mis à jour : v{from} → v{to}.",
        es: "demodatagen actualizado: v{from} → v{to}.",
        it: "demodatagen aggiornato: v{from} → v{to}.",
        pt: "demodatagen atualizado: v{from} → v{to}.",
        nl: "demodatagen bijgewerkt: v{from} → v{to}.",
        pl: "Zaktualizowano demodatagen: v{from} → v{to}.",
        sv: "demodatagen uppdaterad: v{from} → v{to}.",
    },
    /// Tell the user to restart the command.
    update_restart {
        en: "Restart the command to use the new version.",
        de: "Starte den Befehl neu, um die neue Version zu verwenden.",
        fr: "Relancez la commande pour utiliser la nouvelle version.",
        es: "Reinicia el comando para usar la nueva versión.",
        it: "Riavvia il comando per usare la nuova versione.",
        pt: "Reinicie o comando para usar a nova versão.",
        nl: "Start de opdracht opnieuw om de nieuwe versie te gebruiken.",
        pl: "Uruchom polecenie ponownie, aby skorzystać z nowej wersji.",
        sv: "Starta om kommandot för att använda den nya versionen.",
    },
    /// {version} current version.
    update_already_latest {
        en: "Already running the latest version (v{version}).",
        de: "Es läuft bereits die neueste Version (v{version}).",
        fr: "La dernière version est déjà installée (v{version}).",
        es: "Ya se está ejecutando la última versión (v{version}).",
        it: "È già in esecuzione l'ultima versione (v{version}).",
        pt: "Você já está executando a versão mais recente (v{version}).",
        nl: "De nieuwste versie is al actief (v{version}).",
        pl: "Najnowsza wersja jest już uruchomiona (v{version}).",
        sv: "Du kör redan den senaste versionen (v{version}).",
    },
    /// {error} an error message.
    update_failed {
        en: "Update failed: {error}",
        de: "Update fehlgeschlagen: {error}",
        fr: "Échec de la mise à jour : {error}",
        es: "Error en la actualización: {error}",
        it: "Aggiornamento non riuscito: {error}",
        pt: "Falha na atualização: {error}",
        nl: "Update mislukt: {error}",
        pl: "Aktualizacja nie powiodła się: {error}",
        sv: "Uppdateringen misslyckades: {error}",
    },
    /// {error} an error message.
    update_check_failed {
        en: "Update check failed: {error}",
        de: "Update-Prüfung fehlgeschlagen: {error}",
        fr: "Échec de la vérification des mises à jour : {error}",
        es: "Error al comprobar las actualizaciones: {error}",
        it: "Verifica degli aggiornamenti non riuscita: {error}",
        pt: "Falha na verificação de atualização: {error}",
        nl: "Updatecontrole mislukt: {error}",
        pl: "Sprawdzanie aktualizacji nie powiodło się: {error}",
        sv: "Uppdateringskontrollen misslyckades: {error}",
    },
    /// Self-update compiled out. Keep the backticked flag verbatim.
    update_disabled {
        en: "Self-update support is disabled in this build. Rebuild with `--features update`.",
        de: "Die Selbst-Update-Funktion ist in diesem Build deaktiviert. Neu bauen mit `--features update`.",
        fr: "La fonction d'auto-mise à jour est désactivée dans cette version. Recompilez avec `--features update`.",
        es: "La función de autoactualización está desactivada en esta compilación. Recompila con `--features update`.",
        it: "Il supporto all'auto-aggiornamento è disabilitato in questa build. Ricompila con `--features update`.",
        pt: "O suporte a autoatualização está desativado nesta compilação. Recompile com `--features update`.",
        nl: "Zelf-updateondersteuning is uitgeschakeld in deze build. Bouw opnieuw met `--features update`.",
        pl: "Funkcja samoaktualizacji jest wyłączona w tej kompilacji. Skompiluj ponownie z `--features update`.",
        sv: "Stöd för självuppdatering är inaktiverat i detta bygge. Bygg om med `--features update`.",
    },
    /// Title of the `list` command.
    list_title {
        en: "Supported output formats",
        de: "Unterstützte Ausgabeformate",
        fr: "Formats de sortie pris en charge",
        es: "Formatos de salida compatibles",
        it: "Formati di output supportati",
        pt: "Formatos de saída suportados",
        nl: "Ondersteunde uitvoerformaten",
        pl: "Obsługiwane formaty wyjściowe",
        sv: "Format som stöds för utdata",
    },
    /// Format category heading.
    group_structured {
        en: "Structured data",
        de: "Strukturierte Daten",
        fr: "Données structurées",
        es: "Datos estructurados",
        it: "Dati strutturati",
        pt: "Dados estruturados",
        nl: "Gestructureerde data",
        pl: "Dane strukturalne",
        sv: "Strukturerade data",
    },
    /// Format category heading.
    group_text {
        en: "Text & config",
        de: "Text & Konfiguration",
        fr: "Texte & configuration",
        es: "Texto y configuración",
        it: "Testo e configurazione",
        pt: "Texto e configuração",
        nl: "Tekst & configuratie",
        pl: "Tekst i konfiguracja",
        sv: "Text & konfiguration",
    },
    /// Format category heading.
    group_images {
        en: "Images",
        de: "Bilder",
        fr: "Images",
        es: "Imágenes",
        it: "Immagini",
        pt: "Imagens",
        nl: "Afbeeldingen",
        pl: "Obrazy",
        sv: "Bilder",
    },
    /// Format category heading (audio & video).
    group_av {
        en: "Audio & video",
        de: "Audio & Video",
        fr: "Audio & vidéo",
        es: "Audio y vídeo",
        it: "Audio e video",
        pt: "Áudio e vídeo",
        nl: "Audio & video",
        pl: "Audio i wideo",
        sv: "Ljud & video",
    },
    /// Format category heading.
    group_docs {
        en: "Documents",
        de: "Dokumente",
        fr: "Documents",
        es: "Documentos",
        it: "Documenti",
        pt: "Documentos",
        nl: "Documenten",
        pl: "Dokumenty",
        sv: "Dokument",
    },
    /// Format category heading.
    group_binary {
        en: "Binary & archives",
        de: "Binär & Archive",
        fr: "Binaire & archives",
        es: "Binario y archivos comprimidos",
        it: "Binari e archivi",
        pt: "Binários e arquivos compactados",
        nl: "Binair & archieven",
        pl: "Pliki binarne i archiwa",
        sv: "Binärt & arkiv",
    },
    /// Section title. Keep the backticked `field:type` and `--schema` verbatim.
    list_schema_title {
        en: "Schema field types (use as `field:type` in --schema)",
        de: "Schema-Feldtypen (Verwendung als `feld:typ` in --schema)",
        fr: "Types de champ de schéma (à utiliser comme `field:type` dans --schema)",
        es: "Tipos de campo de esquema (úsalos como `field:type` en --schema)",
        it: "Tipi di campo dello schema (da usare come `field:type` in --schema)",
        pt: "Tipos de campo de esquema (use como `field:type` em --schema)",
        nl: "Schema-veldtypen (gebruik als `field:type` in --schema)",
        pl: "Typy pól schematu (użycie jako `field:type` w --schema)",
        sv: "Schemats fälttyper (används som `field:type` i --schema)",
    },
    /// Section title. Keep `--locale` verbatim.
    list_locales_title {
        en: "Data locales (--locale)",
        de: "Daten-Sprachräume (--locale)",
        fr: "Régions linguistiques des données (--locale)",
        es: "Regiones de datos (--locale)",
        it: "Aree linguistiche dei dati (--locale)",
        pt: "Localidades de dados (--locale)",
        nl: "Datataalregio's (--locale)",
        pl: "Regiony danych (--locale)",
        sv: "Dataspråkområden (--locale)",
    },
    /// Section title. Keep `--lang` verbatim.
    list_langs_title {
        en: "Interface languages (--lang)",
        de: "Oberflächensprachen (--lang)",
        fr: "Langues de l'interface (--lang)",
        es: "Idiomas de la interfaz (--lang)",
        it: "Lingue dell'interfaccia (--lang)",
        pt: "Idiomas da interface (--lang)",
        nl: "Interfacetalen (--lang)",
        pl: "Języki interfejsu (--lang)",
        sv: "Gränssnittsspråk (--lang)",
    },
    /// Section title. Keep `--preset` verbatim.
    list_presets_title {
        en: "Schema presets (--preset)",
        de: "Schema-Presets (--preset)",
        fr: "Préréglages de schéma (--preset)",
        es: "Presets de esquema (--preset)",
        it: "Preset di schema (--preset)",
        pt: "Predefinições de esquema (--preset)",
        nl: "Schemapresets (--preset)",
        pl: "Szablony schematu (--preset)",
        sv: "Schemaförinställningar (--preset)",
    },
    /// Closing tip. Keep the backticked command verbatim.
    list_hint {
        en: "Tip: run any format with --help to see its options, e.g. `demodatagen json --help`.",
        de: "Tipp: Jedes Format mit --help aufrufen, um seine Optionen zu sehen, z. B. `demodatagen json --help`.",
        fr: "Astuce : exécutez n'importe quel format avec --help pour voir ses options, par ex. `demodatagen json --help`.",
        es: "Sugerencia: ejecuta cualquier formato con --help para ver sus opciones, p. ej. `demodatagen json --help`.",
        it: "Suggerimento: esegui qualsiasi formato con --help per vederne le opzioni, ad es. `demodatagen json --help`.",
        pt: "Dica: execute qualquer formato com --help para ver suas opções, por exemplo `demodatagen json --help`.",
        nl: "Tip: voer een formaat uit met --help om de opties te zien, bijv. `demodatagen json --help`.",
        pl: "Wskazówka: uruchom dowolny format z --help, aby zobaczyć jego opcje, np. `demodatagen json --help`.",
        sv: "Tips: kör valfritt format med --help för att se dess alternativ, t.ex. `demodatagen json --help`.",
    },
    /// Title of the `presets` command.
    presets_title {
        en: "Built-in schema presets",
        de: "Eingebaute Schema-Presets",
        fr: "Préréglages de schéma intégrés",
        es: "Presets de esquema integrados",
        it: "Preset di schema integrati",
        pt: "Predefinições de esquema integradas",
        nl: "Ingebouwde schemapresets",
        pl: "Wbudowane szablony schematu",
        sv: "Inbyggda schemaförinställningar",
    },
    /// Intro line for `presets`. Keep the backticked command pattern verbatim.
    presets_intro {
        en: "Use a preset instead of writing a schema by hand: `demodatagen <format> --preset <name>`.",
        de: "Nutze ein Preset, statt ein Schema von Hand zu schreiben: `demodatagen <format> --preset <name>`.",
        fr: "Utilisez un préréglage plutôt que d'écrire un schéma à la main : `demodatagen <format> --preset <name>`.",
        es: "Usa un preset en lugar de escribir un esquema a mano: `demodatagen <format> --preset <name>`.",
        it: "Usa un preset invece di scrivere uno schema a mano: `demodatagen <format> --preset <name>`.",
        pt: "Use uma predefinição em vez de escrever um esquema à mão: `demodatagen <format> --preset <name>`.",
        nl: "Gebruik een preset in plaats van een schema met de hand te schrijven: `demodatagen <format> --preset <name>`.",
        pl: "Użyj szablonu zamiast pisać schemat ręcznie: `demodatagen <format> --preset <name>`.",
        sv: "Använd en förinställning istället för att skriva ett schema för hand: `demodatagen <format> --preset <name>`.",
    },
    /// Closing tip for `presets`. Keep backticked flags verbatim.
    presets_hint {
        en: "Works with any structured format (json, csv, sql, yaml, …); combine with --rows and --locale.",
        de: "Funktioniert mit jedem strukturierten Format (json, csv, sql, yaml, …); kombinierbar mit --rows und --locale.",
        fr: "Fonctionne avec tout format structuré (json, csv, sql, yaml, …) ; combinable avec --rows et --locale.",
        es: "Funciona con cualquier formato estructurado (json, csv, sql, yaml, …); combínalo con --rows y --locale.",
        it: "Funziona con qualsiasi formato strutturato (json, csv, sql, yaml, …); combinabile con --rows e --locale.",
        pt: "Funciona com qualquer formato estruturado (json, csv, sql, yaml, …); combine com --rows e --locale.",
        nl: "Werkt met elk gestructureerd formaat (json, csv, sql, yaml, …); te combineren met --rows en --locale.",
        pl: "Działa z każdym formatem strukturalnym (json, csv, sql, yaml, …); można łączyć z --rows i --locale.",
        sv: "Fungerar med alla strukturerade format (json, csv, sql, yaml, …); kombineras med --rows och --locale.",
    },
    /// Inline label preceding a preset's schema string in `presets` output.
    preset_schema_label {
        en: "Schema:",
        de: "Schema:",
        fr: "Schéma :",
        es: "Esquema:",
        it: "Schema:",
        pt: "Esquema:",
        nl: "Schema:",
        pl: "Schemat:",
        sv: "Schema:",
    },
    /// One-line description of the 'users' preset.
    preset_desc_users {
        en: "User accounts with names, emails, usernames and signup dates.",
        de: "Benutzerkonten mit Namen, E-Mails, Benutzernamen und Registrierungsdaten.",
        fr: "Comptes utilisateurs avec noms, e-mails, identifiants et dates d'inscription.",
        es: "Cuentas de usuario con nombres, correos electrónicos, nombres de usuario y fechas de registro.",
        it: "Account utente con nomi, email, nomi utente e date di registrazione.",
        pt: "Contas de usuário com nomes, e-mails, nomes de usuário e datas de cadastro.",
        nl: "Gebruikersaccounts met namen, e-mails, gebruikersnamen en registratiedatums.",
        pl: "Konta użytkowników z nazwiskami, adresami e-mail, nazwami użytkowników i datami rejestracji.",
        sv: "Användarkonton med namn, e-postadresser, användarnamn och registreringsdatum.",
    },
    /// One-line description of the 'employees' preset.
    preset_desc_employees {
        en: "HR records: people, departments, job titles and salaries.",
        de: "Personaldaten: Personen, Abteilungen, Jobtitel und Gehälter.",
        fr: "Données RH : personnes, services, intitulés de poste et salaires.",
        es: "Datos de RR. HH.: personas, departamentos, puestos de trabajo y salarios.",
        it: "Dati HR: persone, reparti, qualifiche e stipendi.",
        pt: "Registros de RH: pessoas, departamentos, cargos e salários.",
        nl: "Personeelsgegevens: personen, afdelingen, functietitels en salarissen.",
        pl: "Dane kadrowe: osoby, działy, stanowiska i wynagrodzenia.",
        sv: "Personaldata: personer, avdelningar, jobbtitlar och löner.",
    },
    /// One-line description of the 'customers' preset.
    preset_desc_customers {
        en: "Customers with contact details, addresses and phone numbers.",
        de: "Kunden mit Kontaktdaten, Adressen und Telefonnummern.",
        fr: "Clients avec coordonnées, adresses et numéros de téléphone.",
        es: "Clientes con datos de contacto, direcciones y números de teléfono.",
        it: "Clienti con dati di contatto, indirizzi e numeri di telefono.",
        pt: "Clientes com dados de contato, endereços e números de telefone.",
        nl: "Klanten met contactgegevens, adressen en telefoonnummers.",
        pl: "Klienci z danymi kontaktowymi, adresami i numerami telefonów.",
        sv: "Kunder med kontaktuppgifter, adresser och telefonnummer.",
    },
    /// One-line description of the 'products' preset.
    preset_desc_products {
        en: "Catalog products with SKU, price, category and rating.",
        de: "Katalogprodukte mit SKU, Preis, Kategorie und Bewertung.",
        fr: "Produits de catalogue avec référence (SKU), prix, catégorie et note.",
        es: "Productos de catálogo con SKU, precio, categoría y valoración.",
        it: "Prodotti a catalogo con SKU, prezzo, categoria e valutazione.",
        pt: "Produtos de catálogo com SKU, preço, categoria e avaliação.",
        nl: "Catalogusproducten met SKU, prijs, categorie en beoordeling.",
        pl: "Produkty katalogowe z SKU, ceną, kategorią i oceną.",
        sv: "Katalogprodukter med SKU, pris, kategori och betyg.",
    },
    /// One-line description of the 'orders' preset.
    preset_desc_orders {
        en: "E-commerce orders with customer, product, quantity and total.",
        de: "E-Commerce-Bestellungen mit Kunde, Produkt, Menge und Summe.",
        fr: "Commandes e-commerce avec client, produit, quantité et total.",
        es: "Pedidos de comercio electrónico con cliente, producto, cantidad y total.",
        it: "Ordini e-commerce con cliente, prodotto, quantità e totale.",
        pt: "Pedidos de e-commerce com cliente, produto, quantidade e total.",
        nl: "E-commercebestellingen met klant, product, aantal en totaal.",
        pl: "Zamówienia e-commerce z klientem, produktem, ilością i sumą.",
        sv: "E-handelsbeställningar med kund, produkt, antal och summa.",
    },
    /// One-line description of the 'transactions' preset.
    preset_desc_transactions {
        en: "Financial transactions with amount, currency, IBAN and status.",
        de: "Finanztransaktionen mit Betrag, Währung, IBAN und Status.",
        fr: "Transactions financières avec montant, devise, IBAN et statut.",
        es: "Transacciones financieras con importe, moneda, IBAN y estado.",
        it: "Transazioni finanziarie con importo, valuta, IBAN e stato.",
        pt: "Transações financeiras com valor, moeda, IBAN e status.",
        nl: "Financiële transacties met bedrag, valuta, IBAN en status.",
        pl: "Transakcje finansowe z kwotą, walutą, IBAN i statusem.",
        sv: "Finansiella transaktioner med belopp, valuta, IBAN och status.",
    },
    /// One-line description of the 'events' preset.
    preset_desc_events {
        en: "Analytics events with type, timestamp, user and session id.",
        de: "Analyse-Ereignisse mit Typ, Zeitstempel, Nutzer und Sitzungs-ID.",
        fr: "Événements d'analyse avec type, horodatage, utilisateur et ID de session.",
        es: "Eventos de analítica con tipo, marca de tiempo, usuario e ID de sesión.",
        it: "Eventi di analisi con tipo, timestamp, utente e ID sessione.",
        pt: "Eventos de análise com tipo, carimbo de data/hora, usuário e ID de sessão.",
        nl: "Analyse-gebeurtenissen met type, tijdstempel, gebruiker en sessie-ID.",
        pl: "Zdarzenia analityczne z typem, znacznikiem czasu, użytkownikiem i identyfikatorem sesji.",
        sv: "Analyshändelser med typ, tidsstämpel, användare och sessions-ID.",
    },
    /// One-line description of the 'servers' preset.
    preset_desc_servers {
        en: "Infrastructure hosts with hostnames, IPs, ports and uptime.",
        de: "Infrastruktur-Hosts mit Hostnamen, IPs, Ports und Laufzeit.",
        fr: "Hôtes d'infrastructure avec noms d'hôte, IP, ports et temps de fonctionnement.",
        es: "Hosts de infraestructura con nombres de host, IP, puertos y tiempo de actividad.",
        it: "Host infrastrutturali con hostname, IP, porte e tempo di attività.",
        pt: "Hosts de infraestrutura com nomes de host, IPs, portas e tempo de atividade.",
        nl: "Infrastructuurhosts met hostnamen, IP's, poorten en uptime.",
        pl: "Hosty infrastruktury z nazwami hostów, adresami IP, portami i czasem działania.",
        sv: "Infrastrukturvärdar med värdnamn, IP-adresser, portar och drifttid.",
    },
    /// One-line description of the 'geo' preset.
    preset_desc_geo {
        en: "Geographic points with latitude, longitude, city and country.",
        de: "Geografische Punkte mit Breitengrad, Längengrad, Stadt und Land.",
        fr: "Points géographiques avec latitude, longitude, ville et pays.",
        es: "Puntos geográficos con latitud, longitud, ciudad y país.",
        it: "Punti geografici con latitudine, longitudine, città e paese.",
        pt: "Pontos geográficos com latitude, longitude, cidade e país.",
        nl: "Geografische punten met breedtegraad, lengtegraad, stad en land.",
        pl: "Punkty geograficzne z szerokością i długością geograficzną, miastem i krajem.",
        sv: "Geografiska punkter med latitud, longitud, stad och land.",
    },
    /// One-line description of the 'posts' preset.
    preset_desc_posts {
        en: "Blog or social posts with author, title, body and tags.",
        de: "Blog- oder Social-Media-Beiträge mit Autor, Titel, Text und Tags.",
        fr: "Publications de blog ou réseaux sociaux avec auteur, titre, texte et tags.",
        es: "Publicaciones de blog o redes sociales con autor, título, cuerpo y etiquetas.",
        it: "Post di blog o social con autore, titolo, testo e tag.",
        pt: "Posts de blog ou redes sociais com autor, título, texto e tags.",
        nl: "Blog- of socialmediaberichten met auteur, titel, tekst en tags.",
        pl: "Wpisy blogowe lub w mediach społecznościowych z autorem, tytułem, treścią i tagami.",
        sv: "Blogg- eller sociala inlägg med författare, titel, text och taggar.",
    },
    /// One-line description of the 'payments' preset.
    preset_desc_payments {
        en: "Payment records with method, card, amount and currency.",
        de: "Zahlungsdatensätze mit Methode, Karte, Betrag und Währung.",
        fr: "Enregistrements de paiement avec méthode, carte, montant et devise.",
        es: "Registros de pago con método, tarjeta, importe y moneda.",
        it: "Dati di pagamento con metodo, carta, importo e valuta.",
        pt: "Registros de pagamento com método, cartão, valor e moeda.",
        nl: "Betalingsgegevens met methode, kaart, bedrag en valuta.",
        pl: "Rekordy płatności z metodą, kartą, kwotą i walutą.",
        sv: "Betalningsposter med metod, kort, belopp och valuta.",
    },
    /// One-line description of the 'sensors' preset.
    preset_desc_sensors {
        en: "IoT sensor readings with device id, metric, value and time.",
        de: "IoT-Sensormesswerte mit Geräte-ID, Metrik, Wert und Zeit.",
        fr: "Relevés de capteurs IoT avec ID d'appareil, métrique, valeur et heure.",
        es: "Lecturas de sensores IoT con ID de dispositivo, métrica, valor y hora.",
        it: "Letture di sensori IoT con ID dispositivo, metrica, valore e ora.",
        pt: "Leituras de sensores IoT com ID de dispositivo, métrica, valor e hora.",
        nl: "IoT-sensormetingen met apparaat-ID, metriek, waarde en tijd.",
        pl: "Odczyty czujników IoT z identyfikatorem urządzenia, metryką, wartością i czasem.",
        sv: "IoT-sensormätvärden med enhets-ID, mätvärde, värde och tid.",
    },
    /// Title of the `info` command panel.
    info_title {
        en: "Environment & build information",
        de: "Umgebungs- & Build-Informationen",
        fr: "Informations sur l'environnement & la compilation",
        es: "Información del entorno y de la compilación",
        it: "Informazioni su ambiente e build",
        pt: "Informações de ambiente e compilação",
        nl: "Omgevings- & build-informatie",
        pl: "Informacje o środowisku i kompilacji",
        sv: "Miljö- & bygginformation",
    },
    /// Row label for the program version.
    info_version {
        en: "Version",
        de: "Version",
        fr: "Version",
        es: "Versión",
        it: "Versione",
        pt: "Versão",
        nl: "Versie",
        pl: "Wersja",
        sv: "Version",
    },
    /// Row label for the compilation target triple (e.g. x86_64-unknown-linux-gnu).
    info_build_target {
        en: "Build target",
        de: "Build-Ziel",
        fr: "Cible de compilation",
        es: "Destino de compilación",
        it: "Target di build",
        pt: "Alvo de compilação",
        nl: "Build-doel",
        pl: "Cel kompilacji",
        sv: "Byggmål",
    },
    /// Row label for the build profile (debug/release).
    info_profile {
        en: "Build profile",
        de: "Build-Profil",
        fr: "Profil de compilation",
        es: "Perfil de compilación",
        it: "Profilo di build",
        pt: "Perfil de compilação",
        nl: "Build-profiel",
        pl: "Profil kompilacji",
        sv: "Byggprofil",
    },
    /// Row label for the number of output formats.
    info_formats {
        en: "Output formats",
        de: "Ausgabeformate",
        fr: "Formats de sortie",
        es: "Formatos de salida",
        it: "Formati di output",
        pt: "Formatos de saída",
        nl: "Uitvoerformaten",
        pl: "Formaty wyjściowe",
        sv: "Utdataformat",
    },
    /// Row label for the number of data locales.
    info_locales {
        en: "Data locales",
        de: "Daten-Sprachräume",
        fr: "Régions linguistiques des données",
        es: "Regiones de datos",
        it: "Aree linguistiche dei dati",
        pt: "Localidades de dados",
        nl: "Datataalregio's",
        pl: "Regiony danych",
        sv: "Dataspråkområden",
    },
    /// Row label for the number of interface languages.
    info_languages {
        en: "Interface languages",
        de: "Oberflächensprachen",
        fr: "Langues de l'interface",
        es: "Idiomas de la interfaz",
        it: "Lingue dell'interfaccia",
        pt: "Idiomas da interface",
        nl: "Interfacetalen",
        pl: "Języki interfejsu",
        sv: "Gränssnittsspråk",
    },
    /// Row label for the number of schema presets.
    info_presets {
        en: "Schema presets",
        de: "Schema-Presets",
        fr: "Préréglages de schéma",
        es: "Presets de esquema",
        it: "Preset di schema",
        pt: "Predefinições de esquema",
        nl: "Schemapresets",
        pl: "Szablony schematu",
        sv: "Schemaförinställningar",
    },
    /// Row label for the number of worker threads.
    info_threads {
        en: "Worker threads",
        de: "Worker-Threads",
        fr: "Threads de travail",
        es: "Hilos de trabajo",
        it: "Thread di lavoro",
        pt: "Threads de trabalho",
        nl: "Worker-threads",
        pl: "Wątki robocze",
        sv: "Arbetstrådar",
    },
    /// Row label for whether self-update is compiled in.
    info_update_feature {
        en: "Self-update",
        de: "Selbst-Update",
        fr: "Auto-mise à jour",
        es: "Autoactualización",
        it: "Auto-aggiornamento",
        pt: "Autoatualização",
        nl: "Zelf-update",
        pl: "Samoaktualizacja",
        sv: "Självuppdatering",
    },
    /// Row label for the source repository URL.
    info_repository {
        en: "Repository",
        de: "Repository",
        fr: "Dépôt",
        es: "Repositorio",
        it: "Repository",
        pt: "Repositório",
        nl: "Repository",
        pl: "Repozytorium",
        sv: "Repository",
    },
    /// Row label for the software license.
    info_license {
        en: "License",
        de: "Lizenz",
        fr: "Licence",
        es: "Licencia",
        it: "Licenza",
        pt: "Licença",
        nl: "Licentie",
        pl: "Licencja",
        sv: "Licens",
    },
    /// Value meaning a feature is on.
    info_enabled {
        en: "enabled",
        de: "aktiviert",
        fr: "activé",
        es: "activada",
        it: "abilitato",
        pt: "ativado",
        nl: "ingeschakeld",
        pl: "włączona",
        sv: "aktiverad",
    },
    /// Value meaning a feature is off.
    info_disabled {
        en: "disabled",
        de: "deaktiviert",
        fr: "désactivé",
        es: "desactivada",
        it: "disabilitato",
        pt: "desativado",
        nl: "uitgeschakeld",
        pl: "wyłączona",
        sv: "inaktiverad",
    },
    /// Closing tip for `info`. Keep the backticked command verbatim.
    info_hint {
        en: "Run `demodatagen list` for formats, schema types, locales and languages.",
        de: "Führe `demodatagen list` aus für Formate, Schema-Typen, Sprachräume und Sprachen.",
        fr: "Exécutez `demodatagen list` pour les formats, types de schéma, régions linguistiques et langues.",
        es: "Ejecuta `demodatagen list` para ver formatos, tipos de esquema, regiones e idiomas.",
        it: "Esegui `demodatagen list` per formati, tipi di schema, aree linguistiche e lingue.",
        pt: "Execute `demodatagen list` para ver formatos, tipos de esquema, localidades e idiomas.",
        nl: "Voer `demodatagen list` uit voor formaten, schematypen, taalregio's en talen.",
        pl: "Uruchom `demodatagen list`, aby zobaczyć formaty, typy schematu, regiony i języki.",
        sv: "Kör `demodatagen list` för format, schematyper, språkområden och språk.",
    },
    /// Header for a --dry-run. Make clear nothing is written.
    dryrun_header {
        en: "Dry run — planning only, no files will be written.",
        de: "Probelauf — nur Planung, es werden keine Dateien geschrieben.",
        fr: "Simulation — planification uniquement, aucun fichier ne sera écrit.",
        es: "Simulación — solo planificación, no se escribirá ningún archivo.",
        it: "Prova a vuoto — solo pianificazione, non verrà scritto alcun file.",
        pt: "Execução simulada — apenas planejamento, nenhum arquivo será gravado.",
        nl: "Proefrun — alleen plannen, er worden geen bestanden geschreven.",
        pl: "Próbny przebieg — tylko planowanie, żadne pliki nie zostaną zapisane.",
        sv: "Provkörning — endast planering, inga filer skrivs.",
    },
    /// {count} number, {format} format name, {dir} path. Keep × and arrow.
    dryrun_plan {
        en: "Would generate {count} × {format} → {dir}",
        de: "Würde erzeugen: {count} × {format} → {dir}",
        fr: "Générerait {count} × {format} → {dir}",
        es: "Se generaría: {count} × {format} → {dir}",
        it: "Verrebbero generati {count} × {format} → {dir}",
        pt: "Geraria {count} × {format} → {dir}",
        nl: "Zou genereren: {count} × {format} → {dir}",
        pl: "Zostałoby wygenerowane: {count} × {format} → {dir}",
        sv: "Skulle generera {count} × {format} → {dir}",
    },
    /// Heading above the list of planned file names.
    dryrun_files_title {
        en: "Planned files:",
        de: "Geplante Dateien:",
        fr: "Fichiers prévus :",
        es: "Archivos planificados:",
        it: "File pianificati:",
        pt: "Arquivos planejados:",
        nl: "Geplande bestanden:",
        pl: "Zaplanowane pliki:",
        sv: "Planerade filer:",
    },
    /// Shown after a truncated file list. {count} is how many more files.
    dryrun_more {
        en: "… and {count} more",
        de: "… und {count} weitere",
        fr: "… et {count} de plus",
        es: "… y {count} más",
        it: "… e altri {count}",
        pt: "… e mais {count}",
        nl: "… en nog {count} meer",
        pl: "… i {count} więcej",
        sv: "… och {count} till",
    },
    /// Closing line of a dry run. {count} number of files planned.
    dryrun_done {
        en: "Dry run complete — {count} file(s) planned.",
        de: "Probelauf abgeschlossen — {count} Datei(en) geplant.",
        fr: "Simulation terminée — {count} fichier(s) prévu(s).",
        es: "Simulación completada — {count} archivo(s) planificado(s).",
        it: "Prova a vuoto completata — {count} file pianificati.",
        pt: "Execução simulada concluída — {count} arquivo(s) planejado(s).",
        nl: "Proefrun voltooid — {count} bestand(en) gepland.",
        pl: "Próbny przebieg zakończony — zaplanowano {count} plik(ów).",
        sv: "Provkörning slutförd — {count} fil(er) planerade.",
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
    /// Italian.
    It,
    /// Portuguese.
    Pt,
    /// Dutch.
    Nl,
    /// Polish.
    Pl,
    /// Swedish.
    Sv,
}

impl Language {
    /// Returns the immutable message catalog for this language.
    pub fn catalog(self) -> &'static Catalog {
        match self {
            Language::En => &EN_CATALOG,
            Language::De => &DE_CATALOG,
            Language::Fr => &FR_CATALOG,
            Language::Es => &ES_CATALOG,
            Language::It => &IT_CATALOG,
            Language::Pt => &PT_CATALOG,
            Language::Nl => &NL_CATALOG,
            Language::Pl => &PL_CATALOG,
            Language::Sv => &SV_CATALOG,
        }
    }

    /// Returns the canonical lowercase identifier (e.g. `"en"`).
    pub fn as_str(self) -> &'static str {
        match self {
            Language::En => "en",
            Language::De => "de",
            Language::Fr => "fr",
            Language::Es => "es",
            Language::It => "it",
            Language::Pt => "pt",
            Language::Nl => "nl",
            Language::Pl => "pl",
            Language::Sv => "sv",
        }
    }

    /// Returns a human-readable, endonymic label (e.g. `"English"`, `"Deutsch"`).
    pub fn label(self) -> &'static str {
        match self {
            Language::En => "English",
            Language::De => "Deutsch",
            Language::Fr => "Français",
            Language::Es => "Español",
            Language::It => "Italiano",
            Language::Pt => "Português",
            Language::Nl => "Nederlands",
            Language::Pl => "Polski",
            Language::Sv => "Svenska",
        }
    }

    /// Returns every supported language, in declaration order.
    pub fn variants() -> &'static [Language] {
        &[
            Language::En,
            Language::De,
            Language::Fr,
            Language::Es,
            Language::It,
            Language::Pt,
            Language::Nl,
            Language::Pl,
            Language::Sv,
        ]
    }

    /// Returns all canonical language identifiers, for help text and `list`.
    pub fn all() -> &'static [&'static str] {
        &["en", "de", "fr", "es", "it", "pt", "nl", "pl", "sv"]
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
            "de" | "de_de" | "de_at" | "de_ch" | "german" | "deutsch" => Ok(Language::De),
            "fr" | "fr_fr" | "fr_ca" | "french" | "francais" | "français" => Ok(Language::Fr),
            "es" | "es_es" | "es_mx" | "spanish" | "espanol" | "español" => Ok(Language::Es),
            "it" | "it_it" | "italian" | "italiano" => Ok(Language::It),
            "pt" | "pt_br" | "pt_pt" | "portuguese" | "portugues" | "português" => {
                Ok(Language::Pt)
            }
            "nl" | "nl_nl" | "nl_be" | "dutch" | "nederlands" => Ok(Language::Nl),
            "pl" | "pl_pl" | "polish" | "polski" => Ok(Language::Pl),
            "sv" | "sv_se" | "swedish" | "svenska" => Ok(Language::Sv),
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
/// let s = tr!(Language::En, update_up_to_date, "version" => "0.5.0");
/// assert_eq!(s, "demodatagen is up to date (v0.5.0).");
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
        assert_eq!("it".parse::<Language>().unwrap(), Language::It);
        assert_eq!("pt_BR".parse::<Language>().unwrap(), Language::Pt);
        assert_eq!("Nederlands".parse::<Language>().unwrap(), Language::Nl);
        assert_eq!("pl".parse::<Language>().unwrap(), Language::Pl);
        assert_eq!("svenska".parse::<Language>().unwrap(), Language::Sv);
        assert!("xx".parse::<Language>().is_err());
    }

    #[test]
    fn test_roundtrip() {
        for id in Language::all() {
            assert_eq!(id.parse::<Language>().unwrap().as_str(), *id);
        }
        assert_eq!(Language::variants().len(), Language::all().len());
        assert_eq!(Language::variants().len(), 9);
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
    fn test_every_language_has_a_distinct_label() {
        use std::collections::HashSet;
        let labels: HashSet<&str> = Language::variants().iter().map(|l| l.label()).collect();
        assert_eq!(labels.len(), Language::variants().len());
    }

    #[test]
    fn test_all_catalogs_are_complete_and_nonempty() {
        // Every language must define every key with a non-blank string. The set
        // of keys is derived from the catalog itself via `fields()`, so adding a
        // message can never silently skip a language.
        let key_count = EN_CATALOG.fields().len();
        assert!(
            key_count >= 70,
            "expected a sizable catalog, got {key_count}"
        );
        for lang in Language::variants() {
            let fields = lang.catalog().fields();
            assert_eq!(fields.len(), key_count, "{lang} has a different key count");
            for (i, field) in fields.iter().enumerate() {
                assert!(
                    !field.trim().is_empty(),
                    "{lang} has an empty message at index {i}"
                );
            }
        }
    }

    #[test]
    fn test_placeholders_consistent_across_languages() {
        // Each translation must use exactly the same `{placeholder}` set as the
        // English source, so `fill` never leaves a dangling slot or drops a value.
        fn placeholders(s: &str) -> std::collections::BTreeSet<String> {
            let mut out = std::collections::BTreeSet::new();
            let bytes = s.as_bytes();
            let mut i = 0;
            while i < bytes.len() {
                if bytes[i] == b'{' {
                    if let Some(end) = s[i..].find('}') {
                        out.insert(s[i..i + end + 1].to_string());
                        i += end + 1;
                        continue;
                    }
                }
                i += 1;
            }
            out
        }
        let en = EN_CATALOG.fields();
        for lang in Language::variants() {
            let fields = lang.catalog().fields();
            for (i, (en_field, tr_field)) in en.iter().zip(fields.iter()).enumerate() {
                assert_eq!(
                    placeholders(en_field),
                    placeholders(tr_field),
                    "{lang} placeholder mismatch at key index {i}: '{en_field}' vs '{tr_field}'"
                );
            }
        }
    }
}
