//! Die Aufgabendatei: lesen, in Ordnung bringen, schreiben.
//!
//! Die Datei ist Markdown, damit man sie mit jedem Texteditor lesen und ändern
//! kann. Der Abschnitt bestimmt den Zustand einer Aufgabe (`## Heute`,
//! `## Warte auf`, …), die eckige Klammer am Zeilenende trägt Nummer und Daten.

use std::collections::HashSet;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;

use crate::{date, tags};

/// Höchstens so viele offene Aufgaben stehen auf heute.
pub const TODAY_LIMIT: usize = 3;
/// Höchstens so viele Themen stehen über der Linie.
pub const IMPORTANT_LIMIT: usize = 3;
/// Dringend ist, was in so vielen Tagen fällig ist (oder schon überfällig).
pub const URGENT_DAYS: i64 = 2;
/// Ab so vielen Tagen ohne Bewegung fragt `fokus aufräumen` nach.
pub const STALE_OPEN_DAYS: i64 = 14;
pub const STALE_WAITING_DAYS: i64 = 7;
/// Unwichtiges ohne Termin wandert nach so vielen Tagen ins Archiv.
pub const ARCHIVE_DAYS: i64 = 30;
/// So lange stehen Erledigtes und Gestrichenes im Archiv; danach löscht Fokus sie.
pub const KEEP_DAYS: i64 = 14;

const HEADER: &str = "# Fokus\n\n\
Deine Aufgaben. Du darfst diese Datei mit jedem Texteditor ändern;\n\
Fokus bringt sie beim nächsten Aufruf wieder in Ordnung.\n\n";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Today,
    Open,
    Waiting,
    Done,
    Archived,
}

#[derive(Clone, Debug)]
pub struct Task {
    /// Nummer für die Befehle; nur offene Aufgaben haben eine (sonst 0).
    pub nr: u32,
    /// Der Text, wie er eingegeben wurde, mit `#thema` und `@person`.
    pub text: String,
    pub status: Status,
    pub due: Option<NaiveDate>,
    /// Seit wann sich an der Aufgabe nichts getan hat.
    pub since: NaiveDate,
    pub waiting_for: Option<String>,
    pub done_on: Option<NaiveDate>,
    /// Nur bei erledigten: stand heute auf der Liste für heute.
    pub on_today: bool,
    pub archived_on: Option<NaiveDate>,
    /// Wiederholung: alle so viele Wochen, am Wochentag des Termins.
    pub every: Option<u32>,
}

impl Task {
    pub fn new(text: impl Into<String>, due: Option<NaiveDate>, today: NaiveDate) -> Self {
        Task {
            nr: 0,
            text: text.into(),
            status: Status::Open,
            due,
            since: today,
            waiting_for: None,
            done_on: None,
            on_today: false,
            archived_on: None,
            every: None,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, Status::Today | Status::Open | Status::Waiting)
    }

    /// Seit wie vielen Tagen erledigt oder gestrichen (sonst `None`).
    pub fn inactive_age(&self, today: NaiveDate) -> Option<i64> {
        match self.status {
            Status::Done => self.done_on.map(|d| (today - d).num_days()),
            Status::Archived => self.archived_on.map(|d| (today - d).num_days()),
            _ => None,
        }
    }

    /// Steht im Archiv und lässt sich mit `fokus zurück` holen.
    pub fn is_restorable(&self, today: NaiveDate) -> bool {
        self.inactive_age(today).is_some_and(|a| a <= KEEP_DAYS)
    }

    /// Das erste `#thema` im Text.
    pub fn topic_tag(&self) -> Option<String> {
        self.text
            .split_whitespace()
            .find(|w| tags::is_tag(w, '#'))
            .map(|w| tags::clean(w).to_string())
    }

    /// Alle `@personen` im Text, ohne Doppelte.
    pub fn persons(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for w in self
            .text
            .split_whitespace()
            .filter(|w| tags::is_tag(w, '@'))
        {
            let p = tags::clean(w).to_string();
            if !out.iter().any(|o| tags::key(o) == tags::key(&p)) {
                out.push(p);
            }
        }
        out
    }

    pub fn mentions(&self, person_key: &str) -> bool {
        self.persons().iter().any(|p| tags::key(p) == person_key)
    }

