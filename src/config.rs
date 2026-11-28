//! Die Einstellungen: wo die Aufgabendatei liegt und wo die Unterwegs-Datei.
//!
//! Eine kleine Textdatei, `~/.config/focus/einstellungen`, eine Zeile je Wert:
//!
//! ```text
//! datei = ~/Dokumente/Focus.md
//! unterwegs = ~/Library/Mobile Documents/com~apple~CloudDocs/Focus/Focus-unterwegs.md
//! ```
//!
//! Sie steht nicht in der Aufgabendatei selbst: Focus muss den Ort kennen, bevor
//! es die Datei lesen kann. Zeilen, die Focus nicht kennt, bleiben stehen.

use crate::{tr, trs};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub const DATA_KEY: &str = "datei";
pub const AWAY_KEY: &str = "unterwegs";
/// `leeren = ja`: vor jeder Ausgabe den sichtbaren Bereich leeren, mit Kopfzeile.
pub const CLEAR_KEY: &str = "leeren";

fn header() -> &'static str {
    trs!(
        "# Focus-Einstellungen. Ändern geht am einfachsten mit „focus einrichten“.\n",
        "# Focus settings. The easiest way to change them is “focus setup”.\n"
    )
}
/// Beide Kopfzeilen, damit ein Wechsel der Sprache keine zweite einträgt.
const HEADERS: [&str; 2] = [
    "# Focus-Einstellungen. Ändern geht am einfachsten mit „focus einrichten“.",
    "# Focus settings. The easiest way to change them is “focus setup”.",
];

/// `FOCUS_EINSTELLUNGEN`, sonst `$XDG_CONFIG_HOME/focus/einstellungen`, sonst `~/.config/…`.
pub fn path() -> PathBuf {
    if let Some(p) = std::env::var_os("FOCUS_EINSTELLUNGEN") {
        return PathBuf::from(p);
    }
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".config"));
    let new = base.join("focus").join("einstellungen");
    // Aus der Zeit, als Focus „fokus“ hieß: Die Einstellungen bleiben, wo sie sind.
    let old = base.join("fokus").join("einstellungen");
    if !new.exists() && old.exists() {
        old
    } else {
        new
    }
}

pub fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Ein Wert aus den Einstellungen, mit `~` aufgelöst. Leer zählt als nicht gesetzt.
pub fn get(key: &str) -> Option<PathBuf> {
    let text = fs::read_to_string(path()).ok()?;
    value_in(&text, key).map(|v| expand(&v))
}

/// Ein Wert als Text, wie er dasteht.
pub fn get_str(key: &str) -> Option<String> {
    let text = fs::read_to_string(path()).ok()?;
    value_in(&text, key)
}

pub fn set_str(key: &str, value: Option<&str>) -> Result<(), String> {
    set_text(key, value)
}

/// Ein Ja/Nein-Wert. Fehlt er, gilt nein.
pub fn flag(key: &str) -> bool {
    fs::read_to_string(path())
        .ok()
        .and_then(|t| value_in(&t, key))
        .is_some_and(|v| matches!(v.to_lowercase().as_str(), "ja" | "yes" | "true" | "1"))
}

pub fn set_flag(key: &str, on: bool) -> Result<(), String> {
    set_text(key, Some(if on { "ja" } else { "nein" }))
}

/// Setzt einen Wert (`None` entfernt ihn) und lässt alles andere stehen.
pub fn set(key: &str, value: Option<&Path>) -> Result<(), String> {
    let value = value.map(shorten);
    set_text(key, value.as_deref())
}

fn set_text(key: &str, value: Option<&str>) -> Result<(), String> {
    let file = path();
    let existing = match fs::read_to_string(&file) {
        Ok(s) => s,
        Err(e) if e.kind() == ErrorKind::NotFound => String::new(),
        Err(e) => {
            return Err(tr!(
                "Ich kann {} nicht lesen: {e}",
                "I can't read {}: {e}",
                file.display()
            ));
        }
    };
    let updated = with_value(&existing, key, value);
    if updated == existing {
        return Ok(());
    }
    if updated.is_empty() {
        return match fs::remove_file(&file) {
            Err(e) if e.kind() != ErrorKind::NotFound => Err(tr!(
                "Ich kann {} nicht löschen: {e}",
                "I can't delete {}: {e}",
                file.display()
            )),
            _ => Ok(()),
        };
    }
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir).map_err(|e| {
            tr!(
                "Ich kann {} nicht anlegen: {e}",
                "I can't create {}: {e}",
                dir.display()
            )
        })?;
    }
    fs::write(&file, updated).map_err(|e| {
        tr!(
            "Ich kann {} nicht schreiben: {e}",
            "I can't write {}: {e}",
            file.display()
        )
    })
}

