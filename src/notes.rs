//! Notizen und Links zu Aufgaben und Themen: `fokus notiz …`, `fokus öffnen …`.
//!
//! Eine Notiz ist eine Zeile Text oder ein Link, ohne Felder. Sie heißt nach
//! ihrer Aufgabe: `5.2` ist die zweite Notiz zu Aufgabe 5, `#arbeit.1` die
//! erste zum Thema Arbeit. In der Datei stehen sie eingerückt darunter.

use std::process::Command;

use crate::store::{self, Store};
use crate::{tags, term, view};

/// Wozu eine Notiz gehört, und welche gemeint ist (ab 1).
#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    Task(u32, Option<usize>),
    Topic(String, Option<usize>),
}

impl Target {
    /// `5`, `5.2`, `#arbeit`, `#arbeit.2`
    pub fn parse(s: &str) -> Option<Target> {
        let (head, idx) = match s.rsplit_once('.') {
            Some((h, i)) if !i.is_empty() => (h, Some(i.parse::<usize>().ok()?)),
            Some((h, _)) => (h, None),
            None => (s, None),
        };
        if idx == Some(0) {
            return None;
        }
        if head.starts_with('#') {
            let name = tags::clean(head);
            return (!tags::key(name).is_empty()).then(|| Target::Topic(name.to_string(), idx));
        }
        let nr = head.parse::<u32>().ok().filter(|&n| n > 0)?;
        Some(Target::Task(nr, idx))
    }

    fn index(&self) -> Option<usize> {
        match self {
            Target::Task(_, i) | Target::Topic(_, i) => *i,
        }
    }

    fn whole(&self) -> Target {
        match self {
            Target::Task(n, _) => Target::Task(*n, None),
            Target::Topic(t, _) => Target::Topic(t.clone(), None),
        }
    }
}

/// Die Aufgabennummer aus `5` oder `5.2`.
pub fn task_nr(s: &str) -> Option<u32> {
    match Target::parse(s)? {
        Target::Task(nr, _) => Some(nr),
        Target::Topic(..) => None,
    }
}

/// So heißt das Ding in Meldungen: „5“ oder „#arbeit“.
fn label(store: &Store, t: &Target) -> String {
    match t {
        Target::Task(n, _) => n.to_string(),
        Target::Topic(name, _) => format!("#{}", tags::to_tag(&topic_name(store, name))),
    }
}

fn topic_name(store: &Store, name: &str) -> String {
    store
        .all_topics()
        .find(|n| tags::key(n) == tags::key(name))
        .cloned()
        .unwrap_or_else(|| tags::display(name))
}

/// Gibt es das, wozu die Notiz gehören soll?
fn check(store: &Store, t: &Target) -> Result<(), String> {
    match t {
        Target::Task(n, _) => store.find(*n).map(|_| ()).ok_or_else(|| {
            format!("Eine offene Aufgabe {n} gibt es nicht. Die Nummern stehen in: fokus alles")
        }),
        Target::Topic(name, _) => {
            if tags::key(name) == tags::NO_TOPIC {
                return Err("„#ohne“ ist kein Thema, dazu gibt es keine Notizen.".into());
            }
            if store.all_topics().any(|n| tags::key(n) == tags::key(name)) {
                Ok(())
            } else {
                Err(format!(
                    "Ein Thema „{}“ kenne ich nicht. Deine Themen: fokus themen",
                    tags::display(name)
                ))
            }
        }
    }
}

pub fn notes<'a>(store: &'a Store, t: &Target) -> &'a [String] {
    match t {
        Target::Task(n, _) => store
            .find(*n)
            .map(|i| store.tasks[i].notes.as_slice())
            .unwrap_or(&[]),
        Target::Topic(name, _) => store.notes_of_topic(name),
    }
}

/// Zum Ändern. Bei einer Aufgabe zählt das als Bewegung: „seit“ wird heute.
fn notes_mut<'a>(store: &'a mut Store, t: &Target) -> &'a mut Vec<String> {
    match t {
        Target::Task(n, _) => {
            let i = store.find(*n).expect("vorher geprüft");
            let today = store.today;
            let task = &mut store.tasks[i];
            task.since = today;
            &mut task.notes
        }
        Target::Topic(name, _) => store.topic_notes.entry(tags::key(name)).or_default(),
    }
}

const DELETE: &[&str] = &["löschen", "loeschen", "weg", "streichen", "entfernen"];

