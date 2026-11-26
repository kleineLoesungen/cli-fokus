//! Was die Befehle an den Aufgaben ändern. Jede Funktion liefert die Rückmeldung als Text.

use crate::store::{IMPORTANT_LIMIT, Status, Store, TODAY_LIMIT, Task};
use crate::{date, input, tags, term, view};

pub fn not_found(nr: u32) -> String {
    format!("Eine offene Aufgabe {nr} gibt es nicht. Alle Nummern siehst du mit: fokus alles")
}

fn index(store: &Store, nr: u32) -> Result<usize, String> {
    store.find(nr).ok_or_else(|| not_found(nr))
}

/// `fokus + …`
pub fn add(store: &mut Store, raw: &str) -> Result<String, String> {
    let parsed = input::parse_new(raw, store.today);
    if parsed.text.trim().is_empty() {
        return Err("Da fehlt noch, was zu tun ist.".into());
    }
    let task = Task::new(parsed.text, parsed.due, store.today);
    let mut notes = Vec::new();
    if let Some(tag) = task.topic_tag()
        && let Some(above) = store.ensure_topic(&tag)
    {
        let name = tags::display(&tag);
        notes.push(if above {
            format!("Neues Thema „{name}“. Es steht über der Linie und zählt als wichtig.")
        } else {
            format!(
                "Neues Thema „{name}“. Über der Linie sind schon {IMPORTANT_LIMIT} Themen, also zählt es als nicht wichtig. Ändern: fokus themen ordnen"
            )
        });
    }
    let i = store.add_task(task);
    let t = &store.tasks[i];
    let mut msg = format!(
        " {} Aufgabe {}: {}",
        term::green("✓"),
        t.nr,
        view::summary(store, t)
    );
    if store.is_important(t) && store.is_urgent(t) && store.today_open().len() < TODAY_LIMIT {
        notes.push(format!(
            "Dringend und wichtig. Auf heute legen: fokus heute {}",
            t.nr
        ));
    }
    for n in notes {
        msg.push_str(&format!("\n   {}", term::dim(&n)));
    }
    Ok(msg)
}

/// `fokus heute 4 [statt 2]` – höchstens drei offene Aufgaben auf heute.
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
                    "Auf heute stehen schon {TODAY_LIMIT} Aufgaben. Sag, welche weichen soll: fokus heute {nr} statt <Nr>"
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

/// `fokus fertig 4`
pub fn done(store: &mut Store, nr: u32) -> Result<String, String> {
    let i = index(store, nr)?;
    let today = store.today;
    let t = &mut store.tasks[i];
    let was_today = t.status == Status::Today;
    t.on_today = was_today;
    t.status = Status::Done;
    t.done_on = Some(today);
    let mut msg = format!(" {} Erledigt: {}", term::green("✓"), t.display());
    if was_today && store.today_open().is_empty() {
        msg.push_str(&format!(
            "\n   {}",
            term::green("Alles für heute geschafft.")
        ));
    }
    Ok(msg)
}

/// `fokus später 4`
pub fn later(store: &mut Store, nr: u32) -> Result<String, String> {
    let i = index(store, nr)?;
    if store.tasks[i].status != Status::Today {
        return Err(format!("Aufgabe {nr} steht nicht auf heute."));
    }
    store.tasks[i].status = Status::Open;
    store.suggested = false;
    Ok(format!(" Aufgabe {nr} ist wieder von heute herunter."))
}

/// `fokus streichen 4`
pub fn strike(store: &mut Store, nr: u32) -> Result<String, String> {
    let i = index(store, nr)?;
    let today = store.today;
    let t = &mut store.tasks[i];
    t.status = Status::Archived;
    t.archived_on = Some(today);
    Ok(format!(
        " Gestrichen: {}\n   {}",
        t.display(),
        term::dim("Sie steht noch im Archiv deiner Datei (fokus datei).")
    ))
}

/// `fokus abgeben 4 @tom`
pub fn delegate(store: &mut Store, nr: u32, person: &str) -> Result<String, String> {
    let name = tags::clean(person);
    if tags::key(name).is_empty() {
        return Err(format!("An wen? Zum Beispiel: fokus abgeben {nr} @tom"));
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
    let shown = tags::display(name);
    Ok(format!(
        " Aufgabe {nr} liegt jetzt bei {shown}: {}\n   {}",
        t.display(),
        term::dim(&format!("Alles zu {shown}: fokus @{}", tags::key(name)))
    ))
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

/// `fokus themen ordnen 2 1 4` – genau diese (höchstens drei) kommen über die Linie.
pub fn reorder(store: &mut Store, picks: &[String]) -> Result<String, String> {
    if picks.len() > IMPORTANT_LIMIT {
        return Err(format!(
            "Höchstens {IMPORTANT_LIMIT} Themen dürfen über der Linie stehen. Genau darum geht es bei Fokus: Wenn alles wichtig ist, ist nichts wichtig."
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

/// `fokus themen ordnen` ohne Angaben: fragt nach.
pub fn reorder_interactive(store: &mut Store) -> Result<String, String> {
    let all: Vec<String> = store.all_topics().cloned().collect();
    if all.is_empty() {
        return Ok(" Noch keine Themen. Schreib #thema in eine Aufgabe.".into());
    }
    if !term::is_interactive() {
        return Err(
            "Sag, welche drei wichtig sind, zum Beispiel: fokus themen ordnen 2 1 4".into(),
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
            "„{s}“ ist keine Aufgabennummer. Die Nummern siehst du mit: fokus alles"
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
