//! Die Unterwegs-Datei: oben der Stand zum Lesen am Handy, ganz unten ein
//! Abschnitt „Neu“ für das, was dir unterwegs einfällt.
//!
//! Sie ist eine Kopie, nie die Aufgabendatei selbst, und eine schlichte
//! Textdatei: Welcher Sync-Dienst sie zum Handy bringt (iCloud Drive, Google
//! Drive, Dropbox, Nextcloud, Syncthing …) und welcher Editor sie dort öffnet,
//! ist egal. Bei jedem Aufruf holt Focus zuerst die Zeilen unter „Neu“ als
//! Aufgaben ab und schreibt danach den Stand neu. Was zwischendurch unter „Neu“
//! dazukommt, bleibt stehen und kommt beim nächsten Mal dran.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{Datelike, NaiveDate};

use crate::store::{Status, Store, Task};
use crate::{config, date, i18n, ops, store, tags, term, tr, trs};

/// Der Dateiname für einen neu eingerichteten Ordner. Ein schon eingerichteter
/// Ort steht mit Dateinamen in den Einstellungen und bleibt.
pub fn file_name() -> &'static str {
    trs!("Focus-unterwegs.md", "Focus-mobile.md")
}

fn new_heading() -> &'static str {
    trs!("## Neu", "## New")
}

/// Welcher Dienst den Ordner zum Handy bringt, entscheidest du.
pub fn sync_hint() -> &'static str {
    trs!(
        "Den Ordner muss ein Sync-Dienst zum Handy bringen, etwa iCloud Drive, Google Drive, Dropbox, OneDrive, Nextcloud oder Syncthing.",
        "A sync service has to bring the folder to your phone, for example iCloud Drive, Google Drive, Dropbox, OneDrive, Nextcloud or Syncthing."
    )
}

/// Vor dem Datum oben in der Datei. Gelesen werden beide.
fn stand() -> &'static str {
    trs!("Stand: ", "As of: ")
}
const STANDS: [&str; 2] = ["Stand: ", "As of: "];

/// Wo die Unterwegs-Datei liegt, oder `None`, wenn es keine gibt.
///
/// `FOCUS_UNTERWEGS` geht vor (leer oder „aus“: keine). Mit einer Spieldatei in
/// `FOCUS_DATEI` gilt die Einstellung nicht: Sonst stünde Spielzeug am Handy.
pub fn path() -> Option<PathBuf> {
    if let Some(v) = std::env::var_os("FOCUS_UNTERWEGS") {
        let s = v.to_string_lossy();
        if s.trim().is_empty() || i18n::word(s.trim()) == "aus" {
            return None;
        }
        return Some(file_in(config::expand(s.trim())));
    }
    if store::play_file().is_some() {
        return None;
    }
    config::get(config::AWAY_KEY).map(file_in)
}

/// Ein Ordner bekommt den üblichen Dateinamen, eine `.md`-Datei bleibt, wie sie ist.
pub fn file_in(p: PathBuf) -> PathBuf {
    let is_md = p
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("txt"));
    if is_md && !p.is_dir() {
        p
    } else {
        p.join(file_name())
    }
}

/// Was beim Abholen herauskam.
#[derive(Default)]
pub struct Pickup {
    /// Die Zeilen unter „Neu“, die jetzt Aufgaben sind (so, wie sie dort standen).
    pub taken: Vec<String>,
    /// Bestätigungen zum Anzeigen.
    pub messages: Vec<String>,
    /// iCloud hat die Datei noch nicht auf den Mac geladen: diesmal nichts tun.
    pub pending: bool,
}

/// Holt die Zeilen unter „Neu“ als Aufgaben in die Aufgabendatei.
pub fn pick_up(store: &mut Store, path: &Path) -> Result<Pickup, String> {
    if still_in_cloud(path) {
        return Ok(Pickup {
            pending: true,
            messages: vec![tr!(
                "Die Unterwegs-Datei ist noch nicht aus iCloud geladen. Beim nächsten Aufruf hole ich sie ab.",
                "The mobile file isn't downloaded from iCloud yet. I'll pick it up next time."
            )],
            ..Pickup::default()
        });
    }
    let text = read(path)?;
    let mut pickup = Pickup::default();
    for raw in new_lines(&text) {
        let Some(entry) = clean(&raw) else {
            pickup.taken.push(raw);
            continue;
        };
        // Klappt das Aufschreiben nicht, bleibt die Zeile stehen.
        if let Ok(msg) = ops::add(store, &entry) {
            pickup.messages.push(msg);
            pickup.taken.push(raw);
        }
    }
    Ok(pickup)
}