/// `fokus notiz 5 Text` · `notiz 5.2 Text` · `notiz 5.2 löschen` · `notiz 5`
pub fn command(store: &mut Store, args: &[String]) -> Result<String, String> {
    let Some(first) = args.first() else {
        return Err(
            "Wozu? Zum Beispiel: fokus notiz 5 https://… oder fokus notiz #arbeit Wiki: …".into(),
        );
    };
    let target = Target::parse(first).ok_or_else(|| {
        format!("„{first}“ ist keine Aufgabe und kein Thema. Zum Beispiel: fokus notiz 5 Text")
    })?;
    check(store, &target)?;
    let text = args[1..].join(" ");
    let text = text.trim();
    let name = label(store, &target);

    match (target.index(), text) {
        (None, "") => {
            if term::is_interactive() {
                interactive(store, &target)
            } else {
                Ok(listing(store, &target))
            }
        }
        (None, text) => {
            let list = notes_mut(store, &target);
            list.push(text.to_string());
            Ok(format!(
                " {} Notiz {name}.{}: {text}",
                term::green("✓"),
                list.len()
            ))
        }
        (Some(i), text) => {
            let count = notes(store, &target).len();
            if i > count {
                return Err(match count {
                    0 => {
                        format!("Zu {name} gibt es noch keine Notiz. Neu: fokus notiz {name} Text")
                    }
                    1 => format!("Zu {name} gibt es nur eine Notiz: {name}.1"),
                    n => format!("Zu {name} gibt es nur {n} Notizen, bis {name}.{n}."),
                });
            }
            if text.is_empty() {
                return if term::is_interactive() {
                    edit_one(store, &target.whole(), i).map(|m| m.unwrap_or_default())
                } else {
                    Ok(format!("   {name}.{i}  {}", notes(store, &target)[i - 1]))
                };
            }
            if DELETE.contains(&text.to_lowercase().as_str()) {
                return Ok(delete(store, &target.whole(), i));
            }
            let list = notes_mut(store, &target);
            let old = std::mem::replace(&mut list[i - 1], text.to_string());
            Ok(format!(
                " {} Notiz {name}.{i} geändert: {text}\n   {}",
                term::green("✓"),
                term::dim(&format!("vorher: {old}"))
            ))
        }
    }
}

fn delete(store: &mut Store, target: &Target, i: usize) -> String {
    let name = label(store, target);
    let old = notes_mut(store, target).remove(i - 1);
    if let Target::Topic(n, _) = target
        && store.notes_of_topic(n).is_empty()
    {
        store.topic_notes.remove(&tags::key(n));
    }
    format!(
        " {} Notiz {name}.{i} gelöscht: {old}\n   {}",
        term::green("✓"),
        term::dim(&format!("Zurückholen: fokus notiz {name} {old}"))
    )
}

/// Die Notizen mit Nummern, eingerückt.
pub fn lines(store: &Store, target: &Target) -> String {
    let name = label(store, target);
    let mut out = String::new();
    for (k, n) in notes(store, target).iter().enumerate() {
        out.push_str(&format!(
            "   {}  {n}\n",
            term::dim(&format!("{name}.{}", k + 1))
        ));
    }
    out
}

fn listing(store: &Store, target: &Target) -> String {
    match target {
        Target::Task(n, _) => view::detail(store, *n),
        Target::Topic(..) => {
            let body = lines(store, target);
            let name = label(store, target);
            if body.is_empty() {
                format!(" Zu {name} gibt es noch keine Notiz. Neu: fokus notiz {name} Text")
            } else {
                format!("\n {}\n{body}", term::bold(&topic_heading(store, target)))
            }
        }
    }
}

fn topic_heading(store: &Store, target: &Target) -> String {
    match target {
        Target::Topic(name, _) => tags::topic_label(&topic_name(store, name)),
        Target::Task(..) => String::new(),
    }
}

/// `fokus notiz 5` im Terminal: Ziffer wählt, [n] neu, Enter fertig.
fn interactive(store: &mut Store, target: &Target) -> Result<String, String> {
    let name = label(store, target);
    loop {
        print!("{}", listing(store, target));
        let count = notes(store, target).len();
        let prompt = if count == 0 {
            " [n] neue Notiz · Enter fertig ".to_string()
        } else {
            format!(
                " Ziffer wählt eine Notiz{} · [n] neue · Enter fertig ",
                if count > 9 { " (1–9)" } else { "" }
            )
        };
        print!("\n{}", term::dim(&prompt));
        let key = term::read_key().map_err(term::io_err)?;
        println!();
        match key {
            '\n' | '\x1b' | 'q' => return Ok(String::new()),
            'n' | '+' => {
                let text = term::read_line(" Neue Notiz: ")
                    .map_err(term::io_err)?
                    .unwrap_or_default();
                if !text.is_empty() {
                    let list = notes_mut(store, target);
                    list.push(text.clone());
                    println!(" {} Notiz {name}.{}: {text}", term::green("✓"), list.len());
                }
            }
            c if c.is_ascii_digit() => {
                let i = c.to_digit(10).unwrap_or(0) as usize;
                if (1..=count).contains(&i)
                    && let Some(msg) = edit_one(store, target, i)?
                {
                    println!("{msg}");
                }
            }
            _ => {}
        }
    }
}

