//! Sprache: Englisch oder Deutsch.
//!
//! Neue Installationen sprechen Englisch. Bestehende behalten Deutsch: Gibt es
//! noch keine Einstellung, aber schon eine deutsche Aufgabendatei, merkt sich
//! Focus Deutsch. `FOCUS_SPRACHE=de|en` geht vor (für Tests).
//!
//! Texte stehen mit `tr!("Deutsch {x}", "English {x}")` direkt im Code, beide
//! nebeneinander. Gelesen wird immer in beiden Sprachen: Befehle, Termine und
//! die Aufgabendatei.

use std::sync::atomic::{AtomicU8, Ordering};

use crate::{config, store};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    De,
    En,
}

/// 0: noch nicht bestimmt, 1: Deutsch, 2: Englisch.
static LANG: AtomicU8 = AtomicU8::new(0);

pub const KEY: &str = "sprache";

/// Die Sprache dieses Aufrufs, beim ersten Mal bestimmt.
pub fn lang() -> Lang {
    match LANG.load(Ordering::Relaxed) {
        1 => Lang::De,
        2 => Lang::En,
        _ => {
            let (l, remember) = detect();
            // Erst festlegen, dann merken: Das Schreiben der Einstellung fragt
            // selbst nach der Sprache.
            set(l);
            if remember {
                let _ = config::set_str(KEY, Some(code(l)));
            }
            l
        }
    }
}

/// Ab jetzt in dieser Sprache, etwa gleich nach der Frage in `focus einrichten`.
pub fn set(l: Lang) {
    LANG.store(if l == Lang::De { 1 } else { 2 }, Ordering::Relaxed);
}

pub fn en() -> bool {
    lang() == Lang::En
}

/// `de`, `deutsch`, `german` · `en`, `englisch`, `english`
pub fn parse(s: &str) -> Option<Lang> {
    match s.trim().to_lowercase().as_str() {
        "de" | "deutsch" | "german" | "d" => Some(Lang::De),
        "en" | "englisch" | "english" | "e" => Some(Lang::En),
        _ => None,
    }
}

pub fn code(l: Lang) -> &'static str {
    match l {
        Lang::De => "de",
        Lang::En => "en",
    }
}

/// Die Sprache, und ob sie sich gemerkt werden soll.
fn detect() -> (Lang, bool) {
    for var in ["FOCUS_SPRACHE", "FOCUS_LANG"] {
        if let Ok(v) = std::env::var(var)
            && let Some(l) = parse(&v)
        {
            return (l, false);
        }
    }
    if cfg!(test) {
        return (Lang::De, false);
    }
    if let Some(l) = config::get_str(KEY).as_deref().and_then(parse) {
        return (l, false);
    }
    // Wer Focus schon auf Deutsch benutzt, soll nach dem Update nicht plötzlich
    // Englisch lesen.
    let file = store::file_path();
    let german = std::fs::read_to_string(&file).is_ok_and(|t| looks_german(&t));
    if german {
        (Lang::De, store::play_file().is_none())
    } else {
        (Lang::En, false)
    }
}

/// Eine Aufgabendatei mit deutschen Überschriften.
pub fn looks_german(text: &str) -> bool {
    text.lines().any(|l| {
        let l = l.trim().to_lowercase();
        [
            "## themen",
            "## heute",
            "## offen",
            "## eingang",
            "## warte auf",
        ]
        .iter()
        .any(|h| l.starts_with(h))
    })
}

/// Befehle: (deutsch, englisch). Intern arbeitet Focus mit dem deutschen Wort;
/// gelesen werden beide, gezeigt wird das der gewählten Sprache.
pub const COMMANDS: &[(&str, &str)] = &[
    ("hilfe", "help"),
    ("einrichten", "setup"),
    ("anleitung", "manual"),
    ("deinstallieren", "uninstall"),
    ("datei", "file"),
    ("heute", "today"),
    ("neu", "new"),
    ("fertig", "done"),
    ("später", "later"),
    ("zurück", "restore"),
    ("archiv", "archive"),
    ("streichen", "drop"),
    ("ändern", "edit"),
    ("abgeben", "delegate"),
    ("jetzt", "now"),
    ("alles", "all"),
    ("themen", "topics"),
    ("personen", "people"),
    ("warten", "waiting"),
    ("suche", "search"),
    ("aufräumen", "review"),
    ("unterwegs", "mobile"),
    ("notiz", "note"),
    ("öffnen", "open"),
];

