//! Was die Befehle an den Aufgaben ändern. Jede Funktion liefert die Rückmeldung als Text.

use crate::store::{IMPORTANT_LIMIT, Status, Store, TODAY_LIMIT, Task};
use chrono::Days;

use crate::{date, input, tags, term, view};

pub fn not_found(nr: u32) -> String {
    format!("Eine offene Aufgabe {nr} gibt es nicht. Alle Nummern siehst du mit: focus alles")
}

fn index(store: &Store, nr: u32) -> Result<usize, String> {
    store.find(nr).ok_or_else(|| not_found(nr))
}

/// `focus + …`
pub fn add(store: &mut Store, raw: &str) -> Result<String, String> {
    let parsed = input::parse_new(raw, store.today);
    if tags::without_no_topic(&parsed.text).is_empty() {
        return Err("Da fehlt noch, was zu tun ist.".into());
    }
    let mut task = Task::new(parsed.text, parsed.due, store.today);
    task.every = parsed.every;
    let mut notes = Vec::new();
    if let Some(tag) = task.topic_tag()
        && let Some(above) = store.ensure_topic(&tag)
    {
        notes.push(new_topic_note(&tag, above));
    }
    let i = store.add_task(task);
    let nr = store.tasks[i].nr;
    let due_today = due_today_note(store, nr);
    let i = store.find(nr).expect("eben angelegt");
    let t = &store.tasks[i];
    let mut msg = format!(
        " {} Aufgabe {}: {}",
        term::green("✓"),
        t.nr,
        view::summary(store, t)
    );
    if let Some(n) = due_today {
        notes.insert(0, n);
    }
    for n in notes {
        msg.push_str(&format!("\n   {}", term::dim(&n)));
    }
    Ok(msg)
}

/// `focus heute 4 [statt 2]` – höchstens drei offene Aufgaben auf heute.
pub fn put_today(store: &mut Store, nr: u32, instead: Option<u32>) -> Result<String, String> {
    let i = index(store, nr)?;
    if store.tasks[i].status == Status::Today {
        return Ok(format!(" Aufgabe {nr} steht schon auf heute."));
    }
    let mut out = String::new();
    if store.today_open().len() >= TODAY_LIMIT {
        let leave = match instead {
            Some(m) => m,
            None if term::is_interactive() => {
                println!(
                    " Auf heute stehen schon {TODAY_LIMIT} Aufgaben:\n\n{}",
                    view::today_short(store)
                );
                let answer =
                    term::read_line(" Welche soll dafür weichen? Nummer, Enter bricht ab: ")
                        .map_err(term::io_err)?
                        .unwrap_or_default();
                if answer.is_empty() {
                    return Ok(" Nichts geändert.".into());
                }
                parse_nr(&answer)?
            }
            None => {
                return Err(format!(
                    "Auf heute stehen schon {TODAY_LIMIT} Aufgaben. Sag, welche weichen soll: focus heute {nr} statt <Nr>"
                ));
            }
        };
        let j = index(store, leave)?;
        if store.tasks[j].status != Status::Today {
            return Err(format!("Aufgabe {leave} steht nicht auf heute."));
        }
        store.tasks[j].status = Status::Open;
        out.push_str(&format!(
            " Aufgabe {leave} ist wieder von heute herunter.\n"
        ));
    }
    let i = index(store, nr)?;
    let i = store.move_to_end(i);
    let today = store.today;
    let t = &mut store.tasks[i];
    t.status = Status::Today;
    t.waiting_for = None;
    t.since = today;
    store.suggested = false;
    out.push_str(&format!(
        " Aufgabe {nr} steht jetzt auf heute: {}",
        store.tasks[i].display()
    ));
    Ok(out)
}

