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
//! translated into **fifteen languages**: English, German, French, Spanish,
//! Italian, Portuguese, Dutch, Polish, Swedish, Czech, Danish, Finnish,
//! Norwegian Bokmål, Turkish, and Japanese.
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
            sv: $sv:literal,
            cs: $cs:literal,
            da: $da:literal,
            fi: $fi:literal,
            nb: $nb:literal,
            tr: $tr:literal,
            ja: $ja:literal $(,)?
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
        const CS_CATALOG: Catalog = Catalog { $( $key: $cs, )+ };
        const DA_CATALOG: Catalog = Catalog { $( $key: $da, )+ };
        const FI_CATALOG: Catalog = Catalog { $( $key: $fi, )+ };
        const NB_CATALOG: Catalog = Catalog { $( $key: $nb, )+ };
        const TR_CATALOG: Catalog = Catalog { $( $key: $tr, )+ };
        const JA_CATALOG: Catalog = Catalog { $( $key: $ja, )+ };
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
        cs: "Realistické ukázkové soubory v {formats} formátech a {locales} jazykových oblastech — offline, deterministické, bez externích závislostí.",
        da: "Realistiske demofiler i {formats} formater og {locales} sprogområder — offline, deterministiske, uden eksterne afhængigheder.",
        fi: "Realistisia demotiedostoja {formats} muodossa ja {locales} kielialueella — offline, determinististä, ilman ulkoisia riippuvuuksia.",
        nb: "Realistiske demofiler i {formats} formater og {locales} språkområder — frakoblet, deterministisk, uten eksterne avhengigheter.",
        tr: "{formats} biçimde ve {locales} yerel ayarda gerçekçi demo dosyaları — çevrimdışı, deterministik, harici bağımlılık yok.",
        ja: "{formats} 種類の形式と {locales} のロケールに対応した現実的なデモファイル — オフライン、決定論的、外部依存なし。",
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
        cs: "Generuji {count} × {format} → {dir}",
        da: "Genererer {count} × {format} → {dir}",
        fi: "Luodaan {count} × {format} → {dir}",
        nb: "Genererer {count} × {format} → {dir}",
        tr: "{count} × {format} oluşturuluyor → {dir}",
        ja: "{count} × {format} を生成中 → {dir}",
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
        cs: "zapisuji {format}",
        da: "skriver {format}",
        fi: "kirjoitetaan {format}",
        nb: "skriver {format}",
        tr: "{format} yazılıyor",
        ja: "{format} を書き込み中",
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
        cs: "hotovo",
        da: "færdig",
        fi: "valmis",
        nb: "ferdig",
        tr: "tamamlandı",
        ja: "完了",
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
        cs: "Vygenerováno {count} souborů · {bytes} · {elapsed}",
        da: "{count} fil(er) genereret · {bytes} · {elapsed}",
        fi: "{count} tiedosto(a) luotu · {bytes} · {elapsed}",
        nb: "{count} fil(er) generert · {bytes} · {elapsed}",
        tr: "{count} dosya oluşturuldu · {bytes} · {elapsed}",
        ja: "{count} 件のファイルを生成しました · {bytes} · {elapsed}",
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
        cs: "Výstupní adresář: {dir}",
        da: "Outputmappe: {dir}",
        fi: "Tuloshakemisto: {dir}",
        nb: "Utdatakatalog: {dir}",
        tr: "Çıktı dizini: {dir}",
        ja: "出力ディレクトリ: {dir}",
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
        cs: "Vygenerováno {ok}/{total} souborů — {errors} chyb(y)",
        da: "{ok}/{total} fil(er) genereret — {errors} fejl",
        fi: "{ok}/{total} tiedostoa luotu — {errors} virhettä",
        nb: "{ok}/{total} fil(er) generert — {errors} feil",
        tr: "{ok}/{total} dosya oluşturuldu — {errors} hata",
        ja: "{ok}/{total} 件のファイルを生成しました — エラー {errors} 件",
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
        cs: "Nepodařilo se vygenerovat žádný z {total} souborů. První chyba: {error}",
        da: "Alle {total} fil(er) kunne ikke genereres. Første fejl: {error}",
        fi: "Kaikkien {total} tiedoston luonti epäonnistui. Ensimmäinen virhe: {error}",
        nb: "Alle {total} fil(er) kunne ikke genereres. Første feil: {error}",
        tr: "{total} dosyanın tümü oluşturulamadı. İlk hata: {error}",
        ja: "{total} 件のファイルすべての生成に失敗しました。最初のエラー: {error}",
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
        cs: "Neznámý formát: {format}",
        da: "Ukendt format: {format}",
        fi: "Tuntematon muoto: {format}",
        nb: "Ukjent format: {format}",
        tr: "Bilinmeyen biçim: {format}",
        ja: "不明な形式です: {format}",
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
        cs: "Generování se nezdařilo: {error}",
        da: "Generering mislykkedes: {error}",
        fi: "Luonti epäonnistui: {error}",
        nb: "Generering mislyktes: {error}",
        tr: "Oluşturma başarısız oldu: {error}",
        ja: "生成に失敗しました: {error}",
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
        cs: "Neznámá předvolba: {preset}. Spusťte `demodatagen presets` pro výpis vestavěných předvoleb.",
        da: "Ukendt forudindstilling: {preset}. Kør `demodatagen presets` for at vise de indbyggede forudindstillinger.",
        fi: "Tuntematon esiasetus: {preset}. Suorita `demodatagen presets` nähdäksesi sisäänrakennetut esiasetukset.",
        nb: "Ukjent forhåndsinnstilling: {preset}. Kjør `demodatagen presets` for å liste de innebygde forhåndsinnstillingene.",
        tr: "Bilinmeyen ön ayar: {preset}. Yerleşik ön ayarları listelemek için `demodatagen presets` komutunu çalıştırın.",
        ja: "不明なプリセットです: {preset}。`demodatagen presets` を実行すると組み込みプリセットの一覧が表示されます。",
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
        cs: "{error} Použije se výchozí hodnota '{default}'.",
        da: "{error} Bruger standardværdien '{default}' i stedet.",
        fi: "{error} Käytetään oletusarvoa '{default}'.",
        nb: "{error} Bruker standardverdien '{default}' i stedet.",
        tr: "{error} Bunun yerine varsayılan '{default}' kullanılıyor.",
        ja: "{error} 既定値 '{default}' を代わりに使用します。",
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
        cs: "Neznámý typ schématu '{type}'; mysleli jste '{suggestion}'? Místo toho generuji obecné slovo.",
        da: "Ukendt skematype '{type}'; mente du '{suggestion}'? Genererer et generisk ord i stedet.",
        fi: "Tuntematon skeematyyppi '{type}'; tarkoititko '{suggestion}'? Luodaan sen sijaan yleinen sana.",
        nb: "Ukjent skjematype '{type}'; mente du '{suggestion}'? Genererer et generisk ord i stedet.",
        tr: "Bilinmeyen şema türü '{type}'; '{suggestion}' mı demek istediniz? Bunun yerine genel bir sözcük oluşturuluyor.",
        ja: "不明なスキーマ型 '{type}' です。'{suggestion}' のことでしょうか？代わりに汎用的な単語を生成します。",
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
        cs: "Neznámý typ schématu '{type}'; místo toho generuji obecné slovo.",
        da: "Ukendt skematype '{type}'; genererer et generisk ord i stedet.",
        fi: "Tuntematon skeematyyppi '{type}'; luodaan sen sijaan yleinen sana.",
        nb: "Ukjent skjematype '{type}'; genererer et generisk ord i stedet.",
        tr: "Bilinmeyen şema türü '{type}'; bunun yerine genel bir sözcük oluşturuluyor.",
        ja: "不明なスキーマ型 '{type}' です。代わりに汎用的な単語を生成します。",
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
        cs: "Kontroluji aktualizace (aktuální: v{version})…",
        da: "Søger efter opdateringer (nuværende: v{version})…",
        fi: "Tarkistetaan päivityksiä (nykyinen: v{version})…",
        nb: "Ser etter oppdateringer (nåværende: v{version})…",
        tr: "Güncellemeler denetleniyor (geçerli: v{version})…",
        ja: "更新を確認しています（現在: v{version}）…",
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
        cs: "Je k dispozici aktualizace: v{current} → v{latest}",
        da: "Opdatering tilgængelig: v{current} → v{latest}",
        fi: "Päivitys saatavilla: v{current} → v{latest}",
        nb: "Oppdatering tilgjengelig: v{current} → v{latest}",
        tr: "Güncelleme mevcut: v{current} → v{latest}",
        ja: "更新があります: v{current} → v{latest}",
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
        cs: "Spusťte `demodatagen update` pro aktualizaci, nebo si ji stáhněte z {url}",
        da: "Kør `demodatagen update` for at opgradere, eller download fra {url}",
        fi: "Suorita `demodatagen update` päivittääksesi tai lataa osoitteesta {url}",
        nb: "Kjør `demodatagen update` for å oppgradere, eller last ned fra {url}",
        tr: "Yükseltmek için `demodatagen update` komutunu çalıştırın veya {url} adresinden indirin",
        ja: "`demodatagen update` を実行して更新するか、{url} からダウンロードしてください",
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
        cs: "demodatagen je aktuální (v{version}).",
        da: "demodatagen er opdateret (v{version}).",
        fi: "demodatagen on ajan tasalla (v{version}).",
        nb: "demodatagen er oppdatert (v{version}).",
        tr: "demodatagen güncel (v{version}).",
        ja: "demodatagen は最新です（v{version}）。",
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
        cs: "Nepodařilo se zjistit nejnovější verzi. Zkontrolujte {url} ručně.",
        da: "Kunne ikke bestemme den nyeste version. Tjek {url} manuelt.",
        fi: "Uusinta versiota ei voitu selvittää. Tarkista {url} manuaalisesti.",
        nb: "Kunne ikke fastslå den nyeste versjonen. Sjekk {url} manuelt.",
        tr: "En son sürüm belirlenemedi. {url} adresini elle kontrol edin.",
        ja: "最新バージョンを確認できませんでした。{url} を手動でご確認ください。",
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
        cs: "demodatagen aktualizován: v{from} → v{to}.",
        da: "demodatagen opdateret: v{from} → v{to}.",
        fi: "demodatagen päivitetty: v{from} → v{to}.",
        nb: "demodatagen oppdatert: v{from} → v{to}.",
        tr: "demodatagen güncellendi: v{from} → v{to}.",
        ja: "demodatagen を更新しました: v{from} → v{to}。",
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
        cs: "Spusťte příkaz znovu, aby se použila nová verze.",
        da: "Genstart kommandoen for at bruge den nye version.",
        fi: "Käynnistä komento uudelleen käyttääksesi uutta versiota.",
        nb: "Start kommandoen på nytt for å bruke den nye versjonen.",
        tr: "Yeni sürümü kullanmak için komutu yeniden başlatın.",
        ja: "新しいバージョンを使用するには、コマンドを再実行してください。",
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
        cs: "Již běží nejnovější verze (v{version}).",
        da: "Kører allerede den nyeste version (v{version}).",
        fi: "Uusin versio on jo käytössä (v{version}).",
        nb: "Kjører allerede den nyeste versjonen (v{version}).",
        tr: "Zaten en son sürüm çalışıyor (v{version}).",
        ja: "すでに最新バージョンを実行しています（v{version}）。",
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
        cs: "Aktualizace se nezdařila: {error}",
        da: "Opdatering mislykkedes: {error}",
        fi: "Päivitys epäonnistui: {error}",
        nb: "Oppdatering mislyktes: {error}",
        tr: "Güncelleme başarısız oldu: {error}",
        ja: "更新に失敗しました: {error}",
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
        cs: "Kontrola aktualizací se nezdařila: {error}",
        da: "Opdateringstjek mislykkedes: {error}",
        fi: "Päivitysten tarkistus epäonnistui: {error}",
        nb: "Oppdateringssjekk mislyktes: {error}",
        tr: "Güncelleme denetimi başarısız oldu: {error}",
        ja: "更新の確認に失敗しました: {error}",
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
        cs: "Podpora samoaktualizace je v tomto sestavení vypnutá. Sestavte znovu s `--features update`.",
        da: "Selvopdatering er deaktiveret i dette build. Byg igen med `--features update`.",
        fi: "Itsepäivitys on poistettu käytöstä tässä koontiversiossa. Käännä uudelleen valinnalla `--features update`.",
        nb: "Selvoppdatering er deaktivert i dette bygget. Bygg på nytt med `--features update`.",
        tr: "Bu derlemede kendi kendine güncelleme desteği devre dışı. `--features update` ile yeniden derleyin.",
        ja: "このビルドでは自己更新機能が無効になっています。`--features update` を付けて再ビルドしてください。",
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
        cs: "Podporované výstupní formáty",
        da: "Understøttede outputformater",
        fi: "Tuetut tulostemuodot",
        nb: "Støttede utdataformater",
        tr: "Desteklenen çıktı biçimleri",
        ja: "対応している出力形式",
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
        cs: "Strukturovaná data",
        da: "Strukturerede data",
        fi: "Rakenteinen data",
        nb: "Strukturerte data",
        tr: "Yapılandırılmış veri",
        ja: "構造化データ",
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
        cs: "Text a konfigurace",
        da: "Tekst & konfiguration",
        fi: "Teksti ja asetukset",
        nb: "Tekst & konfigurasjon",
        tr: "Metin ve yapılandırma",
        ja: "テキストと設定",
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
        cs: "Obrázky",
        da: "Billeder",
        fi: "Kuvat",
        nb: "Bilder",
        tr: "Görseller",
        ja: "画像",
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
        cs: "Audio a video",
        da: "Lyd & video",
        fi: "Ääni ja video",
        nb: "Lyd & video",
        tr: "Ses ve video",
        ja: "音声と動画",
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
        cs: "Dokumenty",
        da: "Dokumenter",
        fi: "Asiakirjat",
        nb: "Dokumenter",
        tr: "Belgeler",
        ja: "ドキュメント",
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
        cs: "Binární soubory a archivy",
        da: "Binære filer & arkiver",
        fi: "Binaarit ja arkistot",
        nb: "Binærfiler & arkiver",
        tr: "İkili dosyalar ve arşivler",
        ja: "バイナリとアーカイブ",
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
        cs: "Typy polí schématu (použití jako `field:type` v --schema)",
        da: "Skemafelttyper (brug som `field:type` i --schema)",
        fi: "Skeeman kenttätyypit (käytä muodossa `field:type` --schema-valinnassa)",
        nb: "Skjemafelttyper (brukes som `field:type` i --schema)",
        tr: "Şema alan türleri (--schema içinde `field:type` olarak kullanın)",
        ja: "スキーマのフィールド型（--schema で `field:type` として使用）",
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
        cs: "Jazykové oblasti dat (--locale)",
        da: "Datasprogområder (--locale)",
        fi: "Datan kielialueet (--locale)",
        nb: "Dataspråkområder (--locale)",
        tr: "Veri yerel ayarları (--locale)",
        ja: "データロケール (--locale)",
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
        cs: "Jazyky rozhraní (--lang)",
        da: "Grænsefladesprog (--lang)",
        fi: "Käyttöliittymän kielet (--lang)",
        nb: "Grensesnittspråk (--lang)",
        tr: "Arayüz dilleri (--lang)",
        ja: "インターフェース言語 (--lang)",
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
        cs: "Předvolby schématu (--preset)",
        da: "Skemaforudindstillinger (--preset)",
        fi: "Skeeman esiasetukset (--preset)",
        nb: "Skjemaforhåndsinnstillinger (--preset)",
        tr: "Şema ön ayarları (--preset)",
        ja: "スキーマプリセット (--preset)",
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
        cs: "Tip: spusťte libovolný formát s --help pro zobrazení jeho možností, např. `demodatagen json --help`.",
        da: "Tip: kør et vilkårligt format med --help for at se dets muligheder, f.eks. `demodatagen json --help`.",
        fi: "Vinkki: suorita mikä tahansa muoto --help-valinnalla nähdäksesi sen asetukset, esim. `demodatagen json --help`.",
        nb: "Tips: kjør et hvilket som helst format med --help for å se alternativene, f.eks. `demodatagen json --help`.",
        tr: "İpucu: seçeneklerini görmek için herhangi bir biçimi --help ile çalıştırın, örn. `demodatagen json --help`.",
        ja: "ヒント: 各形式を --help 付きで実行するとオプションを確認できます。例: `demodatagen json --help`。",
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
        cs: "Vestavěné předvolby schématu",
        da: "Indbyggede skemaforudindstillinger",
        fi: "Sisäänrakennetut skeeman esiasetukset",
        nb: "Innebygde skjemaforhåndsinnstillinger",
        tr: "Yerleşik şema ön ayarları",
        ja: "組み込みのスキーマプリセット",
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
        cs: "Použijte předvolbu místo ručního psaní schématu: `demodatagen <format> --preset <name>`.",
        da: "Brug en forudindstilling i stedet for at skrive et skema i hånden: `demodatagen <format> --preset <name>`.",
        fi: "Käytä esiasetusta sen sijaan, että kirjoittaisit skeeman käsin: `demodatagen <format> --preset <name>`.",
        nb: "Bruk en forhåndsinnstilling i stedet for å skrive et skjema for hånd: `demodatagen <format> --preset <name>`.",
        tr: "Elle şema yazmak yerine bir ön ayar kullanın: `demodatagen <format> --preset <name>`.",
        ja: "スキーマを手書きする代わりにプリセットを使えます: `demodatagen <format> --preset <name>`。",
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
        cs: "Funguje s libovolným strukturovaným formátem (json, csv, sql, yaml, …); lze kombinovat s --rows a --locale.",
        da: "Virker med ethvert struktureret format (json, csv, sql, yaml, …); kombiner med --rows og --locale.",
        fi: "Toimii minkä tahansa rakenteisen muodon kanssa (json, csv, sql, yaml, …); yhdistä valintoihin --rows ja --locale.",
        nb: "Fungerer med alle strukturerte formater (json, csv, sql, yaml, …); kombiner med --rows og --locale.",
        tr: "Her yapılandırılmış biçimle çalışır (json, csv, sql, yaml, …); --rows ve --locale ile birleştirin.",
        ja: "構造化形式（json、csv、sql、yaml、…）すべてで使用可能。--rows や --locale と組み合わせられます。",
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
        cs: "Schéma:",
        da: "Skema:",
        fi: "Skeema:",
        nb: "Skjema:",
        tr: "Şema:",
        ja: "スキーマ:",
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
        cs: "Uživatelské účty se jmény, e-maily, uživatelskými jmény a daty registrace.",
        da: "Brugerkonti med navne, e-mails, brugernavne og tilmeldingsdatoer.",
        fi: "Käyttäjätilit, joissa on nimet, sähköpostit, käyttäjätunnukset ja rekisteröitymispäivät.",
        nb: "Brukerkontoer med navn, e-postadresser, brukernavn og registreringsdatoer.",
        tr: "Adlar, e-postalar, kullanıcı adları ve kayıt tarihleri içeren kullanıcı hesapları.",
        ja: "氏名、メールアドレス、ユーザー名、登録日を含むユーザーアカウント。",
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
        cs: "Personální záznamy: osoby, oddělení, pracovní pozice a platy.",
        da: "HR-data: personer, afdelinger, jobtitler og lønninger.",
        fi: "HR-tiedot: henkilöt, osastot, tehtävänimikkeet ja palkat.",
        nb: "HR-data: personer, avdelinger, stillingstitler og lønninger.",
        tr: "İK kayıtları: kişiler, departmanlar, unvanlar ve maaşlar.",
        ja: "人事記録: 人物、部署、役職、給与。",
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
        cs: "Zákazníci s kontaktními údaji, adresami a telefonními čísly.",
        da: "Kunder med kontaktoplysninger, adresser og telefonnumre.",
        fi: "Asiakkaat, joissa on yhteystiedot, osoitteet ja puhelinnumerot.",
        nb: "Kunder med kontaktinformasjon, adresser og telefonnumre.",
        tr: "İletişim bilgileri, adresler ve telefon numaraları içeren müşteriler.",
        ja: "連絡先、住所、電話番号を含む顧客データ。",
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
        cs: "Katalogové produkty s SKU, cenou, kategorií a hodnocením.",
        da: "Katalogprodukter med SKU, pris, kategori og bedømmelse.",
        fi: "Katalogituotteet, joissa on SKU, hinta, kategoria ja arvosana.",
        nb: "Katalogprodukter med SKU, pris, kategori og vurdering.",
        tr: "SKU, fiyat, kategori ve puan içeren katalog ürünleri.",
        ja: "SKU、価格、カテゴリ、評価を含むカタログ商品。",
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
        cs: "E-commerce objednávky se zákazníkem, produktem, množstvím a celkovou částkou.",
        da: "E-handelsordrer med kunde, produkt, antal og total.",
        fi: "Verkkokaupan tilaukset, joissa on asiakas, tuote, määrä ja loppusumma.",
        nb: "E-handelsordrer med kunde, produkt, antall og totalsum.",
        tr: "Müşteri, ürün, adet ve toplam içeren e-ticaret siparişleri.",
        ja: "顧客、商品、数量、合計を含む EC 注文。",
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
        cs: "Finanční transakce s částkou, měnou, IBAN a stavem.",
        da: "Finansielle transaktioner med beløb, valuta, IBAN og status.",
        fi: "Rahoitustapahtumat, joissa on summa, valuutta, IBAN ja tila.",
        nb: "Finansielle transaksjoner med beløp, valuta, IBAN og status.",
        tr: "Tutar, para birimi, IBAN ve durum içeren finansal işlemler.",
        ja: "金額、通貨、IBAN、ステータスを含む金融取引。",
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
        cs: "Analytické události s typem, časovým razítkem, uživatelem a ID relace.",
        da: "Analysehændelser med type, tidsstempel, bruger og sessions-id.",
        fi: "Analytiikkatapahtumat, joissa on tyyppi, aikaleima, käyttäjä ja istuntotunnus.",
        nb: "Analysehendelser med type, tidsstempel, bruker og økt-ID.",
        tr: "Tür, zaman damgası, kullanıcı ve oturum kimliği içeren analitik olayları.",
        ja: "種類、タイムスタンプ、ユーザー、セッション ID を含む分析イベント。",
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
        cs: "Infrastrukturní hostitelé s názvy hostitelů, IP adresami, porty a dobou běhu.",
        da: "Infrastrukturværter med værtsnavne, IP'er, porte og oppetid.",
        fi: "Infrastruktuurin isännät, joissa on isäntänimet, IP-osoitteet, portit ja käyttöaika.",
        nb: "Infrastrukturverter med vertsnavn, IP-adresser, porter og oppetid.",
        tr: "Ana bilgisayar adları, IP'ler, bağlantı noktaları ve çalışma süresi içeren altyapı sunucuları.",
        ja: "ホスト名、IP、ポート、稼働時間を含むインフラホスト。",
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
        cs: "Geografické body se zeměpisnou šířkou a délkou, městem a zemí.",
        da: "Geografiske punkter med breddegrad, længdegrad, by og land.",
        fi: "Maantieteelliset pisteet, joissa on leveysaste, pituusaste, kaupunki ja maa.",
        nb: "Geografiske punkter med breddegrad, lengdegrad, by og land.",
        tr: "Enlem, boylam, şehir ve ülke içeren coğrafi noktalar.",
        ja: "緯度、経度、都市、国を含む地理座標。",
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
        cs: "Blogové nebo sociální příspěvky s autorem, titulkem, textem a štítky.",
        da: "Blog- eller sociale opslag med forfatter, titel, tekst og tags.",
        fi: "Blogi- tai somejulkaisut, joissa on kirjoittaja, otsikko, teksti ja tunnisteet.",
        nb: "Blogg- eller sosiale innlegg med forfatter, tittel, tekst og emneknagger.",
        tr: "Yazar, başlık, metin ve etiket içeren blog veya sosyal medya gönderileri.",
        ja: "著者、タイトル、本文、タグを含むブログ・SNS 投稿。",
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
        cs: "Platební záznamy s metodou, kartou, částkou a měnou.",
        da: "Betalingsposter med metode, kort, beløb og valuta.",
        fi: "Maksutapahtumat, joissa on maksutapa, kortti, summa ja valuutta.",
        nb: "Betalingsposter med metode, kort, beløp og valuta.",
        tr: "Yöntem, kart, tutar ve para birimi içeren ödeme kayıtları.",
        ja: "支払い方法、カード、金額、通貨を含む決済記録。",
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
        cs: "Odečty IoT senzorů s ID zařízení, metrikou, hodnotou a časem.",
        da: "IoT-sensormålinger med enheds-id, metrik, værdi og tid.",
        fi: "IoT-anturilukemat, joissa on laitetunnus, mittari, arvo ja aika.",
        nb: "IoT-sensoravlesninger med enhets-ID, metrikk, verdi og tid.",
        tr: "Cihaz kimliği, metrik, değer ve zaman içeren IoT sensör okumaları.",
        ja: "デバイス ID、メトリクス、値、時刻を含む IoT センサー測定値。",
    },
    /// One-line description of the 'invoices' preset.
    preset_desc_invoices {
        en: "Invoices with customer, amount, currency, IBAN and due date.",
        de: "Rechnungen mit Kunde, Betrag, Währung, IBAN und Fälligkeit.",
        fr: "Factures avec client, montant, devise, IBAN et échéance.",
        es: "Facturas con cliente, importe, moneda, IBAN y vencimiento.",
        it: "Fatture con cliente, importo, valuta, IBAN e scadenza.",
        pt: "Faturas com cliente, valor, moeda, IBAN e vencimento.",
        nl: "Facturen met klant, bedrag, valuta, IBAN en vervaldatum.",
        pl: "Faktury z klientem, kwotą, walutą, numerem IBAN i terminem.",
        sv: "Fakturor med kund, belopp, valuta, IBAN och förfallodatum.",
        cs: "Faktury se zákazníkem, částkou, měnou, IBAN a datem splatnosti.",
        da: "Fakturaer med kunde, beløb, valuta, IBAN og forfaldsdato.",
        fi: "Laskut, joissa on asiakas, summa, valuutta, IBAN ja eräpäivä.",
        nb: "Fakturaer med kunde, beløp, valuta, IBAN og forfallsdato.",
        tr: "Müşteri, tutar, para birimi, IBAN ve vade tarihi içeren faturalar.",
        ja: "顧客、金額、通貨、IBAN、支払期日を含む請求書。",
    },
    /// One-line description of the 'logins' preset.
    preset_desc_logins {
        en: "Login attempts with user, IP, user agent, MFA and outcome.",
        de: "Login-Versuche mit Benutzer, IP, User-Agent, MFA und Ergebnis.",
        fr: "Tentatives de connexion avec utilisateur, IP, agent, MFA et résultat.",
        es: "Intentos de inicio de sesión con usuario, IP, agente, MFA y resultado.",
        it: "Tentativi di accesso con utente, IP, user agent, MFA ed esito.",
        pt: "Tentativas de login com usuário, IP, user agent, MFA e resultado.",
        nl: "Inlogpogingen met gebruiker, IP, user-agent, MFA en resultaat.",
        pl: "Próby logowania z użytkownikiem, IP, przeglądarką, MFA i wynikiem.",
        sv: "Inloggningsförsök med användare, IP, user agent, MFA och utfall.",
        cs: "Pokusy o přihlášení s uživatelem, IP, user agentem, MFA a výsledkem.",
        da: "Loginforsøg med bruger, IP, user agent, MFA og resultat.",
        fi: "Kirjautumisyritykset, joissa on käyttäjä, IP, user agent, MFA ja tulos.",
        nb: "Påloggingsforsøk med bruker, IP, user agent, MFA og utfall.",
        tr: "Kullanıcı, IP, kullanıcı aracısı, MFA ve sonuç içeren oturum açma denemeleri.",
        ja: "ユーザー、IP、ユーザーエージェント、MFA、結果を含むログイン試行。",
    },
    /// One-line description of the 'vehicles' preset.
    preset_desc_vehicles {
        en: "Vehicles with make, model year, fuel, mileage and price.",
        de: "Fahrzeuge mit Marke, Baujahr, Antrieb, Laufleistung und Preis.",
        fr: "Véhicules avec marque, année, carburant, kilométrage et prix.",
        es: "Vehículos con marca, año, combustible, kilometraje y precio.",
        it: "Veicoli con marca, anno, alimentazione, chilometraggio e prezzo.",
        pt: "Veículos com marca, ano, combustível, quilometragem e preço.",
        nl: "Voertuigen met merk, bouwjaar, brandstof, kilometerstand en prijs.",
        pl: "Pojazdy z marką, rocznikiem, paliwem, przebiegiem i ceną.",
        sv: "Fordon med märke, årsmodell, bränsle, miltal och pris.",
        cs: "Vozidla se značkou, rokem výroby, palivem, nájezdem a cenou.",
        da: "Køretøjer med mærke, årgang, brændstof, kilometertal og pris.",
        fi: "Ajoneuvot, joissa on merkki, vuosimalli, polttoaine, kilometrilukema ja hinta.",
        nb: "Kjøretøy med merke, årsmodell, drivstoff, kilometerstand og pris.",
        tr: "Marka, model yılı, yakıt, kilometre ve fiyat içeren araçlar.",
        ja: "メーカー、年式、燃料、走行距離、価格を含む車両。",
    },
    /// One-line description of the 'books' preset.
    preset_desc_books {
        en: "Books with ISBN, title, author, pages, language and price.",
        de: "Bücher mit ISBN, Titel, Autor, Seiten, Sprache und Preis.",
        fr: "Livres avec ISBN, titre, auteur, pages, langue et prix.",
        es: "Libros con ISBN, título, autor, páginas, idioma y precio.",
        it: "Libri con ISBN, titolo, autore, pagine, lingua e prezzo.",
        pt: "Livros com ISBN, título, autor, páginas, idioma e preço.",
        nl: "Boeken met ISBN, titel, auteur, pagina's, taal en prijs.",
        pl: "Książki z ISBN, tytułem, autorem, stronami, językiem i ceną.",
        sv: "Böcker med ISBN, titel, författare, sidor, språk och pris.",
        cs: "Knihy s ISBN, titulem, autorem, počtem stran, jazykem a cenou.",
        da: "Bøger med ISBN, titel, forfatter, sider, sprog og pris.",
        fi: "Kirjat, joissa on ISBN, nimi, tekijä, sivumäärä, kieli ja hinta.",
        nb: "Bøker med ISBN, tittel, forfatter, sider, språk og pris.",
        tr: "ISBN, başlık, yazar, sayfa sayısı, dil ve fiyat içeren kitaplar.",
        ja: "ISBN、タイトル、著者、ページ数、言語、価格を含む書籍。",
    },
    /// Header line of the `preview` command. {rows} number, {locale} locale id.
    preview_header {
        en: "Previewing {rows} sample record(s) · data locale {locale}",
        de: "Vorschau von {rows} Beispieldatensätzen · Daten-Sprachraum {locale}",
        fr: "Aperçu de {rows} enregistrement(s) d'exemple · région {locale}",
        es: "Vista previa de {rows} registro(s) de ejemplo · región {locale}",
        it: "Anteprima di {rows} record di esempio · area {locale}",
        pt: "Pré-visualização de {rows} registro(s) de exemplo · localidade {locale}",
        nl: "Voorbeeld van {rows} voorbeeldrecord(s) · dataregio {locale}",
        pl: "Podgląd {rows} przykładowych rekordów · region danych {locale}",
        sv: "Förhandsvisar {rows} exempelpost(er) · dataspråkområde {locale}",
        cs: "Náhled {rows} ukázkových záznamů · jazyková oblast dat {locale}",
        da: "Forhåndsvisning af {rows} eksempelpost(er) · datasprogområde {locale}",
        fi: "Esikatsellaan {rows} esimerkkitietuetta · datan kielialue {locale}",
        nb: "Forhåndsviser {rows} eksempelpost(er) · dataspråkområde {locale}",
        tr: "{rows} örnek kayıt önizleniyor · veri yerel ayarı {locale}",
        ja: "{rows} 件のサンプルレコードをプレビュー中 · データロケール {locale}",
    },
    /// Hint printed under the preview table. Keep the backticked flags verbatim.
    preview_hint {
        en: "Generate real files with any format subcommand, e.g. `demodatagen json --schema …`. Add `--seed` to reproduce this exact preview.",
        de: "Erzeuge echte Dateien mit einem Format-Subcommand, z. B. `demodatagen json --schema …`. Mit `--seed` wird genau diese Vorschau reproduzierbar.",
        fr: "Générez de vrais fichiers avec une sous-commande de format, p. ex. `demodatagen json --schema …`. Ajoutez `--seed` pour reproduire cet aperçu.",
        es: "Genera archivos reales con un subcomando de formato, p. ej. `demodatagen json --schema …`. Añade `--seed` para reproducir esta vista previa.",
        it: "Genera file reali con un sottocomando di formato, ad es. `demodatagen json --schema …`. Aggiungi `--seed` per riprodurre questa anteprima.",
        pt: "Gere arquivos reais com um subcomando de formato, p. ex. `demodatagen json --schema …`. Adicione `--seed` para reproduzir esta pré-visualização.",
        nl: "Genereer echte bestanden met een formaat-subcommando, bijv. `demodatagen json --schema …`. Voeg `--seed` toe om precies dit voorbeeld te reproduceren.",
        pl: "Wygeneruj prawdziwe pliki podkomendą formatu, np. `demodatagen json --schema …`. Dodaj `--seed`, aby odtworzyć dokładnie ten podgląd.",
        sv: "Generera riktiga filer med ett formatunderkommando, t.ex. `demodatagen json --schema …`. Lägg till `--seed` för att återskapa exakt denna förhandsvisning.",
        cs: "Skutečné soubory vygenerujete podpříkazem formátu, např. `demodatagen json --schema …`. Přidejte `--seed` pro reprodukci přesně tohoto náhledu.",
        da: "Generér rigtige filer med en formatunderkommando, f.eks. `demodatagen json --schema …`. Tilføj `--seed` for at genskabe præcis denne forhåndsvisning.",
        fi: "Luo oikeita tiedostoja millä tahansa muotoalikomennolla, esim. `demodatagen json --schema …`. Lisää `--seed` toistaaksesi täsmälleen tämän esikatselun.",
        nb: "Generer ekte filer med en formatunderkommando, f.eks. `demodatagen json --schema …`. Legg til `--seed` for å gjenskape akkurat denne forhåndsvisningen.",
        tr: "Herhangi bir biçim alt komutuyla gerçek dosyalar oluşturun, örn. `demodatagen json --schema …`. Tam olarak bu önizlemeyi yeniden üretmek için `--seed` ekleyin.",
        ja: "各形式のサブコマンドで実際のファイルを生成できます。例: `demodatagen json --schema …`。`--seed` を追加すると、このプレビューを正確に再現できます。",
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
        cs: "Informace o prostředí a sestavení",
        da: "Miljø- & buildoplysninger",
        fi: "Ympäristö- ja käännöstiedot",
        nb: "Miljø- & bygginformasjon",
        tr: "Ortam ve derleme bilgileri",
        ja: "環境とビルドの情報",
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
        cs: "Verze",
        da: "Version",
        fi: "Versio",
        nb: "Versjon",
        tr: "Sürüm",
        ja: "バージョン",
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
        cs: "Cíl sestavení",
        da: "Buildmål",
        fi: "Käännöskohde",
        nb: "Byggmål",
        tr: "Derleme hedefi",
        ja: "ビルドターゲット",
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
        cs: "Profil sestavení",
        da: "Buildprofil",
        fi: "Käännösprofiili",
        nb: "Byggprofil",
        tr: "Derleme profili",
        ja: "ビルドプロファイル",
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
        cs: "Výstupní formáty",
        da: "Outputformater",
        fi: "Tulostemuodot",
        nb: "Utdataformater",
        tr: "Çıktı biçimleri",
        ja: "出力形式",
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
        cs: "Jazykové oblasti dat",
        da: "Datasprogområder",
        fi: "Datan kielialueet",
        nb: "Dataspråkområder",
        tr: "Veri yerel ayarları",
        ja: "データロケール",
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
        cs: "Jazyky rozhraní",
        da: "Grænsefladesprog",
        fi: "Käyttöliittymän kielet",
        nb: "Grensesnittspråk",
        tr: "Arayüz dilleri",
        ja: "インターフェース言語",
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
        cs: "Předvolby schématu",
        da: "Skemaforudindstillinger",
        fi: "Skeeman esiasetukset",
        nb: "Skjemaforhåndsinnstillinger",
        tr: "Şema ön ayarları",
        ja: "スキーマプリセット",
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
        cs: "Pracovní vlákna",
        da: "Arbejdstråde",
        fi: "Työsäikeet",
        nb: "Arbeidstråder",
        tr: "Çalışan iş parçacıkları",
        ja: "ワーカースレッド",
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
        cs: "Samoaktualizace",
        da: "Selvopdatering",
        fi: "Itsepäivitys",
        nb: "Selvoppdatering",
        tr: "Kendi kendine güncelleme",
        ja: "自己更新",
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
        cs: "Repozitář",
        da: "Repository",
        fi: "Repositorio",
        nb: "Kodelager",
        tr: "Depo",
        ja: "リポジトリ",
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
        cs: "Licence",
        da: "Licens",
        fi: "Lisenssi",
        nb: "Lisens",
        tr: "Lisans",
        ja: "ライセンス",
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
        cs: "zapnuto",
        da: "aktiveret",
        fi: "käytössä",
        nb: "aktivert",
        tr: "etkin",
        ja: "有効",
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
        cs: "vypnuto",
        da: "deaktiveret",
        fi: "pois käytöstä",
        nb: "deaktivert",
        tr: "devre dışı",
        ja: "無効",
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
        cs: "Spusťte `demodatagen list` pro formáty, typy schématu, jazykové oblasti a jazyky.",
        da: "Kør `demodatagen list` for formater, skematyper, sprogområder og sprog.",
        fi: "Suorita `demodatagen list` nähdäksesi muodot, skeematyypit, kielialueet ja kielet.",
        nb: "Kjør `demodatagen list` for formater, skjematyper, språkområder og språk.",
        tr: "Biçimler, şema türleri, yerel ayarlar ve diller için `demodatagen list` komutunu çalıştırın.",
        ja: "`demodatagen list` を実行すると、形式、スキーマ型、ロケール、言語を確認できます。",
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
        cs: "Zkušební běh — pouze plánování, žádné soubory nebudou zapsány.",
        da: "Prøvekørsel — kun planlægning, ingen filer skrives.",
        fi: "Kuivaharjoitus — vain suunnittelu, tiedostoja ei kirjoiteta.",
        nb: "Prøvekjøring — kun planlegging, ingen filer skrives.",
        tr: "Deneme çalıştırması — yalnızca planlama, hiçbir dosya yazılmayacak.",
        ja: "ドライラン — 計画のみで、ファイルは書き込まれません。",
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
        cs: "Vygenerovalo by se {count} × {format} → {dir}",
        da: "Ville generere {count} × {format} → {dir}",
        fi: "Luotaisiin {count} × {format} → {dir}",
        nb: "Ville generert {count} × {format} → {dir}",
        tr: "Oluşturulacak: {count} × {format} → {dir}",
        ja: "生成予定: {count} × {format} → {dir}",
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
        cs: "Plánované soubory:",
        da: "Planlagte filer:",
        fi: "Suunnitellut tiedostot:",
        nb: "Planlagte filer:",
        tr: "Planlanan dosyalar:",
        ja: "生成予定のファイル:",
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
        cs: "… a dalších {count}",
        da: "… og {count} mere",
        fi: "… ja {count} lisää",
        nb: "… og {count} til",
        tr: "… ve {count} tane daha",
        ja: "… ほか {count} 件",
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
        cs: "Zkušební běh dokončen — naplánováno {count} souborů.",
        da: "Prøvekørsel fuldført — {count} fil(er) planlagt.",
        fi: "Kuivaharjoitus valmis — {count} tiedosto(a) suunniteltu.",
        nb: "Prøvekjøring fullført — {count} fil(er) planlagt.",
        tr: "Deneme çalıştırması tamamlandı — {count} dosya planlandı.",
        ja: "ドライラン完了 — {count} 件のファイルが計画されました。",
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
    /// Czech.
    Cs,
    /// Danish.
    Da,
    /// Finnish.
    Fi,
    /// Norwegian Bokmål.
    Nb,
    /// Turkish.
    Tr,
    /// Japanese.
    Ja,
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
            Language::Cs => &CS_CATALOG,
            Language::Da => &DA_CATALOG,
            Language::Fi => &FI_CATALOG,
            Language::Nb => &NB_CATALOG,
            Language::Tr => &TR_CATALOG,
            Language::Ja => &JA_CATALOG,
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
            Language::Cs => "cs",
            Language::Da => "da",
            Language::Fi => "fi",
            Language::Nb => "nb",
            Language::Tr => "tr",
            Language::Ja => "ja",
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
            Language::Cs => "Čeština",
            Language::Da => "Dansk",
            Language::Fi => "Suomi",
            Language::Nb => "Norsk bokmål",
            Language::Tr => "Türkçe",
            Language::Ja => "日本語",
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
            Language::Cs,
            Language::Da,
            Language::Fi,
            Language::Nb,
            Language::Tr,
            Language::Ja,
        ]
    }

    /// Returns all canonical language identifiers, for help text and `list`.
    pub fn all() -> &'static [&'static str] {
        &[
            "en", "de", "fr", "es", "it", "pt", "nl", "pl", "sv", "cs", "da", "fi", "nb", "tr",
            "ja",
        ]
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
            "cs" | "cs_cz" | "czech" | "cestina" | "čeština" => Ok(Language::Cs),
            "da" | "da_dk" | "danish" | "dansk" => Ok(Language::Da),
            "fi" | "fi_fi" | "finnish" | "suomi" => Ok(Language::Fi),
            "nb" | "nb_no" | "no" | "no_no" | "norwegian" | "norsk" | "bokmal" | "bokmål" => {
                Ok(Language::Nb)
            }
            "tr" | "tr_tr" | "turkish" | "turkce" | "türkçe" => Ok(Language::Tr),
            "ja" | "ja_jp" | "japanese" | "日本語" => Ok(Language::Ja),
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
        assert_eq!("cs".parse::<Language>().unwrap(), Language::Cs);
        assert_eq!("da_DK".parse::<Language>().unwrap(), Language::Da);
        assert_eq!("suomi".parse::<Language>().unwrap(), Language::Fi);
        assert_eq!("no".parse::<Language>().unwrap(), Language::Nb);
        assert_eq!("nb_NO".parse::<Language>().unwrap(), Language::Nb);
        assert_eq!("Turkish".parse::<Language>().unwrap(), Language::Tr);
        assert_eq!("ja_JP".parse::<Language>().unwrap(), Language::Ja);
        assert!("xx".parse::<Language>().is_err());
    }

    #[test]
    fn test_roundtrip() {
        for id in Language::all() {
            assert_eq!(id.parse::<Language>().unwrap().as_str(), *id);
        }
        assert_eq!(Language::variants().len(), Language::all().len());
        assert_eq!(Language::variants().len(), 15);
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
