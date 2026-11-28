//! Die Ansichten. Jede liefert einen fertigen Text; gedruckt wird in `main`.

use std::collections::HashMap;
use std::path::Path;

use chrono::NaiveDate;

use crate::store::{self, IMPORTANT_LIMIT, KEEP_DAYS, Status, Store, TODAY_LIMIT, Task};
use crate::{date, notes, tags, term};

#[derive(Clone, Copy)]
enum Tone {
    Plain,
    Dim,
    Warn,
    Alarm,
}

fn tone(s: &str, t: Tone) -> String {
    match t {
        Tone::Plain => s.to_string(),
        Tone::Dim => term::dim(s),
        Tone::Warn => term::yellow(s),
        Tone::Alarm => term::red(s),
    }
}

struct Row {
    nr: u32,
    mark: &'static str,
    text: String,
    /// ↗ oder ✎ hinter dem Text, wenn es Notizen gibt. Bleibt beim Kürzen stehen.
    marker: &'static str,
    mid: (String, Tone),
    right: (String, Tone),
    faded: bool,
}

struct Block {
    title: String,
    hint: String,
    rows: Vec<Row>,
    note: Option<String>,
}

fn due_cell(store: &Store, task: &Task) -> (String, Tone) {
    match task.due {
        None => (String::new(), Tone::Plain),
        Some(d) => {
            let n = (d - store.today).num_days();
            let t = if n < 0 {
                Tone::Alarm
            } else if n <= 2 {
                Tone::Warn
            } else {
                Tone::Dim
            };
            let label = date::due_label(d, store.today);
            let label = if task.every.is_some() {
                format!("↻ {label}")
            } else {
                label
            };
            (label, t)
        }
    }
}

fn topic_cell(store: &Store, t: &Task) -> (String, Tone) {
    (store.topic_label(t).unwrap_or_default(), Tone::Dim)
}

fn row(store: &Store, t: &Task, mark: &'static str) -> Row {
    Row {
        nr: t.nr,
        mark,
        text: t.display(),
        marker: t.marker(),
        mid: due_cell(store, t),
        right: topic_cell(store, t),
        faded: false,
    }
}

/// Genau `w` Spalten; ein Zeichen für Notizen bleibt hinten stehen, auch wenn
/// der Text gekürzt werden muss.
fn fit_marked(text: &str, marker: &str, w: usize) -> String {
    if marker.is_empty() || w < 4 {
        return term::pad_right(&term::truncate_words(text, w), w);
    }
    let body = term::truncate_words(text, w - 2);
    term::pad_right(&format!("{body} {marker}"), w)
}

/// Alle Blöcke einer Ansicht mit gemeinsamen Spalten.
fn layout(blocks: &[Block], width: usize) -> String {
    let rows = blocks.iter().flat_map(|b| b.rows.iter());
    let mid_w = rows
        .clone()
        .map(|r| term::text_width(&r.mid.0))
        .max()
        .unwrap_or(0);
    let right_w = rows
        .map(|r| term::text_width(&r.right.0))
        .max()
        .unwrap_or(0)
        .min(18);
    let fixed =
        7 + if mid_w > 0 { mid_w + 2 } else { 0 } + if right_w > 0 { right_w + 2 } else { 0 };
    let text_w = width.saturating_sub(fixed).max(12);

    let mut out = String::new();
    for b in blocks {
        if b.rows.is_empty() && b.note.is_none() {
            continue;
        }
        out.push(' ');
        out.push_str(&term::bold(&b.title));
        if !b.hint.is_empty() {
            out.push_str("   ");
            out.push_str(&term::dim(&b.hint));
        }
        out.push('\n');
        for r in &b.rows {
            let nr = if r.nr > 0 {
                format!("{:>3}", r.nr)
            } else {
                "   ".into()
            };
            let mut plain_line = String::new();
            let mut line = String::new();
            let text = fit_marked(&r.text, r.marker, text_w);
            plain_line.push_str(&format!(" {nr} {} {text}", r.mark));
            line.push_str(&format!(
                " {} {} {text}",
                term::dim(&nr),
                mark_style(r.mark)
            ));
            if mid_w > 0 {
                let m = term::pad_left(&r.mid.0, mid_w);
                plain_line.push_str(&format!("  {m}"));
                line.push_str(&format!("  {}", tone(&m, r.mid.1)));
            }
            if right_w > 0 {
                let m = term::fit(&r.right.0, right_w);
                plain_line.push_str(&format!("  {m}"));
                line.push_str(&format!("  {}", tone(&m, r.right.1)));
            }
            if r.faded {
                out.push_str(&term::dim(plain_line.trim_end()));
            } else if term::color() {
                out.push_str(&line);
            } else {
                out.push_str(plain_line.trim_end());
            }
            out.push('\n');
        }
        if let Some(n) = &b.note {
            out.push_str(&format!("       {}\n", term::dim(n)));
        }
        out.push('\n');
    }
    out
}

fn mark_style(mark: &str) -> String {
    match mark {
        "▶" => term::cyan(mark),
        "✓" => term::green(mark),
        _ => term::dim(mark),
    }
}

fn header(left: &str, right: &str, width: usize) -> String {
    let gap = width
        .saturating_sub(2 + term::text_width(left) + term::text_width(right))
        .max(2);
    let mut s = format!(" {}", term::bold(left));
    if !right.is_empty() {
        s.push_str(&" ".repeat(gap));
        s.push_str(&term::dim(right));
    }
    s.push('\n');
    s.push(' ');
    s.push_str(&term::dim(&"─".repeat(width.saturating_sub(2))));
    s.push('\n');
    s
}