/// `focus fertig 4`
pub fn done(store: &mut Store, nr: u32) -> Result<String, String> {
    let i = index(store, nr)?;
    let today = store.today;
    let t = &mut store.tasks[i];
    let was_today = t.status == Status::Today;
    t.on_today = was_today;
    t.status = Status::Done;
    t.done_on = Some(today);
    let mut msg = format!(" {} Erledigt: {}", term::green("✓"), t.display());
    // Eine Wiederholung legt beim Abhaken die nächste an.
    if let (Some(n), Some(due)) = (t.every, t.due) {
        let step = Days::new(7 * n as u64);
        let mut next = due + step;
        while next <= today {
            next = next + step;
        }
        let mut again = Task::new(t.text.clone(), Some(next), today);
        again.every = Some(n);
        again.notes = t.notes.clone();
        let j = store.add_task(again);
        msg.push_str(&format!(
            "\n   ↻ Nächstes Mal: Aufgabe {}, fällig {}",
            store.tasks[j].nr,
            date::exact(next)
        ));
    }
    if was_today && store.today_open().is_empty() {
        msg.push_str(&format!(
            "\n   {}",
            term::green("Alles für heute geschafft.")
        ));
    }
    Ok(msg)
}

/// `focus später 4`
pub fn later(store: &mut Store, nr: u32) -> Result<String, String> {
    let i = index(store, nr)?;
    if store.tasks[i].status != Status::Today {
        return Err(format!("Aufgabe {nr} steht nicht auf heute."));
    }
    store.tasks[i].status = Status::Open;
    store.suggested = false;
    Ok(format!(" Aufgabe {nr} ist wieder von heute herunter."))
}

/// `focus streichen 4`
pub fn strike(store: &mut Store, nr: u32) -> Result<String, String> {
    let i = index(store, nr)?;
    let today = store.today;
    let t = &mut store.tasks[i];
    t.status = Status::Archived;
    t.archived_on = Some(today);
    let series = if t.every.is_some() {
        " Die Wiederholung endet damit."
    } else {
        ""
    };
    Ok(format!(" Gestrichen: {}.{series}", t.display()))
}

/// `focus zurück 4` – holt Erledigtes oder Gestrichenes aus dem Archiv.
pub fn restore(store: &mut Store, nr: u32) -> Result<String, String> {
    let i = store.find_restorable(nr).ok_or_else(|| {
        format!("Im Archiv gibt es keine Aufgabe {nr}. Was drin steht, zeigt: focus archiv")
    })?;
    let today = store.today;
    let old = store.tasks[i].clone();
    // Beim Abhaken einer Wiederholung entstand die nächste – die wird wieder entfernt.
    let mut note = String::new();
    if old.status == Status::Done && old.every.is_some() {
        let next = store.tasks.iter().position(|t| {
            t.is_active()
                && t.text == old.text
                && t.every == old.every
                && t.since == today
                && t.due > old.due
        });
        if let Some(j) = next {
            let removed = store.tasks.remove(j);
            note = format!(
                " Die schon angelegte nächste (Aufgabe {}) ist wieder weg.",
                removed.nr
            );
        }
    }
    let i = store.find_restorable(nr).expect("eben gefunden");
    let back_today =
        old.status == Status::Done && old.on_today && store.today_open().len() < TODAY_LIMIT;
    let t = &mut store.tasks[i];
    t.status = if back_today {
        Status::Today
    } else {
        Status::Open
    };
    t.done_on = None;
    t.archived_on = None;
    t.on_today = false;
    t.since = today;
    let place = if back_today {
        "wieder auf heute"
    } else {
        "wieder offen"
    };
    Ok(format!(" Aufgabe {nr} ist {place}: {}.{note}", t.display()))
}

/// `focus abgeben 4 @tom`
pub fn delegate(store: &mut Store, nr: u32, person: &str) -> Result<String, String> {
    let name = tags::clean(person);
    if tags::key(name).is_empty() {
        return Err(format!("An wen? Zum Beispiel: focus abgeben {nr} @tom"));
    }
    let i = index(store, nr)?;
    let today = store.today;
    let t = &mut store.tasks[i];
    if t.status == Status::Today {
        store.suggested = false;
    }
    let t = &mut store.tasks[i];
    t.status = Status::Waiting;
    t.waiting_for = Some(name.to_string());
    t.since = today;
    let shown = tags::person_label(name);
    Ok(format!(
        " Aufgabe {nr} liegt jetzt bei {shown}: {}",
        t.display()
    ))
}