    /// Der Text zum Anzeigen: ohne `#thema` (das steht in einer eigenen Spalte).
    pub fn display(&self) -> String {
        let words: Vec<&str> = self
            .text
            .split_whitespace()
            .filter(|w| !tags::is_tag(w, '#'))
            .collect();
        if words.is_empty() {
            self.text.clone()
        } else {
            words.join(" ")
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Preamble,
    Topics,
    Today,
    Waiting,
    Inbox,
    Open,
    Done,
    Archive,
    Notes,
    Unknown,
}

pub struct Store {
    pub path: PathBuf,
    /// Themen über der Linie, in ihrer Reihenfolge. Höchstens drei.
    pub important: Vec<String>,
    /// Themen unter der Linie.
    pub other: Vec<String>,
    pub today: NaiveDate,
    /// Die Liste für heute ist noch Fokus' Vorschlag.
    pub suggested: bool,
    pub tasks: Vec<Task>,
    /// Zeilen, die Fokus nicht versteht. Sie bleiben unter „Notizen“ erhalten.
    pub notes: Vec<String>,
    /// Hinweise, die beim Laden entstanden sind (etwa: ins Archiv geräumt).
    pub messages: Vec<String>,
    original: String,
}

/// `~/Documents/Fokus.md`, oder was in `FOKUS_DATEI` steht.
pub fn file_path() -> PathBuf {
    if let Some(p) = std::env::var_os("FOKUS_DATEI") {
        return PathBuf::from(p);
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join("Documents").join("Fokus.md")
}

impl Store {
    pub fn load(path: PathBuf, today: NaiveDate) -> Result<Store, String> {
        let text = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) if e.kind() == ErrorKind::NotFound => String::new(),
            Err(e) => return Err(format!("Ich kann {} nicht lesen: {e}", path.display())),
        };
        Ok(Store::parse(&text, path, today))
    }

    pub fn parse(text: &str, path: PathBuf, today: NaiveDate) -> Store {
        let mut store = Store {
            path,
            important: Vec::new(),
            other: Vec::new(),
            today,
            suggested: false,
            tasks: Vec::new(),
            notes: Vec::new(),
            messages: Vec::new(),
            original: text.to_string(),
        };
        let mut section = Section::Preamble;
        let mut below_line = false;
        let mut list_date: Option<NaiveDate> = None;
        let mut topics: Vec<(String, bool)> = Vec::new();

        for line in text.lines() {
            let trimmed = line.trim();
            if let Some(heading) = trimmed.strip_prefix("## ") {
                section = section_of(heading);
                if section == Section::Today {
                    list_date = heading.split('·').find_map(date::from_file);
                    store.suggested = heading.to_lowercase().contains("vorschlag");
                }
                continue;
            }
            match section {
                Section::Preamble => {}
                Section::Topics => {
                    if trimmed.is_empty() {
                        continue;
                    }
                    if trimmed.len() >= 3
                        && trimmed.chars().all(|c| matches!(c, '-' | '─' | '*' | '_'))
                    {
                        below_line = true;
                        continue;
                    }
                    let name = strip_marker(trimmed);
                    if !tags::key(name).is_empty() {
                        topics.push((name.to_string(), below_line));
                    }
                }
                Section::Notes => store.notes.push(line.to_string()),
                _ => {
                    if trimmed.is_empty() {
                        continue;
                    }
                    match parse_task(trimmed, section, list_date.unwrap_or(today), today) {
                        Some(t) => store.tasks.push(t),
                        None => store.notes.push(line.to_string()),
                    }
                }
            }
        }

        for (name, below) in topics {
            let k = tags::key(&name);
            if store.all_topics().any(|n| tags::key(n) == k) {
                continue;
            }
            if !below && store.important.len() < IMPORTANT_LIMIT {
                store.important.push(name);
            } else {
                store.other.push(name);
            }
        }

        store.normalize(list_date);
        store
    }

    /// Alles, was beim Laden geradegerückt wird: neuer Tag, Archiv, Nummern, Themen.
    fn normalize(&mut self, list_date: Option<NaiveDate>) {
        let today = self.today;

        // Wer „wartet“, braucht jemanden, auf den gewartet wird.
        for t in &mut self.tasks {
            if t.status == Status::Waiting && t.waiting_for.is_none() {
                match t.persons().into_iter().next() {
                    Some(p) => t.waiting_for = Some(p),
                    None => t.status = Status::Open,
                }
            }
            if t.status == Status::Done && t.done_on != Some(today) {
                t.on_today = false;
            }
        }

        // Nach 14 Tagen im Archiv ist etwas endgültig weg.
        self.tasks
            .retain(|t| t.inactive_age(today).is_none_or(|a| a <= KEEP_DAYS));

        // Verpasste Wiederholung: Ist der nächste Termin erreicht, rückt sie dorthin.
        for t in &mut self.tasks {
            if t.is_active()
                && let Some(n) = t.every
            {
                let step = chrono::Days::new(7 * n as u64);
                let mut due = t.due.unwrap_or(today);
                while due + step <= today {
                    due = due + step;
                }
                t.due = Some(due);
            }
        }

        // Ein neuer Tag: Liegengebliebenes zurück, dann ein frischer Vorschlag.
        let new_day = list_date != Some(today);
        let mut leftovers = Vec::new();
        if new_day {
            for (i, t) in self.tasks.iter_mut().enumerate() {
                match t.status {
                    Status::Today => {
                        t.status = Status::Open;
                        leftovers.push(i);
                    }
                    Status::Done => t.on_today = false,
                    _ => {}
                }
            }
        }

        // Unwichtig, ohne Termin und lange nicht angefasst: ins Archiv.
        let mut archived = 0;
        for i in 0..self.tasks.len() {
            let t = &self.tasks[i];
            if t.status == Status::Open
                && t.due.is_none()
                && !self.is_important(t)
                && (today - t.since).num_days() >= ARCHIVE_DAYS
            {
                let t = &mut self.tasks[i];
                t.status = Status::Archived;
                t.archived_on = Some(today);
                archived += 1;
            }
        }
        if archived == 1 {
            self.messages.push(format!(
                " 1 unwichtige Aufgabe lag über {ARCHIVE_DAYS} Tage herum. Sie steht jetzt im Archiv: fokus archiv"
            ));
        } else if archived > 1 {
            self.messages.push(format!(
                " {archived} unwichtige Aufgaben lagen über {ARCHIVE_DAYS} Tage herum. Sie stehen jetzt im Archiv: fokus archiv"
            ));
        }

        // Themen, die in Aufgaben vorkommen, aber nicht in der Liste stehen.
        let tags_in_use: Vec<String> = self
            .tasks
            .iter()
            .filter(|t| t.is_active())
            .filter_map(Task::topic_tag)
            .collect();
        for tag in &tags_in_use {
            let k = tags::key(tag);
            if !self.all_topics().any(|n| tags::key(n) == k) {
                self.other.push(tags::display(tag));
            }
        }
        // Unter der Linie bleibt nur, was noch Aufgaben hat.
        let keys: HashSet<String> = tags_in_use.iter().map(|t| tags::key(t)).collect();
        self.other.retain(|n| keys.contains(&tags::key(n)));

        self.assign_numbers();
        if new_day {
            self.propose(&leftovers);
        }
    }

    /// Fokus' Vorschlag für heute: dringend und wichtig, dann was gestern liegen
    /// blieb, dann wichtig ohne Eile. Unwichtiges schlägt Fokus nie vor.
    pub fn suggestion(&self, leftovers: &[usize]) -> Vec<usize> {
        let open: Vec<usize> = (0..self.tasks.len())
            .filter(|&i| self.tasks[i].status == Status::Open)
            .collect();
        let mut urgent: Vec<usize> = open
            .iter()
            .copied()
            .filter(|&i| self.is_important(&self.tasks[i]) && self.is_urgent(&self.tasks[i]))
            .collect();
        urgent.sort_by_key(|&i| self.tasks[i].due);
        let mut calm: Vec<usize> = open
            .iter()
            .copied()
            .filter(|&i| self.is_important(&self.tasks[i]) && !self.is_urgent(&self.tasks[i]))
            .collect();
        calm.sort_by_key(|&i| (self.topic_rank(&self.tasks[i]), self.tasks[i].since));

        let mut picks: Vec<usize> = Vec::new();
        for i in urgent
            .into_iter()
            .chain(leftovers.iter().copied())
            .chain(calm)
        {
            if picks.len() == TODAY_LIMIT {
                break;
            }
            if self.tasks[i].status == Status::Open && !picks.contains(&i) {
                picks.push(i);
            }
        }
        picks
    }

    fn propose(&mut self, leftovers: &[usize]) {
        let picks = self.suggestion(leftovers);
        // Die Reihenfolge der Liste ist die Reihenfolge im Vektor: Vorschläge nach vorn.
        let mut tasks = Vec::with_capacity(self.tasks.len());
        for &i in &picks {
            let mut t = self.tasks[i].clone();
            t.status = Status::Today;
            tasks.push(t);
        }
        for (i, t) in self.tasks.iter().enumerate() {
            if !picks.contains(&i) {
                tasks.push(t.clone());
            }
        }
        self.tasks = tasks;
        self.suggested = !picks.is_empty();
    }

    /// Jede offene Aufgabe bekommt eine Nummer. Was im Archiv steht, behält seine
    /// Nummer, solange es dort steht: So bleibt `fokus zurück 3` eindeutig, und wer
    /// nach einer älteren Ansicht `fokus fertig 2` tippt, erwischt keine andere.
    fn assign_numbers(&mut self) {
        let today = self.today;
        let mut used = HashSet::new();
        for t in &mut self.tasks {
            if t.is_active() && t.nr > 0 && !used.insert(t.nr) {
                t.nr = 0;
            }
        }
        for t in &mut self.tasks {
            if !t.is_active() && (!t.is_restorable(today) || t.nr == 0 || !used.insert(t.nr)) {
                t.nr = 0;
            }
        }
        let mut next = 1;
        for t in &mut self.tasks {
            if (t.is_active() || t.is_restorable(today)) && t.nr == 0 {
                while used.contains(&next) {
                    next += 1;
                }
                t.nr = next;
                used.insert(next);
            }
        }
    }

    pub fn add_task(&mut self, task: Task) -> usize {
        self.tasks.push(task);
        self.assign_numbers();
        self.tasks.len() - 1
    }

    /// Legt ein Thema an, wenn es neu ist. `Some(true)`: über der Linie gelandet.
    pub fn ensure_topic(&mut self, tag: &str) -> Option<bool> {
        let k = tags::key(tag);
        if k.is_empty() || self.all_topics().any(|n| tags::key(n) == k) {
            return None;
        }
        let name = tags::display(tag);
        if self.important.len() < IMPORTANT_LIMIT {
            self.important.push(name);
            Some(true)
        } else {
            self.other.push(name);
            Some(false)
        }
    }

    pub fn all_topics(&self) -> impl Iterator<Item = &String> {
        self.important.iter().chain(self.other.iter())
    }

    /// Eine erledigte oder gestrichene Aufgabe, die noch im Archiv steht.
    pub fn find_restorable(&self, nr: u32) -> Option<usize> {
        if nr == 0 {
            return None;
        }
        self.tasks
            .iter()
            .position(|t| !t.is_active() && t.nr == nr && t.is_restorable(self.today))
    }

    pub fn find(&self, nr: u32) -> Option<usize> {
        if nr == 0 {
            return None;
        }
        self.tasks.iter().position(|t| t.is_active() && t.nr == nr)
    }

    pub fn topic_name(&self, t: &Task) -> Option<String> {
        let tag = t.topic_tag()?;
        let k = tags::key(&tag);
        Some(
            self.all_topics()
                .find(|n| tags::key(n) == k)
                .cloned()
                .unwrap_or_else(|| tags::display(&tag)),
        )
    }

    /// Rang des Themas: 0 bis 2 über der Linie, danach die übrigen, zuletzt ohne Thema.
    pub fn topic_rank(&self, t: &Task) -> usize {
        match t.topic_tag() {
            Some(tag) => {
                let k = tags::key(&tag);
                self.all_topics()
                    .position(|n| tags::key(n) == k)
                    .unwrap_or(usize::MAX - 1)
            }
            None => usize::MAX,
        }
    }

    pub fn is_important(&self, t: &Task) -> bool {
        match t.topic_tag() {
            Some(tag) => {
                let k = tags::key(&tag);
                self.important.iter().any(|n| tags::key(n) == k)
            }
            None => false,
        }
    }

    pub fn is_urgent(&self, t: &Task) -> bool {
        t.due
            .is_some_and(|d| (d - self.today).num_days() <= URGENT_DAYS)
    }

    /// Termin verstrichen und nicht auf heute. Wiederholungen rücken selbst weiter.
    pub fn is_overdue(&self, t: &Task) -> bool {
        matches!(t.status, Status::Open | Status::Waiting)
            && t.every.is_none()
            && t.due.is_some_and(|d| d < self.today)
    }

    pub fn is_stale(&self, t: &Task) -> bool {
        let age = (self.today - t.since).num_days();
        match t.status {
            Status::Open => age >= STALE_OPEN_DAYS,
            Status::Waiting => age >= STALE_WAITING_DAYS,
            _ => false,
        }
    }

    /// Offene Aufgaben auf heute, in ihrer Reihenfolge.
    pub fn today_open(&self) -> Vec<usize> {
        (0..self.tasks.len())
            .filter(|&i| self.tasks[i].status == Status::Today)
            .collect()
    }

    pub fn done_today(&self) -> Vec<usize> {
        (0..self.tasks.len())
            .filter(|&i| {
                let t = &self.tasks[i];
                t.status == Status::Done && t.on_today && t.done_on == Some(self.today)
            })
            .collect()
    }

    /// Schiebt eine Aufgabe ans Ende, damit sie in ihrem Abschnitt zuletzt steht.
    pub fn move_to_end(&mut self, i: usize) -> usize {
        let t = self.tasks.remove(i);
        self.tasks.push(t);
        self.tasks.len() - 1
    }

    pub fn render(&self) -> String {
        let mut s = String::from(HEADER);
        s.push_str("## Themen\n");
        for (i, n) in self.important.iter().enumerate() {
            s.push_str(&format!("{}. {n}\n", i + 1));
        }
        s.push_str("---\n");
        for n in &self.other {
            s.push_str(&format!("- {n}\n"));
        }

        s.push_str(&format!(
            "\n## Heute · {}{}\n",
            date::to_file(self.today),
            if self.suggested { " · Vorschlag" } else { "" }
        ));
        for &i in &self.done_today() {
            s.push_str(&line(&self.tasks[i]));
        }
        for &i in &self.today_open() {
            s.push_str(&line(&self.tasks[i]));
        }

        s.push_str("\n## Warte auf\n");
        for t in self.tasks.iter().filter(|t| t.status == Status::Waiting) {
            s.push_str(&line(t));
        }
        s.push_str("\n## Eingang\n");
        for t in self
            .tasks
            .iter()
            .filter(|t| t.status == Status::Open && t.topic_tag().is_none())
        {
            s.push_str(&line(t));
        }
        s.push_str("\n## Offen\n");
        for t in self
            .tasks
            .iter()
            .filter(|t| t.status == Status::Open && t.topic_tag().is_some())
        {
            s.push_str(&line(t));
        }

        let mut done: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| t.status == Status::Done && !(t.on_today && t.done_on == Some(self.today)))
            .collect();
        if !done.is_empty() {
            done.sort_by_key(|t| std::cmp::Reverse(t.done_on));
            s.push_str("\n## Erledigt\n");
            for t in done {
                s.push_str(&line(t));
            }
        }
        let mut archived: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| t.status == Status::Archived)
            .collect();
        if !archived.is_empty() {
            archived.sort_by_key(|t| std::cmp::Reverse(t.archived_on));
            s.push_str("\n## Archiv\n");
            for t in archived {
                s.push_str(&line(t));
            }
        }