/// `focus` – was heute dran ist.
pub fn today(store: &Store) -> String {
    let width = term::list_width();
    let done = store.done_today();
    let open = store.today_open();
    let total = done.len() + open.len();
    let right = if total > 0 {
        format!("{} von {} erledigt", done.len(), total)
    } else {
        String::new()
    };
    let mut out = header(&date::long(store.today), &right, width);

    if !store.tasks.iter().any(Task::is_active) && done.is_empty() {
        out.push_str(&welcome());
        return out;
    }
    out.push('\n');

    let mut rows: Vec<Row> = Vec::new();
    for &i in &done {
        let mut r = row(store, &store.tasks[i], "✓");
        r.nr = 0;
        r.mid = (String::new(), Tone::Plain);
        r.faded = true;
        rows.push(r);
    }
    for (n, &i) in open.iter().enumerate() {
        rows.push(row(store, &store.tasks[i], if n == 0 { "▶" } else { "·" }));
    }
    let any_open = store.tasks.iter().any(|t| t.status == Status::Open);
    let suggestion: Vec<String> = store
        .suggestion(&[])
        .iter()
        .map(|&i| store.tasks[i].nr.to_string())
        .collect();
    let note = if open.is_empty() && !done.is_empty() {
        Some("Alles für heute geschafft.".to_string())
    } else if open.is_empty() && !suggestion.is_empty() {
        Some(format!(
            "Noch nichts für heute. Mein Vorschlag: focus heute {}",
            suggestion.join(" ")
        ))
    } else if open.is_empty() && any_open {
        Some(format!(
            "Noch nichts für heute. Wähle bis zu {TODAY_LIMIT}: focus heute <Nr>  (Nummern: focus alles)"
        ))
    } else if open.is_empty() {
        Some("Noch nichts für heute.".to_string())
    } else {
        None
    };
    let hint = if store.suggested && !open.is_empty() {
        "Vorschlag von Focus · tauschen: focus heute <Nr>".to_string()
    } else {
        String::new()
    };

    let urgent = |important: bool| -> Vec<&Task> {
        let mut v: Vec<&Task> = store
            .tasks
            .iter()
            .filter(|t| {
                t.status == Status::Open && store.is_urgent(t) && store.is_important(t) == important
            })
            .collect();
        v.sort_by_key(|t| t.due);
        v
    };
    let must = urgent(true);
    let quick = urgent(false);

    let blocks = vec![
        Block {
            title: "HEUTE".into(),
            hint,
            rows,
            note,
        },
        Block {
            title: "DRINGEND UND WICHTIG".into(),
            hint: "→ gehört eigentlich auf heute".into(),
            rows: must.iter().map(|t| row(store, t, "·")).collect(),
            note: None,
        },
        Block {
            title: "DRINGEND, ABER NICHT WICHTIG".into(),
            hint: "→ schnell erledigen oder abgeben".into(),
            rows: quick.iter().map(|t| row(store, t, "·")).collect(),
            note: None,
        },
    ];
    out.push_str(&layout(&blocks, width));

    let rest = store
        .tasks
        .iter()
        .filter(|t| t.status == Status::Open)
        .count()
        - must.len()
        - quick.len();
    let waiting = store
        .tasks
        .iter()
        .filter(|t| t.status == Status::Waiting)
        .count();
    let overdue = store.tasks.iter().filter(|t| store.is_overdue(t)).count();
    let stale = store
        .tasks
        .iter()
        .filter(|t| !store.is_overdue(t) && store.is_stale(t))
        .count();
    let mut parts = Vec::new();
    if rest > 0 {
        parts.push(format!("{rest} weitere offen"));
    }
    if waiting > 0 {
        parts.push(format!(
            "{waiting} {} auf andere",
            if waiting == 1 { "wartet" } else { "warten" }
        ));
    }
    if overdue > 0 {
        parts.push(format!("{overdue} überfällig"));
    }
    if stale > 0 {
        parts.push(format!(
            "{stale} {} schon lange",
            if stale == 1 { "liegt" } else { "liegen" }
        ));
    }
    if !parts.is_empty() {
        let next = if stale + overdue > 0 {
            "focus aufräumen"
        } else {
            "focus alles"
        };
        if let Some(last) = parts.last_mut() {
            last.push_str(&format!(" → {next}"));
        }
        out.push_str(&hint_lines(&parts, width));
    }
    out
}

fn welcome() -> String {
    format!(
        "\n Noch keine Aufgaben. Schreib die erste auf:\n\n   {}\n\n {}\n {}\n",
        term::bold("focus + Präsentation vorbereiten #arbeit morgen"),
        term::dim("#arbeit ist das Thema, morgen der Termin. Beides darfst du weglassen."),
        term::dim("Alle Befehle: focus hilfe"),
    )
}