/// Schreibt den Stand. Zeilen unter „Neu“, die nicht abgeholt wurden, bleiben
/// stehen; dafür liest Focus die Datei direkt vor dem Schreiben noch einmal.
/// `Ok(true)`, wenn sich etwas geändert hat.
pub fn write(store: &Store, path: &Path, taken: &[String]) -> Result<bool, String> {
    if still_in_cloud(path) {
        return Ok(false);
    }
    let dir = path.parent().unwrap_or(Path::new("."));
    if !dir.as_os_str().is_empty() && !dir.is_dir() {
        return Err(tr!(
            "Den Ordner für die Unterwegs-Datei gibt es nicht mehr: {}. Neu festlegen: focus unterwegs <ordner>, abschalten: focus unterwegs aus",
            "The folder for the mobile file is gone: {}. Set a new one: focus mobile <folder>, turn off: focus mobile off",
            dir.display()
        ));
    }
    let current = read(path)?;
    let mut keep = new_lines(&current);
    for t in taken {
        if let Some(i) = keep.iter().position(|k| k == t) {
            keep.remove(i);
        }
    }
    let now = chrono::Local::now().format("%H:%M").to_string();
    let text = render(store, &keep, &now);
    if without_stand(&text) == without_stand(&current) {
        return Ok(false);
    }
    let tmp = dir.join(format!(
        ".{}.neu",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(file_name())
    ));
    fs::write(&tmp, &text)
        .and_then(|()| fs::rename(&tmp, path))
        .map_err(|e| {
            tr!(
                "Ich kann {} nicht schreiben: {e}",
                "I can't write {}: {e}",
                path.display()
            )
        })?;
    Ok(true)
}

fn read(path: &Path) -> Result<String, String> {
    match fs::read_to_string(path) {
        Ok(s) => Ok(s),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(tr!(
            "Ich kann {} nicht lesen: {e}",
            "I can't read {}: {e}",
            path.display()
        )),
    }
}

/// iCloud ersetzt nicht geladene Dateien durch `.Name.icloud`. Dann wird sie
/// angefordert, und Focus lässt sie diesmal in Ruhe, statt sie für leer zu halten.
fn still_in_cloud(path: &Path) -> bool {
    let (Some(dir), Some(name)) = (path.parent(), path.file_name()) else {
        return false;
    };
    let stub = dir.join(format!(".{}.icloud", name.to_string_lossy()));
    if path.exists() || !stub.exists() {
        return false;
    }
    let _ = Command::new("brctl").arg("download").arg(path).output();
    true
}

/// Die nicht leeren Zeilen unter „## Neu“, bis zur nächsten Überschrift oder zum
/// Ende der Datei. Steht „Neu“ (wie früher) oben, endet es an „## Heute“ und Co.
fn new_lines(text: &str) -> Vec<String> {
    let mut inside = false;
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if let Some(h) = t.strip_prefix("## ") {
            let h = h.trim().to_lowercase();
            inside = h.starts_with("neu") || h.starts_with("new");
            continue;
        }
        if inside && !t.is_empty() {
            out.push(t.to_string());
        }
    }
    out
}

/// Aus „- [ ] Trikots bestellen #verein freitag“ wird, was `focus +` bekäme.
/// `None` für Zeilen ohne Inhalt, etwa ein einzelnes „-“.
fn clean(line: &str) -> Option<String> {
    let mut s = line.trim();
    for prefix in ["- [ ]", "* [ ]", "- [x]", "-", "*", "•", "+", "–"] {
        // Nur als eigenes Zeichen vorn: „+49 anrufen“ bleibt, wie es ist.
        if let Some(rest) = s.strip_prefix(prefix)
            && (rest.is_empty() || rest.starts_with(char::is_whitespace))
        {
            s = rest.trim_start();
            break;
        }
    }
    let s = s.trim();
    (!s.is_empty()).then(|| s.to_string())
}