/// Was heute fällig ist, gehört auf die Liste für heute – solange dort Platz ist.
/// Die Grenze von drei bleibt; ist sie erreicht, sagt Focus, wie man tauscht.
fn due_today_note(store: &mut Store, nr: u32) -> Option<String> {
    let i = store.find(nr)?;
    let t = &store.tasks[i];
    if t.status != Status::Open || t.due.is_none_or(|d| d > store.today) {
        return None;
    }
    let why = if t.due < Some(store.today) {
        "Schon überfällig"
    } else {
        "Fällig heute"
    };
    if store.today_open().len() >= TODAY_LIMIT {
        return Some(format!(
            "{why}, aber heute ist voll. Tauschen: focus heute {nr} statt <Nr>"
        ));
    }
    let i = store.move_to_end(i);
    store.tasks[i].status = Status::Today;
    store.suggested = false;
    Some(format!("{why}, steht jetzt auf heute."))
}

fn new_topic_note(tag: &str, above: bool) -> String {
    let name = tags::topic_label(tag);
    if above {
        format!("Neues Thema {name}, über der Linie.")
    } else {
        format!("Neues Thema {name}, unter der Linie. Ändern: focus themen ordnen")
    }
}

/// `focus ändern 4 …` – ändert nur, was angegeben ist, in derselben Schreibweise
/// wie beim Aufschreiben. Dazu `ohne termin`, `ohne thema` (oder `#ohne`), `ohne @anna`.
pub fn change(store: &mut Store, nr: u32, raw: &str) -> Result<String, String> {
    let i = index(store, nr)?;
    let today = store.today;

    let tokens: Vec<&str> = raw.split_whitespace().collect();
    let mut clear_due = false;
    let mut clear_topic = false;
    let mut drop_persons: Vec<String> = Vec::new();
    let mut clear_every = false;
    let mut rest: Vec<&str> = Vec::new();
    let mut k = 0;
    while k < tokens.len() {
        if tags::is_no_topic(tokens[k]) {
            clear_topic = true;
            k += 1;
            continue;
        }
        if tokens[k].eq_ignore_ascii_case("ohne") && k + 1 < tokens.len() {
            let next = tokens[k + 1];
            let handled = match next.to_lowercase().as_str() {
                "termin" | "datum" => {
                    clear_due = true;
                    clear_every = true;
                    true
                }
                "wiederholung" => {
                    clear_every = true;
                    true
                }
                "thema" => {
                    clear_topic = true;
                    true
                }
                _ if tags::is_tag(next, '@') => {
                    drop_persons.push(tags::key(tags::clean(next)));
                    true
                }
                _ => false,
            };
            if handled {
                k += 2;
                continue;
            }
        }
        rest.push(tokens[k]);
        k += 1;
    }

    let parsed = input::parse_change(&rest.join(" "), today);
    let given: Vec<&str> = parsed.text.split_whitespace().collect();
    let new_topic = given
        .iter()
        .find(|w| tags::is_tag(w, '#'))
        .map(|w| tags::clean(w).to_string());
    let new_persons: Vec<&str> = given
        .iter()
        .copied()
        .filter(|w| tags::is_tag(w, '@'))
        .collect();
    let new_words: Vec<&str> = given
        .iter()
        .copied()
        .filter(|w| !tags::is_tag(w, '#') && !tags::is_tag(w, '@'))
        .collect();
    if parsed.due.is_none()
        && !clear_every
        && !clear_due
        && !clear_topic
        && drop_persons.is_empty()
        && new_topic.is_none()
        && new_persons.is_empty()
        && new_words.is_empty()
    {
        return Err(format!(
            "Was soll sich ändern? Zum Beispiel: focus ändern {nr} morgen"
        ));
    }

    let is_person = |w: &str, key: &str| tags::is_tag(w, '@') && tags::key(tags::clean(w)) == key;
    let mut words: Vec<String> = store.tasks[i]
        .text
        .split_whitespace()
        .map(String::from)
        .collect();
    if !new_words.is_empty() {
        // Neuer Text; Personen und Thema der Aufgabe bleiben, wenn nichts anderes dasteht.
        let kept: Vec<String> = words
            .iter()
            .filter(|w| tags::is_tag(w, '@') || tags::is_tag(w, '#'))
            .cloned()
            .collect();
        words = new_words
            .iter()
            .map(|w| w.to_string())
            .chain(kept)
            .collect();
    }
    words.retain(|w| !drop_persons.iter().any(|k| is_person(w, k)));
    for p in &new_persons {
        let k = tags::key(tags::clean(p));
        if !words.iter().any(|w| is_person(w, &k)) {
            words.push(p.to_string());
        }
    }
    if clear_topic || new_topic.is_some() {
        words.retain(|w| !tags::is_tag(w, '#'));
    }
    if let Some(tp) = &new_topic {
        words.push(format!("#{tp}"));
    }
    if !words
        .iter()
        .any(|w| !tags::is_tag(w, '#') && !tags::is_tag(w, '@'))
    {
        return Err("Dann bliebe von der Aufgabe kein Text übrig.".into());
    }

    let mut notes = Vec::new();
    if let Some(tp) = &new_topic
        && let Some(above) = store.ensure_topic(tp)
    {
        notes.push(new_topic_note(tp, above));
    }
    let t = &mut store.tasks[i];
    t.text = words.join(" ");
    if let Some(d) = parsed.due {
        t.due = Some(d);
    } else if clear_due {
        t.due = None;
    }
    if parsed.every.is_some() {
        t.every = parsed.every;
    } else if clear_every {
        t.every = None;
    }
    t.since = today;
    if parsed.due.is_some()
        && let Some(n) = due_today_note(store, nr)
    {
        notes.insert(0, n);
    }
    let i = index(store, nr)?;
    let t = &store.tasks[i];
    let mut msg = format!(
        " {} Aufgabe {nr} geändert: {}",
        term::green("✓"),
        view::summary(store, t)
    );
    for n in notes {
        msg.push_str(&format!("\n   {}", term::dim(&n)));
    }
    Ok(msg)
}