/// `focus alles` – die vier Felder.
pub fn matrix(store: &Store) -> String {
    let width = term::width();
    let mut q: [Vec<&Task>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    for t in store
        .tasks
        .iter()
        .filter(|t| matches!(t.status, Status::Today | Status::Open))
    {
        let i = match (store.is_important(t), store.is_urgent(t)) {
            (true, true) => 0,
            (true, false) => 1,
            (false, true) => 2,
            (false, false) => 3,
        };
        q[i].push(t);
    }
    q[0].sort_by_key(|t| t.due);
    q[1].sort_by_key(|t| (store.topic_rank(t), t.due.is_none(), t.due, t.since));
    q[2].sort_by_key(|t| t.due);
    q[3].sort_by_key(|t| (t.due.is_none(), t.due, t.since));

    let waiting = store
        .tasks
        .iter()
        .filter(|t| t.status == Status::Waiting)
        .count();
    if q.iter().all(Vec::is_empty) {
        let mut s = String::from("\n Keine offenen Aufgaben.\n");
        if waiting > 0 {
            s.push_str(&format!(
                " {}\n",
                term::dim(&format!(
                    "{waiting} {} bei anderen: focus personen",
                    if waiting == 1 { "liegt" } else { "liegen" }
                ))
            ));
        }
        return s;
    }

    let right = |t: &Task, quadrant: usize| -> String {
        let age = (store.today - t.since).num_days();
        match (quadrant, t.due) {
            (3, _) if age >= 14 => format!("{age} Tage"),
            (_, Some(d)) => date::due_short(d, store.today),
            _ => String::new(),
        }
    };

    let mut out = String::from("\n");
    if width >= 72 {
        out.push_str(&matrix_wide(store, &q, width, &right));
    } else {
        let titles = [
            ("WICHTIG UND DRINGEND", "tun"),
            ("WICHTIG, NICHT DRINGEND", "planen"),
            ("DRINGEND, NICHT WICHTIG", "schnell oder abgeben"),
            ("WEDER NOCH", "streichen?"),
        ];
        let blocks: Vec<Block> = (0..4)
            .map(|i| Block {
                title: titles[i].0.into(),
                hint: format!("→ {}", titles[i].1),
                rows: q[i]
                    .iter()
                    .map(|t| Row {
                        nr: t.nr,
                        mark: if t.status == Status::Today {
                            "▶"
                        } else {
                            " "
                        },
                        text: t.display(),
                        marker: t.marker(),
                        mid: (right(t, i), Tone::Dim),
                        right: topic_cell(store, t),
                        faded: false,
                    })
                    .collect(),
                note: None,
            })
            .collect();
        out.push_str(&layout(&blocks, width));
    }

    let mut legend = vec!["▶ steht auf heute".to_string()];
    if store
        .tasks
        .iter()
        .any(|t| t.is_active() && t.topic_tag().is_none())
    {
        legend.push("ohne Thema zählt als nicht wichtig".into());
    }
    if waiting > 0 {
        legend.push(format!(
            "{waiting} {} bei anderen: focus personen",
            if waiting == 1 { "liegt" } else { "liegen" }
        ));
    }
    out.push_str(&hint_lines(&legend, width));
    out
}

/// Hinweise mit „ · “ dazwischen, umbrochen, damit keine Zeile zu breit wird.
fn hint_lines(parts: &[String], width: usize) -> String {
    let mut lines: Vec<String> = Vec::new();
    for p in parts {
        match lines.last_mut() {
            Some(l) if term::text_width(l) + 3 + term::text_width(p) < width.saturating_sub(1) => {
                l.push_str(" · ");
                l.push_str(p);
            }
            _ => lines.push(p.clone()),
        }
    }
    lines
        .iter()
        .map(|l| format!(" {}\n", term::dim(l)))
        .collect()
}

const MATRIX_ROWS: usize = 7;

fn matrix_wide(
    store: &Store,
    q: &[Vec<&Task>; 4],
    width: usize,
    right: &dyn Fn(&Task, usize) -> String,
) -> String {
    let label_w = 11;
    let cw = ((width.saturating_sub(label_w + 4)) / 2).min(40);
    let line = |l: &str, m: &str, r: &str| -> String {
        format!(
            "{}{}{}{}{}{}\n",
            " ".repeat(label_w),
            term::dim(l),
            term::dim(&"─".repeat(cw)),
            term::dim(m),
            term::dim(&"─".repeat(cw)),
            term::dim(r),
        )
    };

    // Eine Zelle als Zeilen mit genau `cw` Spalten.
    let cell = |quadrant: usize| -> Vec<String> {
        let tasks = &q[quadrant];
        let shown = if tasks.len() > MATRIX_ROWS {
            MATRIX_ROWS - 1
        } else {
            tasks.len()
        };
        let rights: Vec<String> = tasks[..shown].iter().map(|t| right(t, quadrant)).collect();
        let rw = rights
            .iter()
            .map(|r| term::text_width(r))
            .max()
            .unwrap_or(0)
            .min(11);
        let mut lines = Vec::new();
        for (t, r) in tasks[..shown].iter().zip(&rights) {
            let nr = format!("{:>2}", t.nr);
            let mark = if t.status == Status::Today {
                "▶"
            } else {
                " "
            };
            // " NN ▶ " vorn (6), ein Leerzeichen hinten, dazwischen Text und Termin.
            let left_w = cw.saturating_sub(7 + if rw > 0 { rw + 1 } else { 0 });
            let text = fit_marked(&t.display(), t.marker(), left_w);
            let r = term::pad_left(r, rw);
            let tone_r = match t.due {
                Some(d) if (d - store.today).num_days() < 0 => Tone::Alarm,
                Some(d) if (d - store.today).num_days() <= 2 => Tone::Warn,
                _ => Tone::Dim,
            };
            let mut s = format!(" {} {} {text}", term::dim(&nr), mark_style(mark));
            if rw > 0 {
                s.push(' ');
                s.push_str(&tone(&r, tone_r));
            }
            s.push(' ');
            lines.push(s);
        }
        if tasks.len() > shown {
            let more = format!("… und {} weitere", tasks.len() - shown);
            lines.push(format!(" {} ", term::dim(&term::fit(&more, cw - 2))));
        }
        if lines.is_empty() {
            lines.push(format!(" {} ", term::dim(&term::fit("–", cw - 2))));
        }
        lines
    };

    let labels = ["TUN", "PLANEN", "SCHNELL ODER ABGEBEN", "STREICHEN?"];
    let mut out = String::new();
    out.push_str(&format!(
        "{} {} {}\n",
        " ".repeat(label_w),
        term::bold(&term::center("DRINGEND", cw)),
        term::bold(&term::center("NICHT DRINGEND", cw)),
    ));
    out.push_str(&line("┌", "┬", "┐"));
    for (pair, label) in [((0, 1), "WICHTIG"), ((2, 3), "UNWICHTIG")] {
        let a = cell(pair.0);
        let b = cell(pair.1);
        let n = a.len().max(b.len());
        let blank = " ".repeat(cw);
        for i in 0..n {
            let l = if i == 0 {
                term::bold(&format!("{label:>10} "))
            } else {
                " ".repeat(label_w)
            };
            out.push_str(&format!(
                "{l}{}{}{}{}{}\n",
                term::dim("│"),
                a.get(i).unwrap_or(&blank),
                term::dim("│"),
                b.get(i).unwrap_or(&blank),
                term::dim("│"),
            ));
        }
        out.push_str(&format!(
            "{}{}{}{}{}{}\n",
            " ".repeat(label_w),
            term::dim("│"),
            term::dim(&term::pad_left(&format!("{} ", labels[pair.0]), cw)),
            term::dim("│"),
            term::dim(&term::pad_left(&format!("{} ", labels[pair.1]), cw)),
            term::dim("│"),
        ));
        if pair.0 == 0 {
            out.push_str(&line("├", "┼", "┤"));
        }
    }
    out.push_str(&line("└", "┴", "┘"));
    if term::color() {
        out
    } else {
        out.lines()
            .map(str::trim_end)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }
}

/// `focus themen`
pub fn topics(store: &Store) -> String {
    let inbox = store
        .tasks
        .iter()
        .filter(|t| t.is_active() && t.topic_tag().is_none())
        .count();
    if store.all_topics().next().is_none() {
        let mut s = String::from(
            "\n Noch keine Themen. Ein Thema entsteht, sobald du es in eine Aufgabe schreibst:\n\n   ",
        );
        s.push_str(&term::bold("focus + Laufschuhe kaufen #gesundheit"));
        s.push('\n');
        if inbox > 0 {
            s.push_str(&format!(
                "\n {}\n",
                term::dim(&format!("{inbox} Aufgaben sind noch ohne Thema."))
            ));
        }
        return s;
    }

    let today = store.today;
    let name_w = store
        .all_topics()
        .map(|n| term::text_width(&tags::topic_label(n)))
        .chain([term::text_width("#ohne")])
        .max()
        .unwrap_or(10)
        .clamp(10, 24);

    struct Stats {
        open: usize,
        days: [bool; 7],
        last_done: Option<NaiveDate>,
        oldest: Option<NaiveDate>,
    }
    let stats = |name: &str| -> Stats {
        let k = tags::key(name);
        let mut s = Stats {
            open: 0,
            days: [false; 7],
            last_done: None,
            oldest: None,
        };
        for t in &store.tasks {
            if t.topic_tag().map(|g| tags::key(&g)) != Some(k.clone()) {
                continue;
            }
            if t.is_active() {
                s.open += 1;
                s.oldest = Some(s.oldest.map_or(t.since, |o| o.min(t.since)));
            }
            if t.status == Status::Done
                && let Some(d) = t.done_on
            {
                s.last_done = Some(s.last_done.map_or(d, |l| l.max(d)));
                let ago = (today - d).num_days();
                if (0..7).contains(&ago) {
                    s.days[6 - ago as usize] = true;
                }
            }
        }
        s
    };
    let dots = |days: &[bool; 7]| -> String {
        days.iter()
            .map(|&d| {
                if d {
                    term::green("●")
                } else {
                    term::dim("○")
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    };

    let mut out = String::from("\n");
    out.push_str(&format!(
        " {}  {}    {}\n",
        term::bold(&term::pad_right("DEINE THEMEN", name_w + 3)),
        term::dim("offen"),
        term::dim("letzte 7 Tage"),
    ));
    for (i, name) in store.important.iter().enumerate() {
        let s = stats(name);
        let warn = match s.last_done {
            Some(d) if (today - d).num_days() >= 7 => {
                format!("seit {} Tagen nichts erledigt", (today - d).num_days())
            }
            None if s.open > 0 && s.oldest.is_some_and(|o| (today - o).num_days() >= 7) => {
                "noch nichts erledigt".into()
            }
            _ => String::new(),
        };
        out.push_str(&format!(
            " {}  {}  {:>5}    {}   {}\n",
            term::bold(&(i + 1).to_string()),
            term::bold(&term::fit(&tags::topic_label(name), name_w)),
            s.open,
            dots(&s.days),
            term::yellow(&warn),
        ));
    }
    let free = IMPORTANT_LIMIT - store.important.len();
    if free > 0 {
        out.push_str(&format!(
            "    {}\n",
            term::dim(&format!(
                "Über der Linie ist noch Platz für {}.",
                if free == 1 {
                    "ein Thema".to_string()
                } else {
                    format!("{free} Themen")
                }
            ))
        ));
    }
    let rule_w = (name_w + 33).min(term::width().saturating_sub(2));
    out.push_str(&format!(
        " {}\n",
        term::dim(&format!(
            "─── wichtig {}",
            "─".repeat(rule_w.saturating_sub(12))
        ))
    ));
    for name in &store.other {
        let s = stats(name);
        out.push_str(&format!(
            "    {}  {:>5}    {}\n",
            term::fit(&tags::topic_label(name), name_w),
            s.open,
            dots(&s.days)
        ));
    }
    if inbox > 0 {
        out.push_str(&format!(
            "    {}  {:>5}    {}\n",
            term::dim(&term::fit("#ohne", name_w)),
            inbox,
            term::dim("ohne Thema")
        ));
    }
    out.push_str(&format!(
        "\n {}\n",
        term::dim("Welche drei wichtig sind, wählst du mit: focus themen ordnen")
    ));
    if term::color() {
        out
    } else {
        out.lines()
            .map(str::trim_end)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }
}

struct PersonCount {
    name: String,
    with: usize,
    waiting: usize,
}

fn person_counts(store: &Store) -> Vec<PersonCount> {
    let mut map: HashMap<String, PersonCount> = HashMap::new();
    let mut entry = |raw: &str| -> String {
        let k = tags::key(raw);
        map.entry(k.clone()).or_insert_with(|| PersonCount {
            name: tags::person_label(raw),
            with: 0,
            waiting: 0,
        });
        k
    };
    let mut withs = Vec::new();
    let mut waits = Vec::new();
    for t in store.tasks.iter().filter(|t| t.is_active()) {
        if t.status == Status::Waiting {
            if let Some(p) = &t.waiting_for {
                waits.push(entry(p));
            }
        } else {
            for p in t.persons() {
                withs.push(entry(&p));
            }
        }
    }
    for k in withs {
        map.get_mut(&k).unwrap().with += 1;
    }
    for k in waits {
        map.get_mut(&k).unwrap().waiting += 1;
    }
    let mut v: Vec<PersonCount> = map.into_values().collect();
    v.sort_by(|a, b| {
        (b.with + b.waiting)
            .cmp(&(a.with + a.waiting))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    v
}

/// `focus personen`
pub fn persons(store: &Store) -> String {
    let list = person_counts(store);
    if list.is_empty() {
        return format!(
            "\n Bei niemandem ist gerade etwas offen.\n {}\n",
            term::dim("Schreib @name in eine Aufgabe, etwa: focus + Urlaub abstimmen @anna")
        );
    }
    let name_w = list
        .iter()
        .map(|p| term::text_width(&p.name))
        .max()
        .unwrap_or(8)
        .clamp(12, 24);
    let num = |n: usize| {
        if n == 0 {
            "–".to_string()
        } else {
            n.to_string()
        }
    };
    let mut out = String::from("\n");
    out.push_str(&format!(
        " {}  {}    {}\n",
        term::bold(&term::pad_right("PERSONEN", name_w)),
        term::dim("mit dir"),
        term::dim("warte auf"),
    ));
    for p in &list {
        out.push_str(&format!(
            " {}  {}    {}\n",
            term::fit(&p.name, name_w),
            term::pad_left(&num(p.with), 7),
            term::pad_left(&num(p.waiting), 9),
        ));
    }
    out.push_str(&format!(
        "\n {}\n",
        term::dim("Alles zu einer Person: focus @name")
    ));
    out
}

/// `focus @anna`
pub fn person(store: &Store, raw: &str) -> String {
    let k = tags::key(tags::clean(raw));
    let name = store
        .tasks
        .iter()
        .filter(|t| t.is_active())
        .flat_map(|t| t.persons().into_iter().chain(t.waiting_for.clone()))
        .find(|p| tags::key(p) == k)
        .map(|p| tags::person_label(&p))
        .unwrap_or_else(|| tags::person_label(raw));
    let with: Vec<&Task> = store
        .tasks
        .iter()
        .filter(|t| matches!(t.status, Status::Today | Status::Open) && t.mentions(&k))
        .collect();
    let waiting: Vec<&Task> = store
        .tasks
        .iter()
        .filter(|t| {
            t.status == Status::Waiting
                && t.waiting_for.as_deref().map(tags::key) == Some(k.clone())
        })
        .collect();
    if with.is_empty() && waiting.is_empty() {
        return format!(
            "\n Bei {name} ist gerade nichts offen.\n {}\n",
            term::dim("Alle Personen: focus personen")
        );
    }
    let blocks = vec![
        Block {
            title: name.clone(),
            hint: String::new(),
            rows: with
                .iter()
                .map(|t| {
                    row(
                        store,
                        t,
                        if t.status == Status::Today {
                            "▶"
                        } else {
                            "·"
                        },
                    )
                })
                .collect(),
            note: None,
        },
        Block {
            title: format!("WARTE AUF {name}"),
            hint: String::new(),
            rows: waiting
                .iter()
                .map(|t| {
                    let mut r = row(store, t, "·");
                    r.right = (date::age_label(t.since, store.today), Tone::Dim);
                    r
                })
                .collect(),
            note: None,
        },
    ];
    let mut out = String::from("\n");
    out.push_str(&layout(&blocks, term::list_width()));
    out
}

/// `focus #arbeit` – alles in einem Thema.
pub fn topic(store: &Store, raw: &str) -> String {
    let k = tags::key(tags::clean(raw));
    if k.is_empty() {
        return topics(store);
    }
    let none = k == tags::NO_TOPIC;
    let in_topic = |t: &Task| match t.topic_tag() {
        Some(g) => !none && tags::key(&g) == k,
        None => none,
    };
    let name = if none {
        "ohne Thema".to_string()
    } else {
        store
            .all_topics()
            .find(|n| tags::key(n) == k)
            .cloned()
            .unwrap_or_else(|| tags::display(raw))
    };
    let important = store.important.iter().any(|n| tags::key(n) == k);

    let mut open: Vec<&Task> = store
        .tasks
        .iter()
        .filter(|t| matches!(t.status, Status::Today | Status::Open) && in_topic(t))
        .collect();
    open.sort_by_key(|t| (t.status != Status::Today, t.due.is_none(), t.due, t.since));
    let mut waiting: Vec<&Task> = store
        .tasks
        .iter()
        .filter(|t| t.status == Status::Waiting && in_topic(t))
        .collect();
    waiting.sort_by_key(|t| t.since);
    let done: Vec<String> = store
        .tasks
        .iter()
        .filter(|t| {
            t.status == Status::Done
                && in_topic(t)
                && t.done_on.is_some_and(|d| (store.today - d).num_days() < 7)
        })
        .map(Task::display)
        .collect();

    if none && open.is_empty() && waiting.is_empty() {
        return format!(
            "\n Alle offenen Aufgaben haben ein Thema.\n {}\n",
            term::dim("Deine Themen: focus themen")
        );
    }
    let target = notes::Target::Topic(name.clone(), None);
    let topic_notes = if none {
        String::new()
    } else {
        notes::lines(store, &target)
    };
    if open.is_empty() && waiting.is_empty() && done.is_empty() {
        let mut out = format!("\n Im Thema „{name}“ ist gerade nichts offen.\n");
        if !topic_notes.is_empty() {
            out.push_str(&format!("\n{topic_notes}"));
        }
        out.push_str(&format!(" {}\n", term::dim("Deine Themen: focus themen")));
        return out;
    }
    let width = term::list_width();
    let blocks = vec![
        Block {
            title: if none {
                "#ohne".to_string()
            } else {
                tags::topic_label(&name)
            },
            hint: if none {
                "zählt als nicht wichtig"
            } else if important {
                "wichtig"
            } else {
                "unter der Linie, nicht wichtig"
            }
            .into(),
            rows: open
                .iter()
                .map(|t| {
                    let mut r = row(
                        store,
                        t,
                        if t.status == Status::Today {
                            "▶"
                        } else {
                            "·"
                        },
                    );
                    r.right = (String::new(), Tone::Plain);
                    r
                })
                .collect(),
            note: open.is_empty().then(|| "Nichts offen bei dir.".to_string()),
        },
        Block {
            title: "LIEGT BEI ANDEREN".into(),
            hint: String::new(),
            rows: waiting.iter().map(|t| waiting_row(store, t)).collect(),
            note: None,
        },
    ];
    let mut out = String::from("\n");
    out.push_str(&layout(&blocks, width));
    if !topic_notes.is_empty() {
        out.push_str(&format!(" {}\n{topic_notes}\n", term::bold("NOTIZEN")));
    }
    if !done.is_empty() {
        let line = format!("In den letzten 7 Tagen erledigt: {}", done.join(", "));
        out.push_str(&format!(
            " {}\n",
            term::dim(&term::truncate(&line, width - 2))
        ));
    }
    if open.iter().any(|t| t.status == Status::Today) {
        out.push_str(&format!(" {}\n", term::dim("▶ steht auf heute")));
    }
    if none && let Some(t) = open.first().or(waiting.first()) {
        out.push_str(&format!(
            " {}\n",
            term::dim(&format!("Thema geben: focus ändern {} #thema", t.nr))
        ));
    }
    out
}

fn waiting_row(store: &Store, t: &Task) -> Row {
    let who = t
        .waiting_for
        .as_deref()
        .map(tags::person_label)
        .unwrap_or_default();
    let tone_age = if store.is_stale(t) {
        Tone::Warn
    } else {
        Tone::Dim
    };
    Row {
        nr: t.nr,
        mark: "·",
        text: t.display(),
        marker: t.marker(),
        mid: (format!("bei {who}"), Tone::Plain),
        right: (date::age_label(t.since, store.today), tone_age),
        faded: false,
    }
}

/// `focus warten` – alles, was bei anderen liegt, das Älteste zuerst.
pub fn waiting(store: &Store) -> String {
    let mut list: Vec<&Task> = store
        .tasks
        .iter()
        .filter(|t| t.status == Status::Waiting)
        .collect();
    if list.is_empty() {
        return format!(
            "\n Du wartest gerade auf niemanden.\n {}\n",
            term::dim("Abgeben geht so: focus abgeben 4 @tom")
        );
    }
    list.sort_by_key(|t| t.since);
    let stale = list.iter().any(|t| store.is_stale(t));
    let blocks = vec![Block {
        title: "WARTE AUF".into(),
        hint: if stale {
            "→ nachhaken: focus aufräumen".into()
        } else {
            String::new()
        },
        rows: list.iter().map(|t| waiting_row(store, t)).collect(),
        note: None,
    }];
    let mut out = String::from("\n");
    out.push_str(&layout(&blocks, term::list_width()));
    out.push_str(&format!(
        " {}\n",
        term::dim("Zurück zu dir: focus heute <Nr> · kam erledigt zurück: focus fertig <Nr>")
    ));
    out
}

/// Für die Suche: klein, Umlaute ausgeschrieben, damit „pruefen“ auch „prüfen“ findet.
fn fold(s: &str) -> String {
    s.to_lowercase()
        .replace('ä', "ae")
        .replace('ö', "oe")
        .replace('ü', "ue")
        .replace('ß', "ss")
}

/// `focus suche wort …` – Aufgaben, in denen alle Wörter vorkommen: im Text,
/// im Thema, bei der Person, in den Notizen. Dazu, was im Archiv passt.
pub fn search(store: &Store, words: &[String]) -> String {
    let wanted: Vec<String> = words
        .iter()
        .map(|w| fold(w))
        .filter(|w| !w.is_empty())
        .collect();
    let query = words.join(" ");
    let hits = |t: &Task| {
        let mut hay = vec![t.text.clone(), store.topic_name(t).unwrap_or_default()];
        hay.extend(t.waiting_for.clone());
        hay.extend(t.notes.iter().cloned());
        let hay = fold(&hay.join("\n"));
        wanted.iter().all(|w| hay.contains(w.as_str()))
    };
    let today = store.today;
    let mut open: Vec<&Task> = store
        .tasks
        .iter()
        .filter(|t| t.is_active() && hits(t))
        .collect();
    open.sort_by_key(|t| (t.status as u8, t.due.is_none(), t.due, t.nr));
    let gone: Vec<&Task> = store
        .tasks
        .iter()
        .filter(|t| t.is_restorable(today) && hits(t))
        .collect();
    if open.is_empty() && gone.is_empty() {
        return format!(
            "\n Zu „{query}“ finde ich nichts.\n {}\n",
            term::dim("Gesucht wird in Text, Thema, Personen und Notizen, auch im Archiv.")
        );
    }
    let blocks = vec![
        Block {
            title: format!("GEFUNDEN: {}", query.to_uppercase()),
            hint: String::new(),
            rows: open
                .iter()
                .map(|t| match t.status {
                    Status::Today => row(store, t, "▶"),
                    Status::Waiting => {
                        let mut r = waiting_row(store, t);
                        r.right = topic_cell(store, t);
                        r
                    }
                    _ => row(store, t, "·"),
                })
                .collect(),
            note: open
                .is_empty()
                .then(|| "Offen ist nichts davon.".to_string()),
        },
        Block {
            title: "IM ARCHIV".into(),
            hint: "zurückholen: focus zurück <Nr>".into(),
            rows: gone
                .iter()
                .map(|t| {
                    let done = t.status == Status::Done;
                    let mut r = row(store, t, if done { "✓" } else { "×" });
                    r.mid = (
                        if done { "erledigt" } else { "gestrichen" }.into(),
                        Tone::Dim,
                    );
                    r.faded = true;
                    r
                })
                .collect(),
            note: None,
        },
    ];
    let mut out = String::from("\n");
    out.push_str(&layout(&blocks, term::list_width()));
    out
}

/// `focus archiv` – Erledigtes und Gestrichenes der letzten 14 Tage.
pub fn archive(store: &Store) -> String {
    let today = store.today;
    let mut list: Vec<&Task> = store
        .tasks
        .iter()
        .filter(|t| t.is_restorable(today))
        .collect();
    if list.is_empty() {
        return format!(
            "\n Im Archiv ist nichts. {}\n",
            term::dim(&format!(
                "Erledigtes und Gestrichenes bleibt hier {KEEP_DAYS} Tage."
            ))
        );
    }
    list.sort_by_key(|t| std::cmp::Reverse(t.done_on.or(t.archived_on)));
    let when = |d: Option<NaiveDate>| match d.map(|d| (today - d).num_days()) {
        Some(0) => "heute".to_string(),
        Some(1) => "gestern".to_string(),
        Some(n) => format!("vor {n} Tagen"),
        None => String::new(),
    };
    let rows = |status: Status| -> Vec<Row> {
        list.iter()
            .filter(|t| t.status == status)
            .map(|t| Row {
                nr: t.nr,
                mark: if status == Status::Done { "✓" } else { "×" },
                text: t.display(),
                marker: t.marker(),
                mid: (when(t.done_on.or(t.archived_on)), Tone::Dim),
                right: topic_cell(store, t),
                faded: false,
            })
            .collect()
    };
    let blocks = vec![
        Block {
            title: "ERLEDIGT".into(),
            hint: String::new(),
            rows: rows(Status::Done),
            note: None,
        },
        Block {
            title: "GESTRICHEN".into(),
            hint: String::new(),
            rows: rows(Status::Archived),
            note: None,
        },
    ];
    let mut out = String::from("\n");
    out.push_str(&layout(&blocks, term::list_width()));
    out.push_str(&format!(
        " {}\n",
        term::dim(&format!(
            "Zurückholen: focus zurück <Nr> · nach {KEEP_DAYS} Tagen löscht Focus endgültig"
        ))
    ));
    out
}

/// `focus hilfe`
pub fn help(path: &Path, away: Option<&Path>) -> String {
    let rows: &[(&str, &str)] = &[
        ("focus", "zeigt, was heute dran ist"),
        (
            "focus + Text #thema @person termin",
            "schreibt eine Aufgabe auf",
        ),
        (
            "focus +",
            "fragt nach dem Text (gut bei Zeichen wie ' oder ?)",
        ),
        ("focus heute 4", "legt Aufgabe 4 auf heute"),
        ("focus später 4", "nimmt Aufgabe 4 wieder von heute"),
        ("focus fertig 4", "hakt Aufgabe 4 ab"),
        (
            "focus ändern 4 morgen",
            "ändert Text, #thema, @person oder Termin",
        ),
        ("focus 4", "zeigt Aufgabe 4 mit ihren Notizen"),
        (
            "focus notiz 4 text",
            "Notiz oder Link dazu, auch zu #thema (4.2 ändert eine)",
        ),
        ("focus notiz 4.2 löschen", "löscht die zweite Notiz"),
        ("focus öffnen 4", "öffnet den Link aus den Notizen"),
        ("focus jetzt", "zeigt nur eine Aufgabe, mit Uhr"),
        ("focus alles", "zeigt alle Aufgaben in vier Feldern"),
        (
            "focus themen",
            "zeigt deine Themen und wie es dort vorangeht",
        ),
        ("focus themen ordnen", "wählt die drei wichtigen Themen"),
        (
            "focus themen umbenennen a b",
            "benennt Thema a in b um (überall)",
        ),
        ("focus @anna", "zeigt alles, was du mit Anna hast"),
        ("focus personen", "zeigt alle Personen mit offenen Aufgaben"),
        ("focus #arbeit", "zeigt alles im Thema Arbeit"),
        ("focus #ohne", "zeigt die Aufgaben ohne Thema"),
        ("focus warten", "zeigt alles, was bei anderen liegt"),
        (
            "focus suche wort",
            "findet Aufgaben, auch in Notizen und im Archiv",
        ),
        (
            "focus abgeben 4 @tom",
            "gibt Aufgabe 4 an Tom und merkt sie als „warte auf“",
        ),
        (
            "focus streichen 4",
            "streicht Aufgabe 4 (14 Tage im Archiv)",
        ),
        (
            "focus archiv",
            "zeigt Erledigtes und Gestrichenes, 14 Tage lang",
        ),
        ("focus zurück 4", "holt Aufgabe 4 aus dem Archiv zurück"),
        (
            "focus aufräumen",
            "geht liegengebliebene Aufgaben mit dir durch",
        ),
        ("focus datei", "öffnet deine Aufgabendatei im Texteditor"),
        (
            "focus datei verschieben ort",
            "legt deine Aufgabendatei an einen anderen Ort",
        ),
        (
            "focus unterwegs ordner",
            "schreibt den Stand fürs Handy dorthin (aus: abschalten)",
        ),
        ("focus einrichten", "richtet Kurzbefehl f und Tab-Taste ein"),
        (
            "focus web",
            "öffnet Focus im Browser-Tab, im Look des Terminals",
        ),
        (
            "focus deinstallieren",
            "entfernt Focus wieder (deine Aufgaben nur auf Wunsch)",
        ),
        (
            "focus anleitung stichwort",
            "die ganze Anleitung, oder nur der passende Abschnitt",
        ),
    ];
    let mut out = format!("\n {}\n\n", term::bold("Focus – was ist heute dran?"));
    for (cmd, what) in rows {
        if term::text_width(cmd) > 28 {
            out.push_str(&format!(
                "   {}\n   {}{}\n",
                term::bold(cmd),
                " ".repeat(30),
                what
            ));
        } else {
            out.push_str(&format!(
                "   {}  {what}\n",
                term::bold(&term::pad_right(cmd, 28))
            ));
        }
    }
    out.push_str(&format!(
        "\n {}\n {}\n {}\n {}\n",
        term::dim("Termine: heute, morgen, freitag, 3.12., in 2 wochen · jeden montag, alle 2 wochen freitag"),
        term::dim("Statt „focus“ geht auch „f“, wenn du „focus einrichten“ benutzt hast."),
        term::dim("Mehr zu allem: focus anleitung, etwa focus anleitung unterwegs"),
        term::dim(&format!("Deine Aufgaben liegen in {}", path.display())),
    ));
    if let Some(a) = away {
        out.push_str(&format!(
            " {}\n",
            term::dim(&format!("Die Unterwegs-Datei liegt in {}", a.display()))
        ));
    }
    out
}

/// Für Bestätigungen: „Präsentation vorbereiten · Arbeit · fällig Freitag 27.11.“
pub fn summary(store: &Store, t: &Task) -> String {
    let mut parts = vec![t.display()];
    match store.topic_label(t) {
        Some(n) => parts.push(n),
        None => parts.push("ohne Thema".into()),
    }
    if let (Some(n), Some(d)) = (t.every, t.due) {
        parts.push(format!("↻ {}", date::recurrence_label(n, d)));
    }
    if let Some(d) = t.due {
        parts.push(match (d - store.today).num_days() {
            -6..=2 => format!("{} ({})", date::due_label(d, store.today), date::exact(d)),
            _ => format!("fällig {}", date::exact(d)),
        });
    }
    parts.join(" · ")
}

/// `focus 5`: eine Aufgabe mit allen Notizen.
pub fn detail(store: &Store, nr: u32) -> String {
    let Some(i) = store.find(nr) else {
        return format!(" Eine offene Aufgabe {nr} gibt es nicht.\n");
    };
    let t = &store.tasks[i];
    let mut facts = Vec::new();
    match t.status {
        Status::Today => facts.push("steht auf heute".to_string()),
        Status::Waiting => {
            if let Some(p) = &t.waiting_for {
                facts.push(format!("liegt bei {}", tags::person_label(p)));
            }
        }
        _ => {}
    }
    if let Some(n) = store.topic_label(t) {
        facts.push(n);
    } else {
        facts.push("ohne Thema".into());
    }
    if let Some(d) = t.due {
        facts.push(date::due_label(d, store.today));
    }
    if let Some(n) = t.every {
        facts.push(format!(
            "↻ {}",
            date::recurrence_label(n, t.due.unwrap_or(store.today))
        ));
    }
    facts.push(date::age_label(t.since, store.today));

    let target = notes::Target::Task(nr, None);
    let mut out = format!(
        "\n {}  {}\n     {}\n",
        term::dim(&nr.to_string()),
        term::bold(&t.display()),
        term::dim(&facts.join(" · "))
    );
    let lines = notes::lines(store, &target);
    if lines.is_empty() {
        out.push_str(&format!(
            "\n {}\n",
            term::dim(&format!(
                "Noch keine Notizen. Dazu: focus notiz {nr} Text oder Link"
            ))
        ));
    } else {
        out.push_str(&format!("\n{lines}\n"));
        let mut hints = vec![
            format!("neu: focus notiz {nr} …"),
            format!("ändern: focus notiz {nr}.1 …"),
        ];
        if !store::links(&t.notes.join(" ")).is_empty() || !store::links(&t.text).is_empty() {
            hints.push(format!("Link: focus öffnen {nr}"));
        }
        out.push_str(&format!(" {}\n", term::dim(&hints.join(" · "))));
    }
    out
}

/// Die Aufgaben auf heute, kurz, für Rückfragen.
pub fn today_short(store: &Store) -> String {
    let mut out = String::new();
    for &i in &store.today_open() {
        let t = &store.tasks[i];
        out.push_str(&format!("   {:>3}  {}\n", t.nr, t.display()));
    }
    out
}