/// Ohne die Uhrzeit hinter „Stand:“: Die allein ist kein Grund zum Schreiben,
/// ein neuer Tag schon.
fn without_stand(text: &str) -> String {
    text.lines()
        .map(|l| {
            match STANDS
                .iter()
                .find_map(|p| l.strip_prefix(p))
                .and_then(|r| r.rsplit_once(", "))
            {
                Some((day, _)) => day,
                None => l,
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Die ganze Datei. Termine stehen mit Datum da („fällig Samstag 28.11.“),
/// nicht als „morgen“: Am Handy liest man sie vielleicht erst übermorgen.
pub fn render(store: &Store, keep: &[String], now: &str) -> String {
    let today = store.today;
    let mut s = String::from(trs!("# Focus unterwegs\n\n", "# Focus mobile\n\n"));
    s.push_str(&format!("{}{}, {now}\n\n", stand(), date::exact(today)));
    s.push_str(trs!(
        "Neues schreibst du ganz unten unter „Neu“: eine Zeile je Aufgabe, wie bei\n\
         „focus +“, etwa „Trikots bestellen #verein freitag“. Focus übernimmt sie beim\n\
         nächsten Aufruf am Computer. Alles darüber schreibt Focus jedes Mal neu;\n\
         abhaken und ändern geht am Computer.\n\n",
        "Write new things at the very bottom under “New”: one line per task, like\n\
         with “focus +”, for example “order jerseys #club friday”. Focus picks them up\n\
         the next time it runs on your computer. Focus rewrites everything above each\n\
         time; checking off and changing happens on the computer.\n\n"
    ));

    let done = store.done_today();
    let open_today = store.today_open();
    if !done.is_empty() || !open_today.is_empty() {
        s.push_str(&format!(
            "## {} · {}\n",
            trs!("Heute", "Today"),
            date::exact(today)
        ));
        for &i in &done {
            s.push_str(&format!("- ✓ {}\n", store.tasks[i].display()));
            s.push_str(&note_lines(&store.tasks[i].notes));
        }
        for &i in &open_today {
            s.push_str(&item(store, &store.tasks[i], true));
        }
        s.push('\n');
    }

    let mut soon: Vec<&Task> = store
        .tasks
        .iter()
        .filter(|t| t.status == Status::Open && store.is_urgent(t))
        .collect();
    soon.sort_by_key(|t| (t.due, t.nr));
    if !soon.is_empty() {
        s.push_str(trs!("## Bald fällig\n", "## Due soon\n"));
        for t in &soon {
            s.push_str(&item(store, t, true));
        }
        s.push('\n');
    }

    let mut waiting: Vec<&Task> = store
        .tasks
        .iter()
        .filter(|t| t.status == Status::Waiting)
        .collect();
    waiting.sort_by_key(|t| (t.since, t.nr));
    if !waiting.is_empty() {
        s.push_str(trs!("## Warte auf\n", "## Waiting for\n"));
        for t in &waiting {
            let mut line = format!("- {} {}", t.nr, t.display());
            if let Some(p) = &t.waiting_for {
                line.push_str(&tr!(" · bei {}", " · with {}", tags::display(p)));
            }
            line.push_str(&tr!(" seit {}", " since {}", short_date(t.since, today)));
            if let Some(d) = t.due {
                line.push_str(&format!(" · {}", due_text(t, d, today)));
            }
            s.push_str(&line);
            s.push('\n');
            s.push_str(&note_lines(&t.notes));
        }
        s.push('\n');
    }

    let rest: Vec<&Task> = store
        .tasks
        .iter()
        .filter(|t| t.status == Status::Open && !store.is_urgent(t))
        .collect();
    if !rest.is_empty() {
        s.push_str(trs!("## Offen\n", "## Open\n"));
        let mut groups: Vec<(Option<String>, bool)> = store
            .important
            .iter()
            .map(|n| (Some(n.clone()), true))
            .chain(store.other.iter().map(|n| (Some(n.clone()), false)))
            .collect();
        groups.push((None, false));
        for (name, important) in groups {
            let key = name.as_deref().map(tags::key);
            let mut tasks: Vec<&&Task> = rest
                .iter()
                .filter(|t| t.topic_tag().map(|tag| tags::key(&tag)) == key)
                .collect();
            if tasks.is_empty() {
                continue;
            }
            tasks.sort_by_key(|t| (t.due.is_none(), t.due, t.nr));
            let title = match &name {
                Some(n) if important => tr!("{n} · wichtig", "{n} · important"),
                Some(n) => n.clone(),
                None => tr!("Ohne Thema", "No topic"),
            };
            s.push_str(&format!("\n### {title}\n"));
            if let Some(n) = &name {
                for note in store.notes_of_topic(n) {
                    s.push_str(&format!("{note}\n"));
                }
            }
            for t in tasks {
                s.push_str(&item(store, t, false));
            }
        }
        s.push('\n');
    }

    if !store.tasks.iter().any(Task::is_active) {
        s.push_str(trs!(
            "Gerade ist nichts offen.\n\n",
            "Nothing is open right now.\n\n"
        ));
    }

    // „Neu“ steht am Ende: Was Kurzbefehle oder andere Apps an die Datei
    // anhängen, landet so von selbst dort.
    let mut s = s.trim_end().to_string();
    s.push_str("\n\n");
    s.push_str(new_heading());
    s.push('\n');
    for k in keep {
        s.push_str(k);
        s.push('\n');
    }
    s
}

fn item(store: &Store, t: &Task, with_topic: bool) -> String {
    let mut line = format!("- {} {}", t.nr, t.display());
    if with_topic && let Some(n) = store.topic_name(t) {
        line.push_str(&format!(" · {n}"));
    }
    if let Some(d) = t.due {
        line.push_str(&format!(" · {}", due_text(t, d, store.today)));
    }
    line.push('\n');
    line.push_str(&note_lines(&t.notes));
    line
}

/// Notizen eingerückt unter der Aufgabe oder dem Thema: Unterwegs ist das der
/// Zusammenhang, der sonst fehlt.
fn note_lines(notes: &[String]) -> String {
    notes.iter().map(|n| format!("  {n}\n")).collect()
}

fn due_text(t: &Task, d: NaiveDate, today: NaiveDate) -> String {
    let when = if d.year() == today.year() {
        date::exact(d)
    } else {
        format!("{} {}", date::weekday(d), date::to_file(d))
    };
    match t.every {
        Some(n) => tr!(
            "↻ {}, nächstes Mal {when}",
            "↻ {}, next time {when}",
            date::recurrence_label(n, d)
        ),
        None if d < today => tr!("überfällig seit {when}", "overdue since {when}"),
        None => tr!("fällig {when}", "due {when}"),
    }
}

fn short_date(d: NaiveDate, today: NaiveDate) -> String {
    if d.year() == today.year() {
        date::short(d)
    } else {
        date::to_file(d)
    }
}

/// `focus unterwegs`, `focus unterwegs <ordner>`, `focus unterwegs aus`
pub fn command(args: &[String]) -> Result<String, String> {
    let arg = args.join(" ");
    let arg = arg.trim();
    if arg.is_empty() {
        return Ok(match path() {
            Some(p) => tr!(
                " Die Unterwegs-Datei liegt in {}\n {}",
                " The mobile file is in {}\n {}",
                p.display(),
                term::dim(trs!(
                    "Anderer Ort: focus unterwegs <ordner> · abschalten: focus unterwegs aus",
                    "Somewhere else: focus mobile <folder> · turn off: focus mobile off"
                ))
            ),
            None => tr!(
                " Es gibt noch keine Unterwegs-Datei.\n {}",
                " There's no mobile file yet.\n {}",
                term::dim(trs!(
                    "Einschalten: focus unterwegs <ordner>, ein Ordner, den auch dein Handy sieht",
                    "Turn on: focus mobile <folder>, a folder your phone can see too"
                ))
            ),
        });
    }
    if std::env::var_os("FOCUS_UNTERWEGS").is_some() {
        return Err(tr!(
            "Gerade gilt FOCUS_UNTERWEGS. Ohne die Variable noch einmal versuchen.",
            "FOCUS_UNTERWEGS is in effect. Try again without the variable."
        ));
    }
    if let Some(p) = store::play_file() {
        return Err(tr!(
            "Gerade gilt die Spieldatei FOCUS_DATEI={}. Die Unterwegs-Datei gehört zu deinen echten Aufgaben; ohne FOCUS_DATEI noch einmal versuchen.",
            "The practice file FOCUS_DATEI={} is in effect. The mobile file belongs to your real tasks; try again without FOCUS_DATEI.",
            p.display()
        ));
    }
    if i18n::word(arg) == "aus" {
        let before = path();
        config::set(config::AWAY_KEY, None)?;
        return Ok(match before {
            Some(p) => tr!(
                " {} Focus schreibt keine Unterwegs-Datei mehr. Die letzte bleibt in {}",
                " {} Focus no longer writes a mobile file. The last one stays in {}",
                term::green("✓"),
                p.display()
            ),
            None => tr!(
                " Es gab keine Unterwegs-Datei.",
                " There was no mobile file."
            ),
        });
    }
    let target = file_in(config::typed_path(arg).ok_or_else(|| {
        tr!(
            "Wohin? Zum Beispiel: focus unterwegs ~/Dropbox/Focus",
            "Where to? For example: focus mobile ~/Dropbox/Focus"
        )
    })?);
    set(&target)?;
    let mut msg = tr!(
        " {} Focus schreibt den Stand jetzt nach {}\n   {}",
        " {} Focus now writes the state to {}\n   {}",
        term::green("✓"),
        target.display(),
        term::dim(trs!(
            "Am Handy ganz unten unter „Neu“ aufschreiben; beim nächsten Aufruf hier holt Focus es ab.",
            "On your phone, write at the very bottom under “New”; Focus picks it up the next time it runs here."
        ))
    );
    msg.push_str(&format!("\n   {}", term::dim(sync_hint())));
    Ok(msg)
}

/// Legt den Ort fest (auch aus `focus einrichten`). Den Ordner legt Focus an.
pub fn set(target: &Path) -> Result<(), String> {
    if same_file(target, &store::file_path()) {
        return Err(tr!(
            "Das ist deine Aufgabendatei selbst. Die Unterwegs-Datei braucht einen eigenen Ort.",
            "That's your task file itself. The mobile file needs a place of its own."
        ));
    }
    if let Some(dir) = target.parent() {
        fs::create_dir_all(dir).map_err(|e| crate::setup::cant_create(dir, &e))?;
    }
    config::set(config::AWAY_KEY, Some(target))
}

/// Gleich, auch wenn einer der Pfade über einen Link oder `..` geht.
pub fn same_file(a: &Path, b: &Path) -> bool {
    let canon = |p: &Path| {
        p.canonicalize().unwrap_or_else(|_| {
            match (
                p.parent().and_then(|d| d.canonicalize().ok()),
                p.file_name(),
            ) {
                (Some(d), Some(n)) => d.join(n),
                _ => p.to_path_buf(),
            }
        })
    };
    canon(a) == canon(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neu_am_ende_reicht_bis_zum_dateiende() {
        let text = "# Focus unterwegs\n\n## Heute · Freitag 27.11.\n- 2 Präsentation\n\n## Offen\n\n### Arbeit · wichtig\n- 5 Texte\n\n## Neu\n- Trikots bestellen #verein\nper Kurzbefehl angehängt";
        assert_eq!(
            new_lines(text),
            vec!["- Trikots bestellen #verein", "per Kurzbefehl angehängt"]
        );
    }

    #[test]
    fn nur_zeilen_unter_neu_zaehlen() {
        let text = "# Focus unterwegs\n\nWas dir einfällt …\n\n## Neu\n\n- Trikots bestellen #verein\nIdee ohne Strich\n\n## Heute · Freitag 27.11.\n- 2 Präsentation\n";
        assert_eq!(
            new_lines(text),
            vec!["- Trikots bestellen #verein", "Idee ohne Strich"]
        );
        assert!(new_lines("## Heute\n- 1 A\n").is_empty());
    }

    #[test]
    fn striche_und_kaestchen_fallen_weg() {
        assert_eq!(clean("- [ ] Laufen gehen").as_deref(), Some("Laufen gehen"));
        assert_eq!(clean("* Brot").as_deref(), Some("Brot"));
        assert_eq!(clean("+ Brot morgen").as_deref(), Some("Brot morgen"));
        assert_eq!(clean("-"), None);
        assert_eq!(clean("+49 anrufen").as_deref(), Some("+49 anrufen"));
        assert_eq!(clean("Einfach so").as_deref(), Some("Einfach so"));
    }

    #[test]
    fn was_zwischendurch_dazukam_bleibt_unter_neu_stehen() {
        let dir = std::env::temp_dir().join(format!("focus-unterwegs-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join(file_name());
        // Abgeholt wurde „A“; während Focus lief, kam am Handy „B“ dazu.
        fs::write(&file, "# Focus unterwegs\n\n## Neu\n\n- A\n- B\n").unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 11, 27).unwrap();
        let store = Store::parse("## Offen\n- [ ] A\n", dir.join("Focus.md"), today);
        assert!(write(&store, &file, &["- A".to_string()]).unwrap());
        let text = fs::read_to_string(&file).unwrap();
        assert_eq!(new_lines(&text), vec!["- B"]);
        assert!(text.contains("- 1 A"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn ein_ordner_bekommt_den_dateinamen() {
        assert_eq!(
            file_in(PathBuf::from("/nirgends/Focus")),
            PathBuf::from("/nirgends/Focus/Focus-unterwegs.md")
        );
        assert_eq!(
            file_in(PathBuf::from("/nirgends/Liste.md")),
            PathBuf::from("/nirgends/Liste.md")
        );
    }

    #[test]
    fn die_uhrzeit_allein_ist_keine_aenderung() {
        let a = "# x\n\nStand: Freitag 27.11., 06:10\n\n## Neu\n";
        let b = "# x\n\nStand: Freitag 27.11., 07:45\n\n## Neu\n";
        let c = "# x\n\nStand: Samstag 28.11., 06:10\n\n## Neu\n";
        assert_eq!(without_stand(a), without_stand(b));
        assert_ne!(without_stand(a), without_stand(c));
    }
}
