//! Autovervollständigung mit der Tab-Taste (zsh).
//!
//! Die Shell fragt bei jedem Tab `focus _complete <Wörter bis hier>` und
//! bekommt je Zeile `Gruppe<Tab>Wert<Tab>Anzeige` zurück. So kennt die
//! Vervollständigung immer die aktuellen Nummern, Themen und Personen, ohne
//! eigene Liste. Aufeinanderfolgende Zeilen derselben Gruppe zeigt zsh unter
//! einer gemeinsamen Überschrift, in genau dieser Reihenfolge.

use std::path::Path;

use crate::store::{Status, Store, Task};
use crate::{date, i18n, tags, term, tr, trs};

/// Ein Vorschlag: unter welcher Überschrift, was eingesetzt wird, was man sieht.
pub struct Item {
    pub group: String,
    pub value: String,
    pub shown: String,
}

fn item(group: &str, value: impl Into<String>, shown: impl Into<String>) -> Item {
    Item {
        group: group.to_string(),
        value: value.into(),
        shown: shown.into(),
    }
}

/// Wert links, Erklärung daneben – wie in der Hilfe.
fn described(group: &str, value: &str, what: &str) -> Item {
    item(group, value, format!("{value:<16}{what}"))
}

/// Die Befehle für die Tab-Taste, in der gewählten Sprache.
fn commands() -> &'static [(&'static str, &'static str, &'static str)] {
    if i18n::en() { COMMANDS_EN } else { COMMANDS }
}

const COMMANDS_EN: &[(&str, &str, &str)] = &[
    ("For today", "+", "write down a task"),
    ("For today", "today", "put a task on today"),
    ("For today", "done", "check off a task"),
    ("For today", "later", "take a task off today"),
    ("For today", "now", "just one task, with a clock"),
    ("View", "all", "all tasks in four quadrants"),
    ("View", "topics", "your topics"),
    ("View", "people", "everyone with open tasks"),
    ("View", "waiting", "everything that's with others"),
    ("View", "search", "find tasks, also in notes"),
    ("View", "archive", "done and dropped, 14 days"),
    ("Change", "edit", "change text, topic, person or date"),
    ("Change", "delegate", "hand a task to someone"),
    ("Change", "drop", "drop a task"),
    ("Change", "restore", "bring back from the archive"),
    ("Change", "review", "go through what got stuck"),
    ("Notes", "note", "note or link for a task or topic"),
    ("Notes", "open", "open a link from the notes"),
    ("File and setup", "file", "open or move the task file"),
    (
        "File and setup",
        "mobile",
        "state for your phone, with “New”",
    ),
    ("File and setup", "setup", "language, shortcut and Tab key"),
    ("File and setup", "web", "Focus in a browser tab"),
    ("File and setup", "uninstall", "remove Focus"),
    ("File and setup", "help", "all commands"),
    (
        "File and setup",
        "manual",
        "the whole manual, also for a keyword",
    ),
];

const COMMANDS: &[(&str, &str, &str)] = &[
    ("Für heute", "+", "Aufgabe aufschreiben"),
    ("Für heute", "heute", "Aufgabe auf heute legen"),
    ("Für heute", "fertig", "Aufgabe abhaken"),
    ("Für heute", "später", "Aufgabe von heute nehmen"),
    ("Für heute", "jetzt", "nur eine Aufgabe, mit Uhr"),
    ("Ansehen", "alles", "alle Aufgaben in vier Feldern"),
    ("Ansehen", "themen", "deine Themen"),
    ("Ansehen", "personen", "alle Personen mit offenen Aufgaben"),
    ("Ansehen", "warten", "alles, was bei anderen liegt"),
    ("Ansehen", "suche", "Aufgaben finden, auch in Notizen"),
    ("Ansehen", "archiv", "Erledigtes und Gestrichenes, 14 Tage"),
    ("Ändern", "ändern", "Text, Thema, Person oder Termin ändern"),
    ("Ändern", "abgeben", "Aufgabe an jemanden geben"),
    ("Ändern", "streichen", "Aufgabe streichen"),
    ("Ändern", "zurück", "aus dem Archiv zurückholen"),
    ("Ändern", "aufräumen", "Liegengebliebenes durchgehen"),
    ("Notizen", "notiz", "Notiz oder Link zu Aufgabe oder Thema"),
    ("Notizen", "öffnen", "Link aus den Notizen öffnen"),
    (
        "Datei und Einrichtung",
        "datei",
        "Aufgabendatei öffnen oder verschieben",
    ),
    (
        "Datei und Einrichtung",
        "unterwegs",
        "Stand fürs Handy, mit „Neu“",
    ),
    (
        "Datei und Einrichtung",
        "einrichten",
        "Sprache, Kurzbefehl und Tab-Taste",
    ),
    ("Datei und Einrichtung", "web", "Focus im Browser-Tab"),
    ("Datei und Einrichtung", "deinstallieren", "Focus entfernen"),
    ("Datei und Einrichtung", "hilfe", "alle Befehle"),
    (
        "Datei und Einrichtung",
        "anleitung",
        "die ganze Anleitung, auch zu einem Stichwort",
    ),
];