/// `focus themen umbenennen haus garten` – in der Liste und in allen Aufgaben,
/// auch den erledigten. Gibt es das neue Thema schon, werden beide eins.
pub fn rename_topic(store: &mut Store, old: &str, new: &str) -> Result<String, String> {
    let all: Vec<String> = store.all_topics().cloned().collect();
    let from = match old.trim().parse::<usize>() {
        Ok(n) if (1..=all.len()).contains(&n) => all[n - 1].clone(),
        _ => all
            .iter()
            .find(|a| tags::key(a) == tags::key(old))
            .cloned()
            .ok_or_else(|| {
                format!(
                    "Ein Thema „{}“ kenne ich nicht. Deine Themen: focus themen",
                    old.trim_start_matches('#')
                )
            })?,
    };
    let new_name = tags::display(new.trim().trim_start_matches('#'));
    if tags::key(&new_name).is_empty() {
        return Err(
            "Wie soll das Thema heißen? Zum Beispiel: focus themen umbenennen haus garten".into(),
        );
    }
    if tags::key(&new_name) == tags::NO_TOPIC {
        return Err(
            "„Ohne“ ist vergeben: #ohne heißt bei Focus „kein Thema“. Such dir einen anderen Namen aus."
                .into(),
        );
    }
    let from_key = tags::key(&from);
    let merge_into = all
        .iter()
        .find(|n| tags::key(n) == tags::key(&new_name) && tags::key(n) != from_key)
        .cloned();
    let target = merge_into.clone().unwrap_or_else(|| new_name.clone());
    let new_tag = format!("#{}", tags::to_tag(&target));

    let mut changed = 0;
    for t in &mut store.tasks {
        if t.topic_tag().map(|g| tags::key(&g)) != Some(from_key.clone()) {
            continue;
        }
        let words: Vec<String> = t
            .text
            .split_whitespace()
            .map(|w| {
                if tags::is_tag(w, '#') && tags::key(tags::clean(w)) == from_key {
                    new_tag.clone()
                } else {
                    w.to_string()
                }
            })
            .collect();
        t.text = words.join(" ");
        if t.is_active() {
            changed += 1;
        }
    }

    let count = match changed {
        0 => "keine offene Aufgabe".to_string(),
        1 => "1 offene Aufgabe".to_string(),
        n => format!("{n} offene Aufgaben"),
    };
    // Die Notizen ziehen mit, beim Zusammenlegen hinten an.
    if let Some(moved) = store.topic_notes.remove(&from_key) {
        store
            .topic_notes
            .entry(tags::key(&target))
            .or_default()
            .extend(moved);
    }
    if let Some(target) = merge_into {
        store.important.retain(|n| tags::key(n) != from_key);
        store.other.retain(|n| tags::key(n) != from_key);
        Ok(format!(
            " {} „{from}“ und „{target}“ sind jetzt ein Thema: „{target}“ ({count} umgezogen).",
            term::green("✓")
        ))
    } else {
        for n in store.important.iter_mut().chain(store.other.iter_mut()) {
            if tags::key(n) == from_key {
                *n = new_name.clone();
            }
        }
        Ok(format!(
            " {} „{from}“ heißt jetzt „{new_name}“ ({count} geändert).",
            term::green("✓")
        ))
    }
}

