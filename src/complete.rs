//! Autovervollständigung mit der Tab-Taste (zsh).
//!
//! Die Shell fragt bei jedem Tab `fokus _complete <Wörter bis hier>` und
//! bekommt je Zeile `Wert<Tab>Anzeige` zurück. So kennt die Vervollständigung
//! immer die aktuellen Nummern, Themen und Personen, ohne eigene Liste.

use std::path::Path;

use crate::store::{Status, Store, Task};
use crate::{tags, term};

const COMMANDS: &[(&str, &str)] = &[
    ("+", "Aufgabe aufschreiben"),
    ("heute", "Aufgabe auf heute legen"),
    ("fertig", "Aufgabe abhaken"),
    ("später", "Aufgabe von heute nehmen"),
    ("jetzt", "nur eine Aufgabe, mit Uhr"),
    ("alles", "alle Aufgaben in vier Feldern"),
    ("themen", "deine Themen"),
    ("personen", "alle Personen mit offenen Aufgaben"),
    ("ändern", "Text, Thema, Person oder Termin ändern"),
    ("abgeben", "Aufgabe an jemanden geben"),
    ("warten", "alles, was bei anderen liegt"),
    ("streichen", "Aufgabe streichen"),
    ("archiv", "Erledigtes und Gestrichenes, 14 Tage"),
    ("zurück", "aus dem Archiv zurückholen"),
    ("aufräumen", "Liegengebliebenes durchgehen"),
    ("datei", "Aufgabendatei öffnen"),
    ("einrichten", "Kurzbefehl und Tab-Taste einrichten"),
    ("deinstallieren", "Fokus entfernen"),
    ("hilfe", "alle Befehle"),
];

const DATE_WORDS: &[&str] = &[
    "heute",
    "morgen",
    "übermorgen",
    "montag",
    "dienstag",
    "mittwoch",
    "donnerstag",
    "freitag",
    "samstag",
    "sonntag",
    "wochenende",
    "jeden",
    "alle",
];