        let first = self.notes.iter().position(|l| !l.trim().is_empty());
        let last = self.notes.iter().rposition(|l| !l.trim().is_empty());
        if let (Some(a), Some(b)) = (first, last) {
            s.push_str("\n## Notizen\n");
            for l in &self.notes[a..=b] {
                s.push_str(l);
                s.push('\n');
            }
        }
        s
    }

    /// Schreibt die Datei, aber nur, wenn sich etwas geändert hat.
    pub fn save(&mut self) -> Result<(), String> {
        self.assign_numbers();
        let text = self.render();
        if text == self.original {
            return Ok(());
        }
        write_atomic(&self.path, &text)
            .map_err(|e| format!("Ich kann {} nicht schreiben: {e}", self.path.display()))?;
        self.original = text;
        Ok(())
    }
}

fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent()
        && !dir.as_os_str().is_empty()
    {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("md.neu");
    fs::write(&tmp, text)?;
    fs::rename(&tmp, path)
}

fn section_of(heading: &str) -> Section {
    let h = heading.trim().to_lowercase();
    let starts = |p: &str| h.starts_with(p);
    if starts("themen") {
        Section::Topics
    } else if starts("heute") {
        Section::Today
    } else if starts("warte") {
        Section::Waiting
    } else if starts("eingang") {
        Section::Inbox
    } else if starts("offen") {
        Section::Open
    } else if starts("erledigt") {
        Section::Done
    } else if starts("archiv") {
        Section::Archive
    } else if starts("notiz") {
        Section::Notes
    } else {
        Section::Unknown
    }
}