/// Gibt einer Aufgabe ein Thema (Nummer aus der Themenliste oder ein Name).
pub fn set_topic(store: &mut Store, nr: u32, choice: &str) -> Result<String, String> {
    let i = index(store, nr)?;
    let all: Vec<String> = store.all_topics().cloned().collect();
    let name = match choice.trim().parse::<usize>() {
        Ok(n) if (1..=all.len()).contains(&n) => all[n - 1].clone(),
        Ok(_) => return Err(format!("Ein Thema {choice} gibt es nicht.")),
        Err(_) => choice.trim().trim_start_matches('#').to_string(),
    };
    if tags::key(&name).is_empty() {
        return Err("Da fehlt der Name des Themas.".into());
    }
    if tags::key(&name) == tags::NO_TOPIC {
        let today = store.today;
        let t = &mut store.tasks[i];
        t.text = t
            .text
            .split_whitespace()
            .filter(|w| !tags::is_tag(w, '#'))
            .collect::<Vec<_>>()
            .join(" ");
        t.since = today;
        return Ok(" ohne Thema".into());
    }
    let tag = tags::to_tag(&name);
    let existing = all
        .iter()
        .find(|n| tags::key(n) == tags::key(&name))
        .cloned();
    let today = store.today;
    let t = &mut store.tasks[i];
    let mut words: Vec<&str> = t
        .text
        .split_whitespace()
        .filter(|w| !tags::is_tag(w, '#'))
        .collect();
    let tagged = format!("#{tag}");
    words.push(&tagged);
    t.text = words.join(" ");
    t.since = today;
    let mut msg = format!(
        " Thema „{}“",
        existing.clone().unwrap_or_else(|| tags::display(&tag))
    );
    if existing.is_none() {
        match store.ensure_topic(&tag) {
            Some(true) => msg.push_str(" ist neu und steht über der Linie."),
            Some(false) => msg.push_str(" ist neu und steht unter der Linie."),
            None => {}
        }
    }
    Ok(msg)
}

/// `focus themen ordnen 2 1 4` – genau diese (höchstens drei) kommen über die Linie.
pub fn reorder(store: &mut Store, picks: &[String]) -> Result<String, String> {
    if picks.len() > IMPORTANT_LIMIT {
        return Err(format!(
            "Höchstens {IMPORTANT_LIMIT} Themen dürfen über der Linie stehen. Genau darum geht es bei Focus: Wenn alles wichtig ist, ist nichts wichtig."
        ));
    }
    let all: Vec<String> = store.all_topics().cloned().collect();
    let mut chosen: Vec<String> = Vec::new();
    for p in picks {
        let name = match p.parse::<usize>() {
            Ok(n) if (1..=all.len()).contains(&n) => all[n - 1].clone(),
            Ok(n) => return Err(format!("Ein Thema {n} gibt es nicht.")),
            Err(_) => all
                .iter()
                .find(|a| tags::key(a) == tags::key(p))
                .cloned()
                .ok_or_else(|| {
                    format!("Ein Thema „{}“ kenne ich nicht.", p.trim_start_matches('#'))
                })?,
        };
        if chosen.contains(&name) {
            return Err(format!("„{name}“ steht doppelt da."));
        }
        chosen.push(name);
    }
    let rest: Vec<String> = all.into_iter().filter(|n| !chosen.contains(n)).collect();
    store.important = chosen;
    store.other = rest;
    let msg = if store.important.is_empty() {
        " Über der Linie steht jetzt kein Thema. Dann ist nichts wichtig – sicher?".to_string()
    } else {
        format!(" Wichtig sind jetzt: {}", store.important.join(", "))
    };
    Ok(msg)
}