/// Vorschläge für das letzte Wort in `words` (es darf leer sein).
pub fn candidates(store: &Store, words: &[String]) -> Vec<(String, String)> {
    let (prev, cur) = match words.split_last() {
        Some((cur, prev)) => (prev, cur.as_str()),
        None => (&[][..], ""),
    };
    let Some(first) = prev.first().map(|s| s.to_lowercase()) else {
        if cur.starts_with('@') {
            return persons(store);
        }
        if cur.starts_with('#') {
            return topics(store);
        }
        return COMMANDS
            .iter()
            .map(|(c, d)| (c.to_string(), format!("{c:<16}{d}")))
            .collect();
    };
    let mentioned: Vec<u32> = prev[1..].iter().filter_map(|w| w.parse().ok()).collect();
    let nrs = |filter: &dyn Fn(&Task) -> bool| -> Vec<(String, String)> {
        numbers(store, |t| filter(t) && !mentioned.contains(&t.nr))
    };
    match first.as_str() {
        "heute" => {
            if prev.last().is_some_and(|w| w.eq_ignore_ascii_case("statt")) {
                return nrs(&|t| t.status == Status::Today);
            }
            let mut v = nrs(&|t| t.status != Status::Today);
            if prev.len() == 2 && store.today_open().len() >= crate::store::TODAY_LIMIT {
                v.insert(
                    0,
                    (
                        "statt".into(),
                        "statt           eine Aufgabe von heute dafür herunter".into(),
                    ),
                );
            }
            v
        }
        "fertig" | "erledigt" | "streichen" | "löschen" | "loeschen" => nrs(&|_| true),
        "später" | "spaeter" => nrs(&|t| t.status == Status::Today),
        "zurück" | "zurueck" | "rückgängig" => archived(store, &mentioned),
        "jetzt" if prev.len() == 1 => nrs(&|t| t.status != Status::Waiting),
        "abgeben" if prev.len() == 1 => nrs(&|t| t.status != Status::Waiting),
        "abgeben" if prev.len() == 2 => persons(store),
        "themen" | "thema" if prev.len() == 1 => vec![
            (
                "ordnen".into(),
                "ordnen          die drei wichtigen Themen wählen".into(),
            ),
            (
                "umbenennen".into(),
                "umbenennen      ein Thema umbenennen oder zusammenlegen".into(),
            ),
        ],
        "themen" | "thema" if prev.len() == 2 && prev[1].eq_ignore_ascii_case("umbenennen") => {
            topics(store)
                .into_iter()
                .map(|(v, d)| (v.trim_start_matches('#').to_string(), d))
                .collect()
        }
        "themen" | "thema" if prev[1].eq_ignore_ascii_case("ordnen") => {
            let chosen: Vec<String> = prev[2..].iter().map(|w| tags::key(w)).collect();
            topics(store)
                .into_iter()
                .map(|(v, d)| (v.trim_start_matches('#').to_string(), d))
                .filter(|(v, _)| !chosen.contains(&tags::key(v)))
                .collect()
        }
        "ändern" | "aendern" | "bearbeiten" if prev.len() == 1 => nrs(&|_| true),
        "ändern" | "aendern" | "bearbeiten" => {
            if cur.starts_with('#') {
                topics(store)
            } else if cur.starts_with('@') {
                persons(store)
            } else if prev.last().is_some_and(|w| w.eq_ignore_ascii_case("ohne")) {
                let mut v = vec![
                    (
                        "termin".to_string(),
                        "termin          Termin entfernen".to_string(),
                    ),
                    (
                        "thema".to_string(),
                        "thema           Thema entfernen".to_string(),
                    ),
                    (
                        "wiederholung".to_string(),
                        "wiederholung    Wiederholung beenden".to_string(),
                    ),
                ];
                v.extend(persons(store));
                v
            } else if cur.is_empty() {
                Vec::new()
            } else {
                let c = cur.to_lowercase();
                std::iter::once("ohne")
                    .chain(DATE_WORDS.iter().copied())
                    .filter(|w| w.starts_with(&c))
                    .map(|w| {
                        (
                            w.to_string(),
                            format!(
                                "{w:<16}{}",
                                if w == "ohne" {
                                    "etwas entfernen"
                                } else {
                                    date_word_hint(w)
                                }
                            ),
                        )
                    })
                    .collect()
            }
        }
        f if f == "neu" || f.starts_with('+') => {
            if cur.starts_with('#') {
                topics(store)
            } else if cur.starts_with('@') {
                persons(store)
            } else if !cur.is_empty() {
                let c = cur.to_lowercase();
                DATE_WORDS
                    .iter()
                    .filter(|w| w.starts_with(&c))
                    .map(|w| (w.to_string(), format!("{w:<16}{}", date_word_hint(w))))
                    .collect()
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    }
}

/// Nummern mit dem Text der Aufgabe, damit man sieht, welche man nimmt.
fn numbers(store: &Store, keep: impl Fn(&Task) -> bool) -> Vec<(String, String)> {
    let mut tasks: Vec<&Task> = store
        .tasks
        .iter()
        .filter(|t| t.is_active() && keep(t))
        .collect();
    tasks.sort_by_key(|t| t.nr);
    tasks
        .iter()
        .map(|t| {
            let mut d = format!("{:<4}{}", t.nr, t.display());
            match t.status {
                Status::Today => d.push_str("  · heute"),
                Status::Waiting => {
                    if let Some(p) = &t.waiting_for {
                        d.push_str(&format!("  · bei {}", tags::display(p)));
                    }
                }
                _ => {}
            }
            (t.nr.to_string(), term::truncate(&d, 70))
        })
        .collect()
}

fn date_word_hint(w: &str) -> &'static str {
    match w {
        "jeden" => "Wiederholung: jeden montag",
        "alle" => "Wiederholung: alle 2 wochen freitag",
        _ => "Termin",
    }
}

/// Was im Archiv steht, mit Text – für `fokus zurück`.
fn archived(store: &Store, skip: &[u32]) -> Vec<(String, String)> {
    let mut tasks: Vec<&Task> = store
        .tasks
        .iter()
        .filter(|t| t.is_restorable(store.today) && !skip.contains(&t.nr))
        .collect();
    tasks.sort_by_key(|t| t.nr);
    tasks
        .iter()
        .map(|t| {
            let how = if t.status == Status::Done {
                "erledigt"
            } else {
                "gestrichen"
            };
            let d = format!("{:<4}{}  · {how}", t.nr, t.display());
            (t.nr.to_string(), term::truncate(&d, 70))
        })
        .collect()
}

fn topics(store: &Store) -> Vec<(String, String)> {
    store
        .important
        .iter()
        .map(|n| (n, " · wichtig"))
        .chain(store.other.iter().map(|n| (n, "")))
        .map(|(n, extra)| {
            let v = safe(&format!("#{}", tags::to_tag(n)));
            (v.clone(), format!("{v:<16}{n}{extra}"))
        })
        .collect()
}

fn persons(store: &Store) -> Vec<(String, String)> {
    let mut seen: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for t in store.tasks.iter().filter(|t| t.is_active()) {
        for p in t.persons().into_iter().chain(t.waiting_for.clone()) {
            let k = tags::key(&p);
            if seen.contains(&k) {
                continue;
            }
            seen.push(k);
            let v = safe(&format!("@{}", tags::clean(&p).to_lowercase()));
            out.push((v.clone(), format!("{v:<16}{}", tags::display(&p))));
        }
    }
    out
}

/// Die Shell setzt Vorschläge unverändert ein (`compadd -Q`), damit aus `#arbeit`
/// nicht `\#arbeit` wird. Darum nur Zeichen, die der Shell nichts bedeuten.
fn safe(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '#' | '@'))
        .collect()
}