/// Statt eigener Vorschläge: die Shell soll Ordner und Dateien anbieten.
const FILES: &str = "__dateien__";

const DATE_WORDS_EN: &[&str] = &[
    "today",
    "tomorrow",
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
    "weekend",
    "next",
    "every",
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
pub fn candidates(store: &Store, words: &[String]) -> Vec<Item> {
    let (prev, cur) = match words.split_last() {
        Some((cur, prev)) => (prev, cur.as_str()),
        None => (&[][..], ""),
    };
    // Englische Befehle heißen intern wie die deutschen.
    let Some(first) = prev.first().map(|s| i18n::canonical(s)) else {
        if cur.starts_with('@') {
            return persons(store);
        }
        if cur.starts_with('#') {
            return topics_or_none(store);
        }
        return commands()
            .iter()
            .map(|(g, c, d)| described(g, c, d))
            .collect();
    };
    let mentioned: Vec<u32> = prev[1..].iter().filter_map(|w| w.parse().ok()).collect();
    let nrs = |filter: &dyn Fn(&Task) -> bool| -> Vec<Item> {
        numbers(store, |t| filter(t) && !mentioned.contains(&t.nr))
    };
    match first.as_str() {
        "heute" => {
            if prev.last().is_some_and(|w| i18n::word(w) == "statt") {
                return nrs(&|t| t.status == Status::Today);
            }
            let mut v = nrs(&|t| t.status != Status::Today);
            if prev.len() == 2 && store.today_open().len() >= crate::store::today_limit() {
                v.insert(
                    0,
                    described(
                        "",
                        trs!("statt", "instead"),
                        trs!(
                            "eine Aufgabe von heute dafür herunter",
                            "take a task off today for it"
                        ),
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
            described(
                "",
                trs!("ordnen", "order"),
                trs!("die wichtigen Themen wählen", "pick the important topics"),
            ),
            described(
                "",
                trs!("umbenennen", "rename"),
                trs!(
                    "ein Thema umbenennen oder zusammenlegen",
                    "rename or merge a topic"
                ),
            ),
        ],
        "themen" | "thema" if prev.len() == 2 && i18n::word(&prev[1]) == "umbenennen" => {
            bare(topics(store))
        }
        "themen" | "thema" if i18n::word(&prev[1]) == "ordnen" => {
            let chosen: Vec<String> = prev[2..].iter().map(|w| tags::key(w)).collect();
            bare(topics(store))
                .into_iter()
                .filter(|i| !chosen.contains(&tags::key(&i.value)))
                .collect()
        }
        "notiz" | "notizen" if prev.len() == 1 => note_targets(store, cur, false),
        "notiz" | "notizen" if prev.len() == 2 && prev[1].contains('.') => {
            vec![described(
                "",
                trs!("löschen", "delete"),
                trs!("diese Notiz löschen", "delete this note"),
            )]
        }
        "öffnen" | "oeffnen" | "link" if prev.len() == 1 => note_targets(store, cur, true),
        "datei" if prev.len() == 1 => {
            vec![described(
                "",
                trs!("verschieben", "move"),
                trs!("an einen anderen Ort legen", "move it somewhere else"),
            )]
        }
        "datei" if prev.len() == 2 => vec![item("", FILES, "")],
        "unterwegs" if prev.len() == 1 => {
            let off = trs!("aus", "off");
            if off.starts_with(&cur.to_lowercase()) && !cur.is_empty() {
                vec![described("", off, trs!("abschalten", "turn off"))]
            } else {
                vec![item("", FILES, "")]
            }
        }
        "ändern" | "aendern" | "bearbeiten" if prev.len() == 1 => nrs(&|_| true),
        "ändern" | "aendern" | "bearbeiten" => {
            if cur.starts_with('#') {
                topics_or_none(store)
            } else if cur.starts_with('@') {
                persons(store)
            } else if prev.last().is_some_and(|w| i18n::word(w) == "ohne") {
                let g = trs!("Entfernen", "Remove");
                let mut v = vec![
                    described(
                        g,
                        trs!("termin", "date"),
                        trs!("Termin entfernen", "remove the date"),
                    ),
                    described(
                        g,
                        trs!("thema", "topic"),
                        trs!("Thema entfernen", "remove the topic"),
                    ),
                    described(
                        g,
                        trs!("wiederholung", "repeat"),
                        trs!("Wiederholung beenden", "end the repeat"),
                    ),
                ];
                v.extend(persons(store));
                v
            } else if cur.is_empty() {
                Vec::new()
            } else {
                let mut v = date_words(cur);
                let no = trs!("ohne", "no");
                if no.starts_with(&cur.to_lowercase()) {
                    v.insert(
                        0,
                        described(
                            trs!("Entfernen", "Remove"),
                            no,
                            trs!("etwas entfernen", "remove something"),
                        ),
                    );
                }
                v
            }
        }
        f if f == "neu" || f.starts_with('+') => {
            if cur.starts_with('#') {
                topics(store)
            } else if cur.starts_with('@') {
                persons(store)
            } else if !cur.is_empty() {
                date_words(cur)
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    }
}

/// Ohne `#` davor, für `focus themen ordnen|umbenennen`.
fn bare(items: Vec<Item>) -> Vec<Item> {
    items
        .into_iter()
        .map(|mut i| {
            i.value = i.value.trim_start_matches('#').to_string();
            i
        })
        .collect()
}

fn date_words(cur: &str) -> Vec<Item> {
    let c = cur.to_lowercase();
    let words = if i18n::en() {
        DATE_WORDS_EN
    } else {
        DATE_WORDS
    };
    let repeats = trs!("Wiederholungen", "Repeats");
    let dates = trs!("Termine", "Dates");
    words
        .iter()
        .filter(|w| w.starts_with(&c))
        .map(|w| match *w {
            "jeden" => described(repeats, w, "jeden montag"),
            "alle" => described(repeats, w, "alle 2 wochen freitag"),
            "every" => described(repeats, w, "every monday · every 2 weeks friday"),
            "next" => described(dates, w, "next friday · next week"),
            _ => described(dates, w, ""),
        })
        .collect()
}

/// Nummern mit Text, gruppiert wie man denkt: heute, dringend, je Thema
/// (wichtige zuerst), ohne Thema, bei anderen – darin nach Termin.
fn numbers(store: &Store, keep: impl Fn(&Task) -> bool) -> Vec<Item> {
    let today = store.today;
    let group = |t: &Task| -> (usize, String) {
        if t.status == Status::Today {
            return (0, tr!("Heute", "Today"));
        }
        if t.status == Status::Waiting {
            return (usize::MAX, tr!("Bei anderen", "With others"));
        }
        if store.is_urgent(t) {
            return (1, tr!("Dringend", "Urgent"));
        }
        match store.topic_name(t) {
            Some(name) if store.is_important(t) => (
                2 + store.topic_rank(t),
                tr!("{} · wichtig", "{} · important", tags::topic_label(&name)),
            ),
            Some(name) => (2 + store.topic_rank(t), tags::topic_label(&name)),
            None => (usize::MAX - 1, tr!("Ohne Thema", "No topic")),
        }
    };
    let mut tasks: Vec<(&Task, (usize, String))> = store
        .tasks
        .iter()
        .filter(|t| t.is_active() && keep(t))
        .map(|t| (t, group(t)))
        .collect();
    tasks.sort_by_key(|(t, g)| (g.0, t.due.is_none(), t.due, t.nr));
    tasks
        .into_iter()
        .map(|(t, (_, g))| {
            let mut d = format!("{:<4}{}", t.nr, t.display());
            if let Some(due) = t.due {
                d.push_str(&format!("  · {}", date::due_short(due, today)));
            }
            if let Some(p) = &t.waiting_for {
                d.push_str(&tr!("  · bei {}", "  · with {}", tags::person_label(p)));
            }
            item(&g, t.nr.to_string(), term::truncate(&d, 70))
        })
        .collect()
}

/// Was im Archiv steht, mit Text – für `focus zurück`. Das Neueste zuerst.
fn archived(store: &Store, skip: &[u32]) -> Vec<Item> {
    let today = store.today;
    let mut tasks: Vec<&Task> = store
        .tasks
        .iter()
        .filter(|t| t.is_restorable(today) && !skip.contains(&t.nr))
        .collect();
    tasks.sort_by_key(|t| {
        (
            t.status != Status::Done,
            std::cmp::Reverse(t.done_on.or(t.archived_on)),
            t.nr,
        )
    });
    tasks
        .iter()
        .map(|t| {
            let g = if t.status == Status::Done {
                trs!("Erledigt", "Done")
            } else {
                trs!("Gestrichen", "Dropped")
            };
            let when = match t.done_on.or(t.archived_on).map(|d| (today - d).num_days()) {
                Some(0) => tr!("heute", "today"),
                Some(1) => tr!("gestern", "yesterday"),
                Some(n) => tr!("vor {n} Tagen", "{n} days ago"),
                None => String::new(),
            };
            let d = format!("{:<4}{}  · {when}", t.nr, t.display());
            item(g, t.nr.to_string(), term::truncate(&d, 70))
        })
        .collect()
}

/// Für `notiz` und `öffnen`: erst Aufgaben und Themen, nach dem Punkt die
/// einzelnen Notizen mit ihrem Text. `links`: nur, wo ein Link steht.
fn note_targets(store: &Store, cur: &str, links: bool) -> Vec<Item> {
    use crate::notes::{self, Target};
    let has_link = |v: &[String]| v.iter().any(|n| !crate::store::links(n).is_empty());
    if let Some((head, _)) = cur.rsplit_once('.')
        && let Some(target) = Target::parse(head)
    {
        let name = head.to_string();
        return notes::notes(store, &target)
            .iter()
            .enumerate()
            .filter(|(_, n)| !links || !crate::store::links(n).is_empty())
            .map(|(k, n)| {
                let v = format!("{name}.{}", k + 1);
                item(
                    trs!("Notizen", "Notes"),
                    v.clone(),
                    format!("{v:<10}{}", term::truncate(n, 60)),
                )
            })
            .collect();
    }
    let mut v = if cur.starts_with('#') {
        Vec::new()
    } else {
        numbers(store, |t| {
            !links || has_link(&t.notes) || !crate::store::links(&t.text).is_empty()
        })
    };
    v.extend(
        topics(store)
            .into_iter()
            .filter(|i| !links || has_link(store.notes_of_topic(&i.value))),
    );
    v
}

fn topics(store: &Store) -> Vec<Item> {
    store
        .important
        .iter()
        .map(|n| (trs!("Wichtig", "Important"), n))
        .chain(store.other.iter().map(|n| (trs!("Weitere", "More"), n)))
        .map(|(g, n)| {
            let v = safe(&format!("#{}", tags::to_tag(n)));
            described(g, &v, n)
        })
        .collect()
}

/// Die Themen und dazu `#ohne`: zum Anzeigen und beim Ändern.
fn topics_or_none(store: &Store) -> Vec<Item> {
    let mut v = topics(store);
    let none = tags::no_topic_tag();
    v.push(described(
        trs!("Weitere", "More"),
        &none,
        trs!("kein Thema", "no topic"),
    ));
    v
}

/// Personen, die mit den meisten offenen Aufgaben zuerst.
fn persons(store: &Store) -> Vec<Item> {
    let mut list: Vec<(String, String, usize)> = Vec::new();
    for t in store.tasks.iter().filter(|t| t.is_active()) {
        for p in t.persons().into_iter().chain(t.waiting_for.clone()) {
            let k = tags::key(&p);
            match list.iter_mut().find(|(key, _, _)| *key == k) {
                Some(e) => e.2 += 1,
                None => list.push((k, p, 1)),
            }
        }
    }
    list.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));
    list.into_iter()
        .map(|(_, p, n)| {
            let v = safe(&tags::person_label(&p));
            let what = tr!("{v} · {n} offen", "{v} · {n} open");
            described(trs!("Personen", "People"), &v, &what)
        })
        .collect()
}

/// Die Shell setzt Vorschläge unverändert ein (`compadd -Q`), damit aus `#arbeit`
/// nicht `\#arbeit` wird. Darum nur Zeichen, die der Shell nichts bedeuten.
fn safe(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | '#' | '@'))
        .collect()
}

/// Das Skript, das `focus einrichten` per `eval` in die `.zshrc` holt.
pub fn zsh_script(exe: &Path) -> String {
    let exe = format!("'{}'", exe.display().to_string().replace('\'', r"'\''"));
    let files = FILES;
    format!(
        r#"# Tab-Vervollständigung für focus. Erzeugt von „focus _zsh“.
# Jede Gruppe bekommt eine eigene Überschrift; die Reihenfolge kommt von focus.
_focus() {{
  local -a vals disp expl
  local g v d grp="" started=0
  local -i ret=1 n=0
  while IFS=$'\t' read -r g v d; do
    if [[ $v == {files} ]]; then
      _files
      return
    fi
    if (( started )) && [[ $g != $grp ]]; then
      (( n++ ))
      expl=()
      [[ -n $grp ]] && expl=(-X "%B$grp%b")
      compadd -Q -V "focus-$n" "${{expl[@]}}" -l -d disp -a vals && ret=0
      vals=() disp=()
    fi
    started=1 grp=$g
    vals+=("$v")
    disp+=("$d")
  done < <({exe} _complete "${{(@)words[2,CURRENT]}}" 2>/dev/null)
  if (( ${{#vals}} )); then
    (( n++ ))
    expl=()
    [[ -n $grp ]] && expl=(-X "%B$grp%b")
    compadd -Q -V "focus-$n" "${{expl[@]}}" -l -d disp -a vals && ret=0
  fi
  return ret
}}
# -i: Ordner mit lockeren Rechten still überspringen, statt beim Start nachzufragen.
if ! (( $+functions[compdef] )); then
  autoload -Uz compinit && compinit -i
fi
compdef _focus focus f
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
            .map(|i| i.value)
            .collect()
    }

    #[test]
    fn am_anfang_kommen_die_befehle() {
        let v = werte(&[""]);
        assert!(v.contains(&"heute".to_string()));
        assert!(v.contains(&"deinstallieren".to_string()));
        assert_eq!(werte(&["@"]), vec!["@anna", "@tom"]);
        assert_eq!(
            werte(&["#"]),
            vec!["#arbeit", "#eigenes-projekt", "#haus", "#ohne"]
        );
        assert!(werte(&[""]).contains(&"warten".to_string()));
    }

    #[test]
    fn nummern_passen_zum_befehl() {
        // Heute, dann je Thema (Eigenes Projekt vor Haus), zuletzt bei anderen.
        assert_eq!(werte(&["fertig", ""]), vec!["1", "4", "2", "3"]);
        assert_eq!(werte(&["fertig", "2", ""]), vec!["1", "4", "3"]);
        assert_eq!(werte(&["später", ""]), vec!["1"]);
        assert_eq!(werte(&["heute", ""]), vec!["4", "2", "3"]);
        assert_eq!(werte(&["abgeben", ""]), vec!["1", "4", "2"]);
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
        assert_eq!(safe("@michael.l"), "@michael.l");
    }

    #[test]
    fn aendern_bietet_nummern_dann_themen_personen_und_ohne() {
        assert_eq!(werte(&["ändern", ""]), vec!["1", "4", "2", "3"]);
        assert_eq!(
            werte(&["ändern", "2", "#"]),
            vec!["#arbeit", "#eigenes-projekt", "#haus", "#ohne"]
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
        let v: Vec<String> = candidates(&s, &w).into_iter().map(|i| i.value).collect();
        assert_eq!(v, vec!["2"]);
        assert_eq!(werte(&["+", "Plan", "je"]), vec!["jeden"]);
    }

    fn gruppen(words: &[&str]) -> Vec<String> {
        let w: Vec<String> = words.iter().map(|s| s.to_string()).collect();
        let mut g: Vec<String> = Vec::new();
        for i in candidates(&store(), &w) {
            if g.last() != Some(&i.group) {
                g.push(i.group);
            }
        }
        g
    }

    #[test]
    fn vorschlaege_kommen_in_gruppen() {
        assert_eq!(
            gruppen(&[""]),
            vec![
                "Für heute",
                "Ansehen",
                "Ändern",
                "Notizen",
                "Datei und Einrichtung"
            ]
        );
        assert_eq!(
            gruppen(&["fertig", ""]),
            vec![
                "Heute",
                "#eigenes-projekt · wichtig",
                "#haus",
                "Bei anderen"
            ]
        );
        assert_eq!(gruppen(&["+", "x", "#"]), vec!["Wichtig", "Weitere"]);
        assert_eq!(gruppen(&["+", "x", "mo"]), vec!["Termine"]);
    }

    #[test]
    fn themen_ordnen_bietet_jedes_thema_einmal() {
        assert_eq!(
            werte(&["themen", "ordnen", "haus", ""]),
            vec!["arbeit", "eigenes-projekt"]
        );
    }
}