/// `focus themen ordnen` ohne Angaben: fragt nach.
pub fn reorder_interactive(store: &mut Store) -> Result<String, String> {
    let all: Vec<String> = store.all_topics().cloned().collect();
    if all.is_empty() {
        return Ok(" Noch keine Themen. Schreib #thema in eine Aufgabe.".into());
    }
    if !term::is_interactive() {
        return Err(
            "Sag, welche drei wichtig sind, zum Beispiel: focus themen ordnen 2 1 4".into(),
        );
    }
    println!("\n {}\n", term::bold("DEINE THEMEN"));
    for (i, n) in all.iter().enumerate() {
        let mark = if i < store.important.len() {
            term::dim("  (jetzt wichtig)")
        } else {
            String::new()
        };
        println!("   {:>2}  {n}{mark}", i + 1);
    }
    println!();
    let answer = term::read_line(&format!(
        " Welche {IMPORTANT_LIMIT} sind dir gerade am wichtigsten? Nummern der Reihe nach, z. B. 2 1 4\n Enter lässt alles, wie es ist: "
    ))
    .map_err(term::io_err)?
    .unwrap_or_default();
    if answer.is_empty() {
        return Ok(" Nichts geändert.".into());
    }
    let picks: Vec<String> = answer
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    reorder(store, &picks)
}

/// Für das Aufräumen: „Bei Anna nachhaken“ als neue Aufgabe auf heute.
pub fn follow_up(store: &mut Store, nr: u32) -> Result<String, String> {
    let i = index(store, nr)?;
    let today = store.today;
    let t = &store.tasks[i];
    let who = t.waiting_for.clone().unwrap_or_default();
    let mut text = format!("Bei {} nachhaken: {}", tags::display(&who), t.display());
    if let Some(tag) = t.topic_tag() {
        text.push_str(&format!(" #{tag}"));
    }
    store.tasks[i].since = today;
    let new = store.add_task(Task::new(text, None, today));
    let new_nr = store.tasks[new].nr;
    put_today(store, new_nr, None)
}

/// „Diese Woche“: fällig am Sonntag, außer es ist schon früher fällig.
pub fn this_week(store: &mut Store, nr: u32) -> Result<String, String> {
    let i = index(store, nr)?;
    let today = store.today;
    let sunday = date::end_of_week(today);
    let t = &mut store.tasks[i];
    // Ein früherer Termin bleibt, ein verstrichener nicht.
    t.due = Some(match t.due {
        Some(d) if d >= today && d < sunday => d,
        _ => sunday,
    });
    t.since = today;
    Ok(format!(" fällig {}", date::exact(t.due.unwrap())))
}

pub fn keep(store: &mut Store, nr: u32) -> Result<String, String> {
    let i = index(store, nr)?;
    store.tasks[i].since = store.today;
    Ok(" bleibt, wie sie ist.".into())
}

pub fn back_to_me(store: &mut Store, nr: u32) -> Result<String, String> {
    let i = index(store, nr)?;
    let today = store.today;
    let t = &mut store.tasks[i];
    t.status = Status::Open;
    t.waiting_for = None;
    t.since = today;
    Ok(" liegt wieder bei dir.".into())
}