/// Das Skript, das `fokus einrichten` per `eval` in die `.zshrc` holt.
pub fn zsh_script(exe: &Path) -> String {
    let exe = format!("'{}'", exe.display().to_string().replace('\'', r"'\''"));
    format!(
        r#"# Tab-Vervollständigung für fokus. Erzeugt von „fokus _zsh“.
_fokus() {{
  local -a vals disp
  local v d
  while IFS=$'\t' read -r v d; do
    vals+=("$v")
    disp+=("$d")
  done < <({exe} _complete "${{(@)words[2,CURRENT]}}" 2>/dev/null)
  (( ${{#vals}} )) || return 1
  compadd -Q -V fokus -l -d disp -a vals
}}
if ! (( $+functions[compdef] )); then
  autoload -Uz compinit && compinit
fi
compdef _fokus fokus f
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::path::PathBuf;

    fn store() -> Store {
        let text = "\
## Themen
1. Arbeit
2. Eigenes Projekt
---
- Haus

## Heute · 26.11.2026
- [ ] Präsentation #arbeit  [Nr. 1 · seit 26.11.2026]

## Warte auf
- [ ] Fotos  [Nr. 3 · wartet auf @anna · seit 26.11.2026]

## Offen
- [ ] Keller #haus  [Nr. 2 · seit 26.11.2026]
- [ ] Urlaub @tom #eigenes-projekt  [Nr. 4 · seit 26.11.2026]
";
        Store::parse(
            text,
            PathBuf::from("t.md"),
            NaiveDate::from_ymd_opt(2026, 11, 26).unwrap(),
        )
    }

    fn werte(words: &[&str]) -> Vec<String> {
        let w: Vec<String> = words.iter().map(|s| s.to_string()).collect();
        candidates(&store(), &w)
            .into_iter()
            .map(|(v, _)| v)
            .collect()
    }

    #[test]
    fn am_anfang_kommen_die_befehle() {
        let v = werte(&[""]);
        assert!(v.contains(&"heute".to_string()));
        assert!(v.contains(&"deinstallieren".to_string()));
        assert_eq!(werte(&["@"]), vec!["@anna", "@tom"]);
        assert_eq!(werte(&["#"]), vec!["#arbeit", "#eigenes-projekt", "#haus"]);
        assert!(werte(&[""]).contains(&"warten".to_string()));
    }

    #[test]
    fn nummern_passen_zum_befehl() {
        assert_eq!(werte(&["fertig", ""]), vec!["1", "2", "3", "4"]);
        assert_eq!(werte(&["fertig", "2", ""]), vec!["1", "3", "4"]);
        assert_eq!(werte(&["später", ""]), vec!["1"]);
        assert_eq!(werte(&["heute", ""]), vec!["2", "3", "4"]);
        assert_eq!(werte(&["abgeben", ""]), vec!["1", "2", "4"]);
        assert_eq!(werte(&["abgeben", "2", ""]), vec!["@anna", "@tom"]);
    }

    #[test]
    fn beim_aufschreiben_themen_personen_und_termine() {
        assert_eq!(
            werte(&["+", "Bericht", "#"]),
            vec!["#arbeit", "#eigenes-projekt", "#haus"]
        );
        assert_eq!(werte(&["+", "Bericht", "@t"]), vec!["@anna", "@tom"]);
        assert_eq!(werte(&["+", "Bericht", "mo"]), vec!["morgen", "montag"]);
        assert!(werte(&["+", "Bericht", ""]).is_empty());
    }

    #[test]
    fn vorschlaege_enthalten_keine_shell_zeichen() {
        assert_eq!(safe("#haus-&-garten"), "#haus--garten");
        assert_eq!(safe("@o'brien"), "@obrien");
        assert_eq!(safe("#übermorgen"), "#übermorgen");
    }

    #[test]
    fn aendern_bietet_nummern_dann_themen_personen_und_ohne() {
        assert_eq!(werte(&["ändern", ""]), vec!["1", "2", "3", "4"]);
        assert_eq!(
            werte(&["ändern", "2", "#"]),
            vec!["#arbeit", "#eigenes-projekt", "#haus"]
        );
        assert_eq!(werte(&["ändern", "2", "oh"]), vec!["ohne"]);
        assert_eq!(
            werte(&["ändern", "2", "ohne", ""]),
            vec!["termin", "thema", "wiederholung", "@anna", "@tom"]
        );
        assert_eq!(werte(&["themen", ""]), vec!["ordnen", "umbenennen"]);
        assert_eq!(
            werte(&["themen", "umbenennen", ""]),
            vec!["arbeit", "eigenes-projekt", "haus"]
        );
        assert!(werte(&["themen", "umbenennen", "haus", ""]).is_empty());
    }

    #[test]
    fn zurueck_bietet_nur_das_archiv() {
        let mut s = store();
        crate::ops::done(&mut s, 2).unwrap();
        let w: Vec<String> = ["zurück", ""].iter().map(|s| s.to_string()).collect();
        let v: Vec<String> = candidates(&s, &w).into_iter().map(|(v, _)| v).collect();
        assert_eq!(v, vec!["2"]);
        assert_eq!(werte(&["+", "Plan", "je"]), vec!["jeden"]);
    }

    #[test]
    fn themen_ordnen_bietet_jedes_thema_einmal() {
        assert_eq!(
            werte(&["themen", "ordnen", "haus", ""]),
            vec!["arbeit", "eigenes-projekt"]
        );
    }
}