fn value_in(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let (k, v) = line.split_once('=')?;
        (!line.trim_start().starts_with('#') && k.trim().eq_ignore_ascii_case(key))
            .then(|| v.trim().to_string())
            .filter(|v| !v.is_empty())
    })
}

fn with_value(existing: &str, key: &str, value: Option<&str>) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut done = false;
    for line in existing.lines() {
        let is_key = !line.trim_start().starts_with('#')
            && line
                .split_once('=')
                .is_some_and(|(k, _)| k.trim().eq_ignore_ascii_case(key));
        if !is_key {
            lines.push(line.to_string());
        } else if !done && let Some(v) = value {
            lines.push(format!("{key} = {v}"));
            done = true;
        }
    }
    if !done && let Some(v) = value {
        lines.push(format!("{key} = {v}"));
    }
    // Steht nur noch die eigene Kopfzeile da, ist die Datei leer.
    if lines
        .iter()
        .all(|l| l.trim().is_empty() || HEADERS.contains(&l.as_str()))
    {
        return String::new();
    }
    if existing.trim().is_empty() {
        lines.insert(0, header().trim_end().to_string());
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// `~/x` wird zu `$HOME/x`.
pub fn expand(s: &str) -> PathBuf {
    if s == "~" {
        return home();
    }
    match s.strip_prefix("~/") {
        Some(rest) => home().join(rest),
        None => PathBuf::from(s),
    }
}

/// Umgekehrt: Pfade unter `$HOME` mit `~`, damit die Datei lesbar bleibt.
pub fn shorten(p: &Path) -> String {
    match p.strip_prefix(home()) {
        Ok(rest) if !rest.as_os_str().is_empty() => format!("~/{}", rest.display()),
        _ => p.display().to_string(),
    }
}

/// Was jemand als Ort eintippt oder aus dem Finder ins Terminal zieht:
/// Anführungszeichen weg, `\ ` wird zu Leerzeichen, `~` aufgelöst, relativ zum
/// aktuellen Ordner.
pub fn typed_path(input: &str) -> Option<PathBuf> {
    let mut s = input.trim().to_string();
    for q in ['\'', '"'] {
        if s.len() >= 2 && s.starts_with(q) && s.ends_with(q) {
            s = s[1..s.len() - 1].to_string();
        }
    }
    let s = s.replace("\\ ", " ");
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let p = expand(s);
    if p.is_absolute() {
        Some(p)
    } else {
        std::env::current_dir().ok().map(|d| d.join(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ein_wert_wird_ersetzt_und_der_rest_bleibt() {
        let alt = "# meine Notiz\ndatei = ~/a.md\nfarbe = blau\n";
        let neu = with_value(alt, "datei", Some("~/b.md"));
        assert_eq!(neu, "# meine Notiz\ndatei = ~/b.md\nfarbe = blau\n");
        assert_eq!(value_in(&neu, "datei").as_deref(), Some("~/b.md"));
        let ohne = with_value(&neu, "datei", None);
        assert_eq!(ohne, "# meine Notiz\nfarbe = blau\n");
    }

    #[test]
    fn eine_neue_datei_bekommt_eine_kopfzeile() {
        let neu = with_value("", "unterwegs", Some("/x/F.md"));
        assert!(neu.starts_with("# Focus-Einstellungen"));
        assert!(neu.ends_with("unterwegs = /x/F.md\n"));
        assert_eq!(with_value(&neu, "unterwegs", None), "");
    }

    #[test]
    fn auskommentiertes_zaehlt_nicht() {
        assert_eq!(value_in("# datei = /x\n", "datei"), None);
        assert_eq!(value_in("datei =   \n", "datei"), None);
    }

    #[test]
    fn aus_dem_finder_gezogene_pfade_werden_verstanden() {
        assert_eq!(
            typed_path("/Users/s/Focus\\ unterwegs "),
            Some(PathBuf::from("/Users/s/Focus unterwegs"))
        );
        assert_eq!(
            typed_path("'/Users/s/Mein Ordner'"),
            Some(PathBuf::from("/Users/s/Mein Ordner"))
        );
        assert_eq!(typed_path("   "), None);
    }
}