pub fn parse_nr(s: &str) -> Result<u32, String> {
    let s = s.trim().trim_end_matches([',', '.']);
    match s.parse::<u32>() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(format!(
            "„{s}“ ist keine Aufgabennummer. Die Nummern siehst du mit: focus alles"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::path::PathBuf;

    fn leer() -> Store {
        Store::parse(
            "",
            PathBuf::from("t.md"),
            NaiveDate::from_ymd_opt(2026, 11, 26).unwrap(),
        )
    }

    #[test]
    fn die_ersten_drei_themen_sind_wichtig_das_vierte_nicht() {
        let mut s = leer();
        for t in ["A #eins", "B #zwei", "C #drei", "D #vier"] {
            add(&mut s, t).unwrap();
        }
        assert_eq!(s.important, vec!["Eins", "Zwei", "Drei"]);
        assert_eq!(s.other, vec!["Vier"]);
    }

    #[test]
    fn heute_nimmt_hoechstens_drei() {
        let mut s = leer();
        for t in ["A", "B", "C", "D"] {
            add(&mut s, t).unwrap();
        }
        for nr in 1..=3 {
            put_today(&mut s, nr, None).unwrap();
        }
        assert!(put_today(&mut s, 4, None).is_err());
        put_today(&mut s, 4, Some(2)).unwrap();
        let heute: Vec<u32> = s.today_open().iter().map(|&i| s.tasks[i].nr).collect();
        assert_eq!(heute, vec![1, 3, 4]);
    }

    #[test]
    fn mehr_als_drei_wichtige_themen_lehnt_ordnen_ab() {
        let mut s = leer();
        for t in ["A #eins", "B #zwei", "C #drei", "D #vier"] {
            add(&mut s, t).unwrap();
        }
        let vier: Vec<String> = ["1", "2", "3", "4"].iter().map(|s| s.to_string()).collect();
        assert!(reorder(&mut s, &vier).is_err());
        let neu: Vec<String> = ["4", "eins"].iter().map(|s| s.to_string()).collect();
        reorder(&mut s, &neu).unwrap();
        assert_eq!(s.important, vec!["Vier", "Eins"]);
        assert_eq!(s.other, vec!["Zwei", "Drei"]);
    }

    #[test]
    fn abgeben_und_zurueck() {
        let mut s = leer();
        add(&mut s, "Fotos schicken").unwrap();
        delegate(&mut s, 1, "@anna").unwrap();
        assert_eq!(s.tasks[0].status, Status::Waiting);
        assert_eq!(s.tasks[0].waiting_for.as_deref(), Some("anna"));
        back_to_me(&mut s, 1).unwrap();
        assert_eq!(s.tasks[0].status, Status::Open);
    }

    #[test]
    fn thema_geben_ersetzt_das_alte() {
        let mut s = leer();
        add(&mut s, "Keller ausmisten #haus").unwrap();
        set_topic(&mut s, 1, "Eigenes Projekt").unwrap();
        assert_eq!(s.tasks[0].text, "Keller ausmisten #eigenes-projekt");
    }

    #[test]
    fn aendern_aendert_nur_was_dasteht() {
        let mut s = leer();
        add(&mut s, "Mit @anna Urlaub abstimmen").unwrap();
        change(&mut s, 1, "#familie").unwrap();
        assert_eq!(s.tasks[0].text, "Mit @anna Urlaub abstimmen #familie");
        change(&mut s, 1, "@tom morgen").unwrap();
        assert_eq!(s.tasks[0].text, "Mit @anna Urlaub abstimmen #familie @tom");
        assert!(s.tasks[0].due.is_some());
        change(&mut s, 1, "ohne termin ohne @anna").unwrap();
        assert_eq!(s.tasks[0].text, "Mit Urlaub abstimmen #familie @tom");
        assert!(s.tasks[0].due.is_none());
        change(&mut s, 1, "Urlaub buchen").unwrap();
        assert_eq!(s.tasks[0].text, "Urlaub buchen #familie @tom");
        change(&mut s, 1, "ohne thema").unwrap();
        assert_eq!(s.tasks[0].text, "Urlaub buchen @tom");
        assert!(change(&mut s, 1, "").is_err());
    }

    #[test]
    fn termin_heute_kommt_auf_die_liste_solange_platz_ist() {
        let mut s = leer();
        let msg = add(&mut s, "Rechnung zahlen heute").unwrap();
        assert!(msg.contains("steht jetzt auf heute"));
        add(&mut s, "Keller").unwrap();
        add(&mut s, "Bericht morgen").unwrap();
        // Ein Termin morgen ändert an der Liste nichts.
        assert_eq!(s.today_open().len(), 1);
        change(&mut s, 2, "heute").unwrap();
        assert_eq!(s.today_open().len(), 2);
        // Andere Änderungen legen nichts auf heute, auch wenn heute fällig.
        later(&mut s, 2).unwrap();
        change(&mut s, 2, "#haus").unwrap();
        assert_eq!(s.today_open().len(), 1);
    }

    #[test]
    fn termin_heute_bei_voller_liste_nennt_den_tausch() {
        let mut s = leer();
        for t in ["A", "B", "C", "D"] {
            add(&mut s, t).unwrap();
        }
        for nr in 1..=3 {
            put_today(&mut s, nr, None).unwrap();
        }
        let msg = change(&mut s, 4, "heute").unwrap();
        assert!(msg.contains("focus heute 4 statt <Nr>"), "{msg}");
        assert_eq!(s.today_open().len(), 3);
    }

    #[test]
    fn wiederholung_legt_beim_abhaken_die_naechste_an() {
        let mut s = leer();
        let msg = add(&mut s, "Wochenplan #arbeit jeden montag").unwrap();
        assert!(msg.contains("jeden Montag"), "{msg}");
        let msg = done(&mut s, 1).unwrap();
        assert!(
            msg.contains("Nächstes Mal: Aufgabe 2, fällig Montag 7.12."),
            "{msg}"
        );
        // Versehen: zurück holt die alte und entfernt die nächste wieder.
        restore(&mut s, 1).unwrap();
        let offen: Vec<u32> = s
            .tasks
            .iter()
            .filter(|t| t.is_active())
            .map(|t| t.nr)
            .collect();
        assert_eq!(offen, vec![1]);
        assert_eq!(s.tasks[0].due.map(date::to_file), Some("30.11.2026".into()));
    }

    #[test]
    fn zurueck_holt_erledigtes_auf_heute_und_gestrichenes_zurueck() {
        let mut s = leer();
        add(&mut s, "A").unwrap();
        add(&mut s, "B").unwrap();
        put_today(&mut s, 1, None).unwrap();
        done(&mut s, 1).unwrap();
        strike(&mut s, 2).unwrap();
        assert!(restore(&mut s, 1).unwrap().contains("wieder auf heute"));
        assert!(restore(&mut s, 2).unwrap().contains("wieder offen"));
        assert!(restore(&mut s, 2).is_err());
    }

    #[test]
    fn aendern_setzt_und_entfernt_die_wiederholung() {
        let mut s = leer();
        add(&mut s, "Gießen").unwrap();
        change(&mut s, 1, "alle 2 wochen freitag").unwrap();
        assert_eq!(s.tasks[0].every, Some(2));
        change(&mut s, 1, "ohne wiederholung").unwrap();
        assert_eq!(s.tasks[0].every, None);
        assert!(s.tasks[0].due.is_some());
    }

    #[test]
    fn aendern_legt_ein_neues_thema_an() {
        let mut s = leer();
        add(&mut s, "Keller ausmisten").unwrap();
        let msg = change(&mut s, 1, "#haus").unwrap();
        assert!(msg.contains("Neues Thema #haus"));
        assert_eq!(s.important, vec!["Haus"]);
    }

    #[test]
    fn umbenennen_aendert_liste_und_aufgaben() {
        let mut s = leer();
        add(&mut s, "Bericht #arbiet").unwrap();
        add(&mut s, "Angebot #arbiet").unwrap();
        done(&mut s, 2).unwrap();
        rename_topic(&mut s, "arbiet", "Arbeit").unwrap();
        assert_eq!(s.important, vec!["Arbeit"]);
        assert_eq!(s.tasks[0].text, "Bericht #arbeit");
        // Auch Erledigtes zieht mit, damit die Wochenpunkte stimmen.
        assert_eq!(s.tasks[1].text, "Angebot #arbeit");
    }

    #[test]
    fn umbenennen_auf_ein_vorhandenes_thema_legt_zusammen() {
        let mut s = leer();
        add(&mut s, "A #arbeit").unwrap();
        add(&mut s, "B #job").unwrap();
        let msg = rename_topic(&mut s, "job", "arbeit").unwrap();
        assert!(msg.contains("ein Thema"));
        assert_eq!(s.important, vec!["Arbeit"]);
        assert_eq!(s.tasks[1].text, "B #arbeit");
        assert!(rename_topic(&mut s, "gibtsnicht", "x").is_err());
    }

    #[test]
    fn nachhaken_landet_auf_heute() {
        let mut s = leer();
        add(&mut s, "Fotos schicken #familie").unwrap();
        delegate(&mut s, 1, "@anna").unwrap();
        follow_up(&mut s, 1).unwrap();
        let heute: Vec<&str> = s
            .today_open()
            .iter()
            .map(|&i| s.tasks[i].text.as_str())
            .collect();
        assert_eq!(heute, vec!["Bei Anna nachhaken: Fotos schicken #familie"]);
    }
}