/// Weitere englische Wörter für dieselben Befehle und Wörter in Befehlen.
const ALIASES: &[(&str, &str)] = &[
    ("add", "neu"),
    ("undo", "zurück"),
    ("delete", "streichen"),
    ("change", "ändern"),
    ("find", "suche"),
    ("topic", "themen"),
    ("away", "unterwegs"),
    ("notes", "notiz"),
    ("handoff", "abgeben"),
    ("wait", "warten"),
    ("person", "personen"),
];

/// Wörter innerhalb von Befehlen: `file move`, `topics rename`, `mobile off` …
const WORDS: &[(&str, &str)] = &[
    ("move", "verschieben"),
    ("order", "ordnen"),
    ("sort", "ordnen"),
    ("rename", "umbenennen"),
    ("off", "aus"),
    ("delete", "löschen"),
    ("remove", "löschen"),
    ("instead", "statt"),
    ("no", "ohne"),
    ("without", "ohne"),
    ("topic", "thema"),
    ("date", "termin"),
    ("due", "termin"),
    ("repeat", "wiederholung"),
];

/// Das deutsche Befehlswort zu einem englischen; alles andere bleibt, wie es ist.
pub fn canonical(word: &str) -> String {
    let w = word.to_lowercase();
    COMMANDS
        .iter()
        .find(|(_, en)| *en == w)
        .map(|(de, _)| *de)
        .or_else(|| ALIASES.iter().find(|(en, _)| *en == w).map(|(_, de)| *de))
        .map(str::to_string)
        .unwrap_or(w)
}

/// Wie `canonical`, für Wörter innerhalb eines Befehls.
pub fn word(w: &str) -> String {
    let w = w.to_lowercase();
    WORDS
        .iter()
        .find(|(en, _)| *en == w)
        .map(|(_, de)| (*de).to_string())
        .unwrap_or(w)
}

/// Der Befehl in der gewählten Sprache: `cmd("heute")` ist auf Englisch „today“.
pub fn cmd(de: &str) -> &str {
    if !en() {
        return de;
    }
    COMMANDS
        .iter()
        .find(|(d, _)| *d == de)
        .map(|(_, e)| *e)
        .unwrap_or(de)
}

/// `tr!("Deutsch", "English")`, auch mit `{platzhaltern}` wie bei `format!`.
#[macro_export]
macro_rules! tr {
    ($de:literal, $en:literal $(,)?) => {
        if $crate::i18n::en() {
            format!($en)
        } else {
            format!($de)
        }
    };
    ($de:literal, $en:literal, $($arg:tt)+) => {
        if $crate::i18n::en() {
            format!($en, $($arg)+)
        } else {
            format!($de, $($arg)+)
        }
    };
}

/// Wie `tr!`, aber ohne Formatieren: für feste Texte als `&'static str`.
#[macro_export]
macro_rules! trs {
    ($de:literal, $en:literal $(,)?) => {
        if $crate::i18n::en() { $en } else { $de }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprachen_werden_verstanden() {
        assert_eq!(parse("EN"), Some(Lang::En));
        assert_eq!(parse("Deutsch"), Some(Lang::De));
        assert_eq!(parse("fr"), None);
    }

    #[test]
    fn englische_befehle_werden_zu_deutschen() {
        assert_eq!(canonical("Today"), "heute");
        assert_eq!(canonical("delete"), "streichen");
        assert_eq!(canonical("heute"), "heute");
        assert_eq!(canonical("#arbeit"), "#arbeit");
        assert_eq!(word("move"), "verschieben");
        assert_eq!(cmd("heute"), "heute");
    }

    #[test]
    fn eine_deutsche_datei_wird_erkannt() {
        assert!(looks_german("# Focus\n\n## Themen\n1. Arbeit\n"));
        assert!(!looks_german("# Focus\n\n## Topics\n1. Work\n"));
    }
}