/// Eine Notiz: [ä] ändern, [l] löschen, Enter zurück.
fn edit_one(store: &mut Store, target: &Target, i: usize) -> Result<Option<String>, String> {
    let name = label(store, target);
    let old = notes(store, target)[i - 1].clone();
    println!("\n   {}  {old}", term::bold(&format!("{name}.{i}")));
    print!("{}", term::dim(" [ä] ändern · [l] löschen · Enter zurück "));
    loop {
        match term::read_key().map_err(term::io_err)? {
            'ä' | 'a' | 'e' => {
                println!();
                let text = term::read_line(" Neuer Text, Enter behält ihn: ")
                    .map_err(term::io_err)?
                    .unwrap_or_default();
                if text.is_empty() || text == old {
                    return Ok(Some(" Nichts geändert.".into()));
                }
                notes_mut(store, target)[i - 1] = text.clone();
                return Ok(Some(format!(
                    " {} Notiz {name}.{i} geändert: {text}",
                    term::green("✓")
                )));
            }
            'l' | 'x' => {
                println!();
                return Ok(Some(delete(store, target, i)));
            }
            '\n' | '\x1b' | 'q' => {
                println!();
                return Ok(None);
            }
            _ => {}
        }
    }
}

/// `fokus öffnen 5` · `öffnen 5.2` · `öffnen #arbeit`
pub fn open(store: &Store, args: &[String]) -> Result<String, String> {
    let first = args
        .first()
        .ok_or("Welchen Link? Zum Beispiel: fokus öffnen 5")?;
    let target = Target::parse(first)
        .ok_or_else(|| format!("„{first}“ ist keine Aufgabe und kein Thema."))?;
    check(store, &target)?;
    let name = label(store, &target);
    let all = notes(store, &target);
    let links: Vec<String> = match target.index() {
        Some(i) => {
            let note = all
                .get(i - 1)
                .ok_or_else(|| format!("Eine Notiz {name}.{i} gibt es nicht."))?;
            store::links(note)
        }
        None => {
            // Auch ein Link im Text der Aufgabe zählt, und zwar zuerst.
            let mut v = match &target {
                Target::Task(n, _) => store
                    .find(*n)
                    .map(|i| store::links(&store.tasks[i].text))
                    .unwrap_or_default(),
                Target::Topic(..) => Vec::new(),
            };
            for n in all {
                for l in store::links(n) {
                    if !v.contains(&l) {
                        v.push(l);
                    }
                }
            }
            v
        }
    };
    let url = match links.len() {
        0 => {
            return Err(match target.index() {
                Some(i) => format!("In Notiz {name}.{i} steht kein Link."),
                None => format!("Zu {name} gibt es keinen Link. Neu: fokus notiz {name} https://…"),
            });
        }
        1 => links[0].clone(),
        _ if term::is_interactive() => {
            for (k, l) in links.iter().enumerate().take(9) {
                println!("   {}  {l}", term::bold(&(k + 1).to_string()));
            }
            print!("{}", term::dim(" Welcher? Ziffer, Enter bricht ab "));
            let key = term::read_key().map_err(term::io_err)?;
            println!();
            match key.to_digit(10).map(|d| d as usize) {
                Some(d) if (1..=links.len().min(9)).contains(&d) => links[d - 1].clone(),
                _ => return Ok(" Nichts geöffnet.".into()),
            }
        }
        n => {
            let mut msg = format!("Zu {name} gibt es {n} Links:");
            for l in &links {
                msg.push_str(&format!("\n   {l}"));
            }
            msg.push_str(&format!(
                "\n Einen davon: fokus öffnen {name}.1 (Nummer der Notiz)"
            ));
            return Err(msg);
        }
    };
    launch(&url)?;
    Ok(format!(" ↗ {url}"))
}

/// Mit dem Standardbrowser öffnen; `FOKUS_BROWSER` kann ein anderes Programm nennen.
fn launch(url: &str) -> Result<(), String> {
    let program = std::env::var("FOKUS_BROWSER")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| term::opener().into());
    match Command::new(&program).arg(url).status() {
        Ok(s) if s.success() => Ok(()),
        _ => Err(format!("Ich konnte den Link nicht öffnen: {url}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nummern_und_themen_werden_verstanden() {
        assert_eq!(Target::parse("5"), Some(Target::Task(5, None)));
        assert_eq!(Target::parse("5.2"), Some(Target::Task(5, Some(2))));
        assert_eq!(Target::parse("5."), Some(Target::Task(5, None)));
        assert_eq!(
            Target::parse("#arbeit.1"),
            Some(Target::Topic("arbeit".into(), Some(1)))
        );
        assert_eq!(
            Target::parse("#eigenes-projekt"),
            Some(Target::Topic("eigenes-projekt".into(), None))
        );
        assert_eq!(Target::parse("5.0"), None);
        assert_eq!(Target::parse("0"), None);
        assert_eq!(Target::parse("abc"), None);
        assert_eq!(Target::parse("5.x"), None);
    }

    #[test]
    fn links_werden_gefunden() {
        assert_eq!(
            store::links("Figma: https://figma.com/file/abc, und mehr"),
            vec!["https://figma.com/file/abc"]
        );
        assert_eq!(
            store::links("(siehe http://example.org)."),
            vec!["http://example.org"]
        );
        assert!(store::links("kein Link, nur https:// allein").is_empty());
    }
}
