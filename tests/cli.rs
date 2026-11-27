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
        self.command(day, args)
            .env("FOKUS_DATEI", self.file())
            .output()
            .unwrap()
    }

    /// Ohne Spieldatei, mit eigenem Zuhause und eigenen Einstellungen: wie echt.
    fn run_at_home(&self, args: &[&str]) -> Output {
        self.command("2026-11-26", args)
            .env_remove("FOKUS_DATEI")
            .env("HOME", self.dir.join("home"))
            .output()
            .unwrap()
    }

    fn command(&self, day: &str, args: &[&str]) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_fokus"));
        c.args(args)
            .env("FOKUS_HEUTE", day)
            .env("FOKUS_EINSTELLUNGEN", self.dir.join("einstellungen"))
            .env_remove("FOKUS_UNTERWEGS")
            .env_remove("XDG_CONFIG_HOME")
            .env("NO_COLOR", "1")
            .env("COLUMNS", "80");
        c
    }

    fn ok_at_home(&self, args: &[&str]) -> String {
        let out = self.run_at_home(args);
        assert!(
            out.status.success(),
            "fokus {args:?} schlug fehl:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    fn away(&self) -> PathBuf {
        self.dir.join("icloud").join("Fokus-unterwegs.md")
    }

    /// Mit Spieldatei und ausdrücklich gesetzter Unterwegs-Datei.
    fn ok_away(&self, args: &[&str]) -> String {
        let out = self
            .command("2026-11-26", args)
            .env("FOKUS_DATEI", self.file())
            .env("FOKUS_UNTERWEGS", self.away())
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "fokus {args:?} schlug fehl:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
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

#[test]
fn unterwegs_aufgeschriebenes_wird_abgeholt_und_der_stand_geschrieben() {
    let s = Sandbox::new("unterwegs");
    fs::create_dir_all(s.dir.join("icloud")).unwrap();
    s.ok_away(&["+", "Präsentation", "vorbereiten", "#arbeit", "morgen"]);
    s.ok_away(&["+", "Vertrag", "prüfen", "@tom", "#arbeit"]);
    s.ok_away(&["abgeben", "2", "@tom"]);

    let stand = fs::read_to_string(s.away()).unwrap();
    assert!(stand.contains("Stand: Donnerstag 26.11., "));
    assert!(stand.contains("## Neu\n"));
    // Mit Datum statt „morgen“: Am Handy liest man es vielleicht später.
    assert!(stand.contains("- 1 Präsentation vorbereiten · Arbeit · fällig Freitag 27.11."));
    assert!(stand.contains("- 2 Vertrag prüfen @tom · bei Tom seit 26.11."));

    // „Neu“ steht ganz unten, damit Angehängtes von selbst dort landet.
    assert!(
        stand.ends_with("## Neu\n"),
        "„Neu“ gehört ans Ende:\n{stand}"
    );
    assert!(stand.find("## Warte auf").unwrap() < stand.find("## Neu").unwrap());

    // Am Handy: zwei Zeilen unter „Neu“, eine davon ohne Strich, und ein leerer
    // Strich – angehängt, wie es ein Kurzbefehl täte.
    let mut vom_handy = stand.clone();
    vom_handy.push_str("- Trikots bestellen #verein Samstag\nIdee für den Talentpfad\n-\n");
    fs::write(s.away(), vom_handy).unwrap();

    let out = s.ok_away(&[]);
    assert!(out.contains("Von unterwegs:"));
    assert!(
        out.contains("Aufgabe 3: Trikots bestellen · Verein · fällig übermorgen (Samstag 28.11.)")
    );
    assert!(out.contains("Aufgabe 4: Idee für den Talentpfad"));

    let datei = fs::read_to_string(s.file()).unwrap();
    assert!(datei.contains("Trikots bestellen #verein"));
    assert!(datei.contains("Idee für den Talentpfad"));

    let stand = fs::read_to_string(s.away()).unwrap();
    assert!(
        stand.ends_with("## Neu\n"),
        "„Neu“ sollte leer sein:\n{stand}"
    );
    assert!(stand.contains("- 3 Trikots bestellen · Verein · fällig Samstag 28.11."));

    // Ein zweiter Aufruf legt nichts doppelt an.
    let out = s.ok_away(&[]);
    assert!(!out.contains("Von unterwegs:"));
    assert_eq!(fs::read_to_string(s.file()).unwrap(), datei);
}

#[test]
fn ohne_aenderung_bleibt_die_unterwegs_datei_unberuehrt() {
    let s = Sandbox::new("unterwegs-ruhe");
    fs::create_dir_all(s.dir.join("icloud")).unwrap();
    s.ok_away(&["+", "Brot", "kaufen"]);
    // Eine andere Uhrzeit allein ist kein Grund zum Schreiben.
    let alt: String = fs::read_to_string(s.away())
        .unwrap()
        .lines()
        .map(|l| {
            if l.starts_with("Stand: ") {
                "Stand: Donnerstag 26.11., 00:00".to_string()
            } else {
                l.to_string()
            }
        })
        .map(|l| l + "\n")
        .collect();
    fs::write(s.away(), &alt).unwrap();
    s.ok_away(&["alles"]);
    assert_eq!(fs::read_to_string(s.away()).unwrap(), alt);
}

#[test]
fn eine_spieldatei_schreibt_nie_in_die_echte_unterwegs_datei() {
    let s = Sandbox::new("unterwegs-spiel");
    let echt = s.dir.join("echt").join("Fokus-unterwegs.md");
    fs::write(
        s.dir.join("einstellungen"),
        format!("unterwegs = {}\n", echt.display()),
    )
    .unwrap();
    s.ok(&["+", "Spielzeug"]);
    assert!(!echt.exists());
    assert!(!echt.parent().unwrap().exists());
}

#[test]
fn ein_verschwundener_ordner_wird_gemeldet_statt_angelegt() {
    let s = Sandbox::new("unterwegs-weg");
    let out = s.ok_away(&["+", "Brot"]);
    assert!(out.contains("Den Ordner für die Unterwegs-Datei gibt es nicht mehr"));
    assert!(!s.dir.join("icloud").exists());
}

#[test]
fn eine_nicht_geladene_icloud_datei_wird_nicht_ueberschrieben() {
    let s = Sandbox::new("unterwegs-icloud");
    let dir = s.dir.join("icloud");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(".Fokus-unterwegs.md.icloud"), "Platzhalter").unwrap();
    let out = s.ok_away(&["+", "Brot"]);
    assert!(out.contains("noch nicht aus iCloud geladen"));
    assert!(!s.away().exists());
}

#[test]
fn die_unterwegs_datei_wird_eingeschaltet_und_abgeschaltet() {
    let s = Sandbox::new("unterwegs-befehl");
    fs::create_dir_all(s.dir.join("home").join("Documents")).unwrap();
    s.ok_at_home(&["+", "Brot", "kaufen"]);
    let ordner = s.dir.join("home").join("iCloud").join("Fokus");
    let out = s.ok_at_home(&["unterwegs", ordner.to_str().unwrap()]);
    assert!(out.contains("Fokus schreibt den Stand jetzt nach"));
    let datei = ordner.join("Fokus-unterwegs.md");
    assert!(
        fs::read_to_string(&datei)
            .unwrap()
            .contains("- 1 Brot kaufen")
    );
    let einstellungen = fs::read_to_string(s.dir.join("einstellungen")).unwrap();
    assert!(einstellungen.contains("unterwegs = ~/iCloud/Fokus/Fokus-unterwegs.md"));

    assert!(
        s.ok_at_home(&["unterwegs"])
            .contains("Die Unterwegs-Datei liegt in")
    );
    assert!(
        s.ok_at_home(&["hilfe"])
            .contains("Die Unterwegs-Datei liegt in")
    );

    let out = s.ok_at_home(&["unterwegs", "aus"]);
    assert!(out.contains("Fokus schreibt keine Unterwegs-Datei mehr"));
    assert!(!s.dir.join("einstellungen").exists());
    assert!(
        s.ok_at_home(&["unterwegs"])
            .contains("Es gibt noch keine Unterwegs-Datei")
    );
}

#[test]
fn die_unterwegs_datei_kann_nicht_die_aufgabendatei_sein() {
    let s = Sandbox::new("unterwegs-gleich");
    let docs = s.dir.join("home").join("Documents");
    fs::create_dir_all(&docs).unwrap();
    s.ok_at_home(&["+", "Brot"]);
    let out = s.run_at_home(&["unterwegs", docs.join("Fokus.md").to_str().unwrap()]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("Das ist deine Aufgabendatei selbst"));
}

#[test]
fn die_aufgabendatei_zieht_um_und_fokus_findet_sie_wieder() {
    let s = Sandbox::new("verschieben");
    let docs = s.dir.join("home").join("Documents");
    fs::create_dir_all(&docs).unwrap();
    s.ok_at_home(&["+", "Brot", "kaufen"]);
    assert!(docs.join("Fokus.md").exists());

    let ziel = s.dir.join("home").join("Planung");
    let out = s.ok_at_home(&["datei", "verschieben", ziel.to_str().unwrap()]);
    assert!(out.contains("Deine Aufgaben liegen jetzt in"));
    assert!(!docs.join("Fokus.md").exists());
    assert!(ziel.join("Fokus.md").exists());
    let einstellungen = fs::read_to_string(s.dir.join("einstellungen")).unwrap();
    assert!(einstellungen.contains("datei = ~/Planung/Fokus.md"));

    let out = s.ok_at_home(&["alles"]);
    assert!(out.contains("Brot kaufen"));
    assert!(
        !docs.join("Fokus.md").exists(),
        "der alte Ort darf nicht wieder entstehen"
    );
    assert!(s.ok_at_home(&["hilfe"]).contains("Planung/Fokus.md"));

    // Zurück an den Standardort: Die Einstellung verschwindet wieder.
    s.ok_at_home(&["datei", "verschieben", docs.to_str().unwrap()]);
    assert!(docs.join("Fokus.md").exists());
    assert!(!s.dir.join("einstellungen").exists());
}

#[test]
fn beim_umziehen_wird_keine_vorhandene_datei_ueberschrieben() {
    let s = Sandbox::new("verschieben-belegt");
    let docs = s.dir.join("home").join("Documents");
    fs::create_dir_all(&docs).unwrap();
    s.ok_at_home(&["+", "Brot"]);
    let ziel = s.dir.join("anderswo");
    fs::create_dir_all(&ziel).unwrap();
    fs::write(ziel.join("Fokus.md"), "## Offen\n- Fremd\n").unwrap();

    let out = s.run_at_home(&["datei", "verschieben", ziel.to_str().unwrap()]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("Die überschreibe ich nicht"));
    assert_eq!(
        fs::read_to_string(ziel.join("Fokus.md")).unwrap(),
        "## Offen\n- Fremd\n"
    );
    assert!(docs.join("Fokus.md").exists());
    assert!(!s.dir.join("einstellungen").exists());
}

#[test]
fn mit_spieldatei_wird_nichts_verschoben() {
    let s = Sandbox::new("verschieben-spiel");
    let err = s.fails(&["datei", "verschieben", "/tmp/irgendwo"]);
    assert!(err.contains("Spieldatei"));
}

#[test]
fn eine_unterwegs_datei_mit_neu_oben_wird_sauber_uebernommen() {
    // So sah die Datei vor „Neu am Ende“ aus: Nur „Neu“ zählt, nicht „Heute“.
    let s = Sandbox::new("unterwegs-alt");
    fs::create_dir_all(s.dir.join("icloud")).unwrap();
    s.ok_away(&["+", "Brot", "kaufen", "heute"]);
    fs::write(
        s.away(),
        "# Fokus unterwegs\n\nStand: Donnerstag 26.11., 06:10\n\n## Neu\n\n- Milch\n\n## Heute · Donnerstag 26.11.\n- 1 Brot kaufen\n",
    )
    .unwrap();
    let out = s.ok_away(&[]);
    assert!(out.contains("Aufgabe 2: Milch"));
    assert!(
        !out.contains("Aufgabe 3"),
        "aus „Heute“ darf nichts übernommen werden:\n{out}"
    );
    let stand = fs::read_to_string(s.away()).unwrap();
    assert!(stand.ends_with("## Neu\n"));
    assert_eq!(stand.matches("## Neu").count(), 1);
}

#[test]
fn ohne_heisst_ueberall_kein_thema() {
    let s = Sandbox::new("ohne");
    s.ok(&["+", "Bericht", "#arbeit"]);
    s.ok(&["+", "Mama", "anrufen"]);
    s.ok(&["+", "Idee", "#ohne"]);

    let out = s.ok(&["#ohne"]);
    assert!(out.contains("OHNE THEMA"));
    assert!(out.contains("Mama anrufen"));
    assert!(out.contains("Idee"));
    assert!(!out.contains("Bericht"));

    // Ein Thema geben: Die Aufgabe steht nicht mehr unter #ohne.
    s.ok(&["ändern", "2", "#arbeit"]);
    assert!(!s.ok(&["#ohne"]).contains("Mama anrufen"));
    // #ohne nimmt das Thema wieder weg.
    s.ok(&["ändern", "1", "#ohne"]);
    assert!(s.ok(&["#ohne"]).contains("Bericht"));

    let datei = fs::read_to_string(s.file()).unwrap();
    assert!(
        !datei.to_lowercase().contains("ohne"),
        "„ohne“ darf nicht in der Datei landen:\n{datei}"
    );
    assert!(s.ok(&["themen"]).contains("fokus #ohne"));
    assert!(
        s.fails(&["themen", "umbenennen", "arbeit", "ohne"])
            .contains("„Ohne“ ist vergeben")
    );
    assert!(s.fails(&["+", "#ohne"]).contains("Da fehlt noch"));
}

#[test]
fn ohne_in_einer_von_hand_geaenderten_datei_wird_kein_thema() {
    let s = Sandbox::new("ohne-von-hand");
    fs::write(
        s.file(),
        "## Themen\n1. Ohne\n2. Arbeit\n---\n\n## Offen\n- [ ] Brot #ohne\n- [ ] Bericht #arbeit\n",
    )
    .unwrap();
    let out = s.ok(&["#ohne"]);
    assert!(out.contains("Brot"));
    assert!(!out.contains("Bericht"));
    let datei = fs::read_to_string(s.file()).unwrap();
    assert!(!datei.contains("#ohne"));
    assert!(!datei.contains("1. Ohne"));
    assert!(datei.contains("1. Arbeit"));
}

#[test]
fn ohne_ohne_aufgaben_ohne_thema() {
    let s = Sandbox::new("ohne-leer");
    s.ok(&["+", "Bericht", "#arbeit"]);
    assert!(
        s.ok(&["#ohne"])
            .contains("Alle offenen Aufgaben haben ein Thema.")
    );
}

#[test]
fn die_anleitung_zeigt_den_passenden_abschnitt() {
    let s = Sandbox::new("anleitung");
    let out = s.ok(&["anleitung", "aufraeumen"]);
    assert!(out.contains("Einmal pro Woche: aufräumen"));
    assert!(out.contains("fokus aufräumen"));
    assert!(!out.contains("```"));
    assert!(!out.contains("ALLE BEFEHLE"));
    // Lesen legt keine Aufgabendatei an.
    assert!(!s.file().exists());

    let out = s.ok(&["anleitung", "gibtsnicht"]);
    assert!(out.contains("Zu „gibtsnicht“ steht nichts in der Anleitung"));
    assert!(out.contains("Typische Situationen"));

    let alles = s.ok(&["anleitung"]);
    assert!(alles.contains("ALLE BEFEHLE"));
    assert!(alles.contains("LIZENZ"));
}