/// `1. Arbeit`, `- Haus`, `* Haus` → der Name.
fn strip_marker(s: &str) -> &str {
    let s = s.trim();
    if let Some(r) = s.strip_prefix("- ").or_else(|| s.strip_prefix("* ")) {
        return r.trim();
    }
    let digits = s.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits > 0 {
        let r = &s[digits..];
        if let Some(r) = r.strip_prefix('.').or_else(|| r.strip_prefix(')')) {
            return r.trim();
        }
    }
    s
}

enum Meta {
    Nr(u32),
    Due(NaiveDate),
    Since(NaiveDate),
    WaitingFor(String),
    DoneOn(NaiveDate),
    ArchivedOn(NaiveDate),
    Every(u32),
}

fn parse_meta(inner: &str) -> Option<Vec<Meta>> {
    let mut out = Vec::new();
    for part in inner.split('·') {
        let p = part.trim();
        let m = if let Some(r) = p.strip_prefix("Nr.").or_else(|| p.strip_prefix("Nr")) {
            Meta::Nr(r.trim().parse().ok()?)
        } else if let Some(r) = p.strip_prefix("fällig") {
            Meta::Due(date::from_file(r)?)
        } else if let Some(r) = p.strip_prefix("seit") {
            Meta::Since(date::from_file(r)?)
        } else if let Some(r) = p.strip_prefix("wartet auf") {
            let name = r.trim().trim_start_matches('@').trim();
            if name.is_empty() {
                return None;
            }
            Meta::WaitingFor(name.to_string())
        } else if let Some(r) = p.strip_prefix("erledigt") {
            Meta::DoneOn(date::from_file(r)?)
        } else if let Some(r) = p.strip_prefix("archiviert") {
            Meta::ArchivedOn(date::from_file(r)?)
        } else if p.eq_ignore_ascii_case("jede Woche") {
            Meta::Every(1)
        } else if let Some(r) = p.strip_prefix("alle")
            && let Some(n) = r.trim().strip_suffix("Wochen")
        {
            Meta::Every(n.trim().parse().ok().filter(|&n| n > 0)?)
        } else {
            return None;
        };
        out.push(m);
    }
    Some(out)
}

