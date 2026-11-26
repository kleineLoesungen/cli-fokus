//! Das fertige Programm, von außen aufgerufen: mit eigener Datei und festem Tag.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

struct Sandbox {
    dir: PathBuf,
}

impl Sandbox {
    fn new(name: &str) -> Sandbox {
        let dir = std::env::temp_dir().join(format!("fokus-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Sandbox { dir }
    }

    fn file(&self) -> PathBuf {
        self.dir.join("Fokus.md")
    }

    fn run_on(&self, day: &str, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_fokus"))
            .args(args)
            .env("FOKUS_DATEI", self.file())
            .env("FOKUS_HEUTE", day)
            .env("NO_COLOR", "1")
            .env("COLUMNS", "80")
            .output()
            .unwrap()
    }

    fn ok(&self, args: &[&str]) -> String {
        self.ok_on("2026-11-26", args)
    }

    fn ok_on(&self, day: &str, args: &[&str]) -> String {
        let out = self.run_on(day, args);
        assert!(
            out.status.success(),
            "fokus {args:?} schlug fehl:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    fn fails(&self, args: &[&str]) -> String {
        let out = self.run_on("2026-11-26", args);
        assert!(
            !out.status.success(),
            "fokus {args:?} hätte fehlschlagen sollen"
        );
        String::from_utf8(out.stderr).unwrap()
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn ohne_aufgaben_erklaert_fokus_den_ersten_schritt() {
    let s = Sandbox::new("leer");
    let out = s.ok(&[]);
    assert!(out.contains("Donnerstag, 26. November"));
    assert!(out.contains("fokus + Präsentation vorbereiten #arbeit morgen"));
}

#[test]
fn aufschreiben_erkennt_thema_person_und_termin() {
    let s = Sandbox::new("plus");
    let out = s.ok(&["+", "Urlaub", "abstimmen", "@anna", "#familie", "morgen"]);
    assert!(
        out.contains(
            "Aufgabe 1: Urlaub abstimmen @anna · Familie · fällig morgen (Freitag 27.11.)"
        )
    );
    let datei = fs::read_to_string(s.file()).unwrap();
    assert!(datei.contains(
        "- [ ] Urlaub abstimmen @anna #familie  [Nr. 1 · fällig 27.11.2026 · seit 26.11.2026]"
    ));
}

#[test]
fn heute_hat_hoechstens_drei_plaetze() {
    let s = Sandbox::new("heute");
    for t in ["A", "B", "C", "D"] {
        s.ok(&["+", t]);
    }
    s.ok(&["heute", "1", "2", "3"]);
    let err = s.fails(&["heute", "4"]);
    assert!(err.contains("fokus heute 4 statt <Nr>"));
    let out = s.ok(&["heute", "4", "statt", "2"]);
    assert!(out.contains("Aufgabe 2 ist wieder von heute herunter."));
    assert!(out.contains("0 von 3 erledigt"));
}

#[test]
fn eine_erledigte_nummer_bleibt_14_tage_im_archiv_reserviert() {
    let s = Sandbox::new("nummern");
    s.ok(&["+", "Erste"]);
    s.ok(&["+", "Zweite"]);
    s.ok(&["fertig", "1"]);
    assert!(s.ok(&["+", "Dritte"]).contains("Aufgabe 3:"));
    assert!(
        s.ok_on("2026-11-27", &["+", "Vierte"])
            .contains("Aufgabe 4:")
    );
    assert!(s.ok_on("2026-11-27", &["archiv"]).contains("Erste"));
    // Nach 14 Tagen ist sie endgültig weg, und die Nummer ist wieder frei.
    assert!(
        s.ok_on("2026-12-11", &["+", "Fünfte"])
            .contains("Aufgabe 1:")
    );
    assert!(!s.ok_on("2026-12-11", &["archiv"]).contains("Erste"));
}

#[test]
fn vier_wichtige_themen_gibt_es_nicht() {
    let s = Sandbox::new("themen");
    for t in ["A #eins", "B #zwei", "C #drei", "D #vier"] {
        s.ok(&["+", t]);
    }
    let err = s.fails(&["themen", "ordnen", "1", "2", "3", "4"]);
    assert!(err.contains("Höchstens 3 Themen"));
    let out = s.ok(&["themen", "ordnen", "vier", "1"]);
    assert!(out.contains("Wichtig sind jetzt: Vier, Eins"));
    assert!(out.contains("Über der Linie ist noch Platz für ein Thema."));
}

#[test]
fn abgeben_erscheint_bei_der_person() {
    let s = Sandbox::new("personen");
    s.ok(&["+", "Fotos", "schicken"]);
    s.ok(&["+", "Ausflug", "klären", "@anna"]);
    s.ok(&["abgeben", "1", "@anna"]);
    let p = s.ok(&["@anna"]);
    assert!(p.contains("ANNA"));
    assert!(p.contains("Ausflug klären @anna"));
    assert!(p.contains("WARTE AUF ANNA"));
    assert!(p.contains("Fotos schicken"));
    let alle = s.ok(&["personen"]);
    assert!(
        alle.contains("Anna                1            1"),
        "{alle}"
    );
}

#[test]
fn ein_neuer_tag_bringt_einen_vorschlag() {
    let s = Sandbox::new("vorschlag");
    s.ok(&["+", "Bericht", "#arbeit", "morgen"]);
    s.ok(&["+", "Keller", "#haus"]);
    s.ok(&["+", "Laufen", "#sport"]);
    s.ok(&["+", "Nachbar", "#garten"]);
    s.ok(&["+", "Vereinskram", "#verein"]);
    let out = s.ok_on("2026-11-27", &[]);
    assert!(out.contains("Vorschlag von Fokus"));
    assert!(out.contains("1 ▶ Bericht"), "{out}");
    // #verein ist das vierte Thema und damit unwichtig: nie im Vorschlag.
    assert!(!out.contains("Vereinskram"));
}

#[test]
fn unbekannte_befehle_helfen_weiter() {
    let s = Sandbox::new("unbekannt");
    let err = s.fails(&["kaufen", "Milch"]);
    assert!(err.contains("fokus hilfe"));
    assert!(err.contains("fokus + kaufen Milch"));
}

#[test]
fn die_vier_felder_passen_in_80_spalten() {
    let s = Sandbox::new("matrix");
    s.ok(&[
        "+", "Eine", "ziemlich", "lange", "Aufgabe", "mit", "viel", "Text", "#arbeit", "morgen",
    ]);
    s.ok(&["+", "Keller", "ausmisten"]);
    let out = s.ok(&["alles"]);
    assert!(out.contains("TUN"));
    assert!(out.contains("STREICHEN?"));
    for line in out.lines() {
        assert!(unicode_width(line) <= 80, "zu breit: {line}");
    }
    s.ok(&["+", "Fotos", "schicken"]);
    s.ok(&["abgeben", "3", "@anna"]);
    let out = s.ok(&["alles"]);
    for line in out.lines() {
        assert!(unicode_width(line) <= 80, "zu breit: {line}");
    }
    // Im Kasten sind alle Zeilen gleich breit, sonst springt der rechte Rand.
    let box_widths: Vec<usize> = out
        .lines()
        .filter(|l| l.contains('│') || l.contains('┌'))
        .map(unicode_width)
        .collect();
    assert!(box_widths.windows(2).all(|w| w[0] == w[1]), "{out}");
}

fn unicode_width(s: &str) -> usize {
    s.chars().count()
}

#[test]
fn deinstallieren_ohne_terminal_aendert_nichts() {
    let s = Sandbox::new("deinstallieren");
    let err = s.fails(&["deinstallieren"]);
    assert!(err.contains("Terminalfenster"));
    // Auch keine leere Aufgabendatei angelegt.
    assert!(!s.file().exists());
}

#[test]
fn ein_thema_zeigt_offenes_abgegebenes_und_erledigtes() {
    let s = Sandbox::new("thema");
    s.ok(&["+", "Präsentation", "#arbeit", "morgen"]);
    s.ok(&["+", "Vertrag", "prüfen", "#arbeit"]);
    s.ok(&["+", "Rechnung", "#arbeit"]);
    s.ok(&["+", "Keller", "#haus"]);
    s.ok(&["abgeben", "2", "@tom"]);
    s.ok(&["fertig", "3"]);
    let out = s.ok(&["#arbeit"]);
    assert!(out.contains("ARBEIT   wichtig"), "{out}");
    assert!(out.contains("Präsentation"));
    assert!(out.contains("LIEGT BEI ANDEREN"));
    assert!(out.contains("bei Tom"));
    assert!(out.contains("erledigt: Rechnung"));
    assert!(!out.contains("Keller"));
}

#[test]
fn warten_zeigt_das_aelteste_zuerst() {
    let s = Sandbox::new("warten");
    assert!(s.ok(&["warten"]).contains("auf niemanden"));
    s.ok_on("2026-11-10", &["+", "Alt"]);
    s.ok_on("2026-11-10", &["abgeben", "1", "@tom"]);
    s.ok(&["+", "Neu"]);
    s.ok(&["abgeben", "2", "@anna"]);
    let out = s.ok(&["warten"]);
    let alt = out.find("Alt").unwrap();
    let neu = out.find("Neu").unwrap();
    assert!(alt < neu, "{out}");
    assert!(out.contains("seit 16 Tagen"));
    assert!(out.contains("nachhaken"));
}

#[test]
fn versehentlich_abgehakt_kommt_zurueck() {
    let s = Sandbox::new("zurueck");
    s.ok(&["+", "Präsentation"]);
    s.ok(&["heute", "1"]);
    let out = s.ok(&["fertig", "1"]);
    assert!(out.contains("Versehen? fokus zurück 1"));
    assert!(s.ok(&["zurück", "1"]).contains("wieder auf heute"));
    assert!(s.ok(&[]).contains("1 ▶ Präsentation"));
}

#[test]
fn ueberfaelliges_erscheint_am_naechsten_tag_im_aufraeumen() {
    let s = Sandbox::new("ueberfaellig");
    // Ohne Thema, also unwichtig: Fokus schlägt sie morgens nicht für heute vor.
    s.ok(&["+", "Steuer", "heute"]);
    s.ok(&["später", "1"]);
    assert!(s.ok(&["aufräumen"]).contains("Nichts aufzuräumen"));
    let out = s.ok_on("2026-11-27", &["aufräumen"]);
    assert!(out.contains("Steuer"), "{out}");
    assert!(out.contains("seit gestern fällig"));
    assert!(s.ok_on("2026-11-27", &[]).contains("1 überfällig"));
}