/// Trennt `Text  [Nr. 4 · seit 20.11.2026]` in Text und Daten. Eine Klammer, die
/// nicht nach Fokus-Daten aussieht, gehört zum Text.
fn split_meta(s: &str) -> (&str, Vec<Meta>) {
    if s.ends_with(']')
        && let Some(i) = s.rfind('[')
        && let Some(meta) = parse_meta(&s[i + 1..s.len() - 1])
    {
        return (s[..i].trim(), meta);
    }
    (s.trim(), Vec::new())
}

fn parse_task(
    line: &str,
    section: Section,
    list_date: NaiveDate,
    today: NaiveDate,
) -> Option<Task> {
    let (checked, rest) = if let Some(r) = line.strip_prefix("- [ ]") {
        (false, r)
    } else if let Some(r) = line
        .strip_prefix("- [x]")
        .or_else(|| line.strip_prefix("- [X]"))
    {
        (true, r)
    } else if let Some(r) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
        (false, r)
    } else {
        return None;
    };
    let (text, meta) = split_meta(rest.trim());
    if text.is_empty() {
        return None;
    }
    let mut t = Task::new(text, None, today);
    for m in meta {
        match m {
            Meta::Nr(n) => t.nr = n,
            Meta::Due(d) => t.due = Some(d),
            Meta::Since(d) => t.since = d,
            Meta::WaitingFor(p) => t.waiting_for = Some(p),
            Meta::DoneOn(d) => t.done_on = Some(d),
            Meta::ArchivedOn(d) => t.archived_on = Some(d),
            Meta::Every(n) => t.every = Some(n),
        }
    }
    t.status = match section {
        Section::Done => Status::Done,
        Section::Archive => Status::Archived,
        _ if checked => {
            t.on_today = section == Section::Today;
            Status::Done
        }
        Section::Today => Status::Today,
        Section::Waiting => Status::Waiting,
        _ => Status::Open,
    };
    if t.status == Status::Done && t.done_on.is_none() {
        t.done_on = Some(if section == Section::Today {
            list_date
        } else {
            today
        });
    }
    if t.status == Status::Archived && t.archived_on.is_none() {
        t.archived_on = Some(today);
    }
    if t.status != Status::Waiting {
        t.waiting_for = None;
    }
    Some(t)
}

fn line(t: &Task) -> String {
    let mut meta = Vec::new();
    if !t.is_active() && t.nr > 0 {
        // Bleibt reserviert, solange es im Archiv steht, siehe `assign_numbers`.
        meta.push(format!("Nr. {}", t.nr));
    }
    match t.status {
        Status::Done => {
            if let Some(d) = t.due {
                meta.push(format!("fällig {}", date::to_file(d)));
            }
            if let Some(d) = t.done_on {
                meta.push(format!("erledigt {}", date::to_file(d)));
            }
        }
        Status::Archived => {
            if let Some(d) = t.due {
                meta.push(format!("fällig {}", date::to_file(d)));
            }
            if let Some(d) = t.archived_on {
                meta.push(format!("archiviert {}", date::to_file(d)));
            }
        }
        _ => {
            meta.push(format!("Nr. {}", t.nr));
            if let Some(d) = t.due {
                meta.push(format!("fällig {}", date::to_file(d)));
            }
            if let Some(p) = &t.waiting_for {
                meta.push(format!("wartet auf @{p}"));
            }
            meta.push(format!("seit {}", date::to_file(t.since)));
        }
    }
    match t.every {
        Some(1) => meta.push("jede Woche".into()),
        Some(n) => meta.push(format!("alle {n} Wochen")),
        None => {}
    }
    let check = if t.status == Status::Done { "x" } else { " " };
    format!("- [{check}] {}  [{}]\n", t.text, meta.join(" · "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn heute() -> NaiveDate {
        tag(2026, 11, 26)
    }

    fn lies(text: &str) -> Store {
        Store::parse(text, PathBuf::from("test.md"), heute())
    }

    const BEISPIEL: &str = "\
## Themen
1. Arbeit
2. Projekt
3. Gesundheit
---
- Haus

## Heute · 26.11.2026
- [x] Angebot schicken #arbeit  [erledigt 26.11.2026]
- [ ] Präsentation vorbereiten #arbeit  [Nr. 2 · fällig 27.11.2026 · seit 20.11.2026]

## Warte auf
- [ ] Fotos schicken @anna  [Nr. 5 · wartet auf @anna · seit 21.11.2026]

## Eingang
- [ ] Mama zurückrufen  [Nr. 7 · seit 25.11.2026]

## Offen
- [ ] Keller ausmisten #haus  [Nr. 3 · seit 10.11.2026]
";

    #[test]
    fn eine_eigene_datei_bleibt_beim_lesen_und_schreiben_gleich() {
        let s = lies(BEISPIEL);
        let wieder = lies(&s.render());
        assert_eq!(s.render(), wieder.render());
        assert_eq!(s.tasks.len(), 5);
        assert!(!s.suggested);
        assert!(
            s.render()
                .contains("[Nr. 2 · fällig 27.11.2026 · seit 20.11.2026]")
        );
    }

    #[test]
    fn abschnitt_bestimmt_den_zustand() {
        let s = lies(BEISPIEL);
        let status = |nr| s.tasks[s.find(nr).unwrap()].status;
        assert_eq!(status(2), Status::Today);
        assert_eq!(status(5), Status::Waiting);
        assert_eq!(status(7), Status::Open);
        assert_eq!(s.done_today().len(), 1);
    }

    #[test]
    fn wichtig_ist_nur_was_ueber_der_linie_steht() {
        let s = lies(BEISPIEL);
        let t = |nr| &s.tasks[s.find(nr).unwrap()];
        assert!(s.is_important(t(2)));
        assert!(!s.is_important(t(3)));
        assert!(!s.is_important(t(7)));
        assert!(s.is_urgent(t(2)));
    }

    #[test]
    fn mehr_als_drei_themen_ueber_der_linie_rutschen_nach_unten() {
        let s = lies("## Themen\n1. A\n2. B\n3. C\n4. D\n---\n- E\n");
        assert_eq!(s.important, vec!["A", "B", "C"]);
        // D und E haben keine Aufgaben und fallen deshalb ganz weg.
        assert!(s.other.is_empty());
    }

    #[test]
    fn von_hand_geschriebene_zeilen_bekommen_nummer_und_datum() {
        let s = lies("## Offen\n- Steuererklärung #finanzen\n- [ ] Fahrrad reparieren\n");
        assert_eq!(s.tasks.iter().filter(|t| t.nr > 0).count(), 2);
        assert_eq!(s.other, vec!["Finanzen"]);
        assert!(
            s.render()
                .contains("- [ ] Fahrrad reparieren  [Nr. 2 · seit 26.11.2026]")
        );
    }

    #[test]
    fn doppelte_nummern_werden_neu_vergeben() {
        let s = lies(
            "## Offen\n- [ ] A  [Nr. 1 · seit 26.11.2026]\n- [ ] B  [Nr. 1 · seit 26.11.2026]\n",
        );
        let mut nrs: Vec<u32> = s.tasks.iter().map(|t| t.nr).collect();
        nrs.sort();
        assert_eq!(nrs, vec![1, 2]);
    }

    #[test]
    fn eckige_klammern_im_text_bleiben_text() {
        let s =
            lies("## Offen\n- [ ] Notiz [wichtig]  [Nr. 1 · seit 26.11.2026]\n- [ ] Liste [a]\n");
        assert_eq!(s.tasks[0].text, "Notiz [wichtig]");
        assert_eq!(s.tasks[1].text, "Liste [a]");
    }

    #[test]
    fn neuer_tag_schlaegt_dringend_und_wichtig_zuerst_vor() {
        let gestern = BEISPIEL.replace("## Heute · 26.11.2026", "## Heute · 25.11.2026");
        let mut text = gestern.replace(
            "## Offen\n",
            "## Offen\n- [ ] Bericht #arbeit  [Nr. 9 · fällig 26.11.2026 · seit 01.11.2026]\n- [ ] Laufen #gesundheit  [Nr. 10 · seit 01.11.2026]\n",
        );
        text.push_str(
            "- [ ] Unwichtig dringend #haus  [Nr. 11 · fällig 26.11.2026 · seit 25.11.2026]\n",
        );
        let s = lies(&text);
        assert!(s.suggested);
        let heute: Vec<&str> = s
            .today_open()
            .iter()
            .map(|&i| s.tasks[i].text.as_str())
            .collect();
        assert_eq!(
            heute,
            vec![
                "Bericht #arbeit",
                "Präsentation vorbereiten #arbeit",
                "Laufen #gesundheit"
            ]
        );
        // Das Erledigte von gestern zählt heute nicht mehr mit.
        assert!(s.done_today().is_empty());
        // Erledigtes behält seine Nummer, solange es im Archiv steht.
        assert!(s.render().contains(
            "## Erledigt\n- [x] Angebot schicken #arbeit  [Nr. 1 · erledigt 26.11.2026]"
        ));
    }

    #[test]
    fn unwichtiges_ohne_termin_wandert_nach_30_tagen_ins_archiv() {
        let s = lies(
            "## Offen\n\
             - [ ] Alt  [Nr. 1 · seit 20.10.2026]\n\
             - [ ] Alt mit Termin  [Nr. 2 · fällig 20.12.2026 · seit 20.10.2026]\n\
             - [ ] Neu  [Nr. 3 · seit 20.11.2026]\n",
        );
        let archiv: Vec<&str> = s
            .tasks
            .iter()
            .filter(|t| t.status == Status::Archived)
            .map(|t| t.text.as_str())
            .collect();
        assert_eq!(archiv, vec!["Alt"]);
        assert_eq!(s.messages.len(), 1);
    }

    #[test]
    fn unbekannte_zeilen_gehen_nicht_verloren() {
        let s = lies("## Offen\nirgendein Gedanke\n\n## Notizen\nIdee für später\n");
        let out = s.render();
        assert!(out.contains("## Notizen\nirgendein Gedanke\nIdee für später\n"));
    }

    #[test]
    fn nach_14_tagen_ist_das_archiv_weg() {
        let s = lies(
            "## Erledigt\n\
             - [x] Frisch  [Nr. 4 · erledigt 13.11.2026]\n\
             - [x] Alt  [Nr. 5 · erledigt 11.11.2026]\n\
             ## Archiv\n\
             - [ ] Gestrichen  [archiviert 20.11.2026]\n",
        );
        let texte: Vec<&str> = s.tasks.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texte, vec!["Frisch", "Gestrichen"]);
        // Was im Archiv steht, hat eine Nummer zum Zurückholen.
        assert!(s.find_restorable(4).is_some());
        assert!(s.tasks[1].nr > 0);
        assert!(s.find(4).is_none());
    }

    #[test]
    fn verpasste_wiederholung_rueckt_weiter() {
        // 26.11.2026 ist ein Donnerstag; fällig war Montag, der 9.11.
        let s = lies(
            "## Offen\n- [ ] Wochenplan  [Nr. 1 · fällig 09.11.2026 · seit 01.11.2026 · jede Woche]\n",
        );
        assert_eq!(s.tasks[0].due, Some(tag(2026, 11, 23)));
        let s = lies(
            "## Offen\n- [ ] Gießen  [Nr. 1 · fällig 09.11.2026 · seit 01.11.2026 · alle 2 Wochen]\n",
        );
        assert_eq!(s.tasks[0].due, Some(tag(2026, 11, 23)));
        assert!(s.render().contains("alle 2 Wochen]"));
        let s = lies(
            "## Offen\n- [ ] Miete  [Nr. 1 · fällig 16.11.2026 · seit 01.11.2026 · alle 4 Wochen]\n",
        );
        assert_eq!(s.tasks[0].due, Some(tag(2026, 11, 16)));
        assert!(!s.is_overdue(&s.tasks[0]));
    }

    #[test]
    fn wartende_ohne_person_sind_wieder_offen() {
        let s =
            lies("## Warte auf\n- [ ] Rückmeldung  [Nr. 1 · seit 26.11.2026]\n- [ ] Fotos @anna\n");
        assert_eq!(s.tasks[0].status, Status::Open);
        assert_eq!(s.tasks[1].status, Status::Waiting);
        assert_eq!(s.tasks[1].waiting_for.as_deref(), Some("anna"));
    }
}
