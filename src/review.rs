//! `fokus aufräumen`: eine Taste je Aufgabe, die überfällig ist oder lange liegt.

use crate::store::{STALE_OPEN_DAYS, Status, Store};
use crate::{date, input, ops, tags, term};

/// Erst Überfälliges (ab dem Tag nach dem Termin), dann lange Liegendes;
/// je zuerst, was bei dir liegt, dann, was bei anderen liegt.
fn candidates(store: &Store) -> Vec<u32> {
    let pick = |status: Status, overdue: bool| -> Vec<usize> {
        let mut v: Vec<usize> = (0..store.tasks.len())
            .filter(|&i| {
                let t = &store.tasks[i];
                t.status == status
                    && if overdue {
                        store.is_overdue(t)
                    } else {
                        !store.is_overdue(t) && store.is_stale(t)
                    }
            })
            .collect();
        if overdue {
            v.sort_by_key(|&i| store.tasks[i].due);
        } else {
            v.sort_by_key(|&i| store.tasks[i].since);
        }
        v
    };
    pick(Status::Open, true)
        .into_iter()
        .chain(pick(Status::Waiting, true))
        .chain(pick(Status::Open, false))
        .chain(pick(Status::Waiting, false))
        .map(|i| store.tasks[i].nr)
        .collect()
}

pub fn run(store: &mut Store) -> Result<(), String> {
    let nrs = candidates(store);

    if nrs.is_empty() {
        // Überfälliges auf der Liste für heute ist schon eingeplant – aber nicht „frisch“.
        let planned = store
            .tasks
            .iter()
            .filter(|t| t.status == Status::Today && t.due.is_some_and(|d| d < store.today))
            .count();
        println!("\n Nichts aufzuräumen.");
        match planned {
            0 => println!(
                " {}",
                term::dim(&format!(
                    "Nichts ist überfällig, nichts liegt länger als {STALE_OPEN_DAYS} Tage."
                ))
            ),
            1 => println!(
                " {}",
                term::dim("1 überfällige Aufgabe steht schon auf heute.")
            ),
            n => println!(
                " {}",
                term::dim(&format!("{n} überfällige Aufgaben stehen schon auf heute."))
            ),
        }
        return Ok(());
    }

    if !term::is_interactive() {
        println!("\n Diese Aufgaben sind überfällig oder liegen schon länger:\n");
        for nr in &nrs {
            let i = store.find(*nr).unwrap();
            println!("   {:>3}  {}", nr, describe(store, i));
        }
        println!("\n Zum Durchgehen: fokus aufräumen in einem Terminalfenster.");
        return Ok(());
    }

    println!(
        "\n {} {}\n {}\n",
        term::bold(&nrs.len().to_string()),
        term::bold(if nrs.len() == 1 {
            "Aufgabe ist überfällig oder liegt schon länger."
        } else {
            "Aufgaben sind überfällig oder liegen schon länger."
        }),
        term::dim("Eine Taste je Aufgabe. Enter überspringt, q hört auf.")
    );

    for nr in nrs {
        let Some(i) = store.find(nr) else { continue };
        let waiting = store.tasks[i].status == Status::Waiting;
        println!(" {}", describe(store, i));
        let keys = if waiting {
            "[n] nachhaken (auf heute)  [z] zurück zu mir  [b] weiter warten  [f] erledigt  [x] streichen"
                .to_string()
        } else {
            let t = if store.tasks[i].topic_tag().is_some() {
                "[t] anderes Thema"
            } else {
                "[t] Thema geben"
            };
            format!(
                "[h] heute  [n] neuer Termin  [w] diese Woche  {t}\n [b] behalten  [f] erledigt  [x] streichen"
            )
        };
        print!(" {}  ", term::dim(&keys));

        let result = loop {
            let key = term::read_key().map_err(term::io_err)?;
            let r = match (key, waiting) {
                ('\n', _) => Some(Ok(" übersprungen".to_string())),
                ('q' | '\x1b', _) => {
                    println!();
                    println!("\n Gut, für heute reicht das.");
                    return Ok(());
                }
                ('f', _) => Some(ops::done(store, nr)),
                ('x', _) => Some(ops::strike(store, nr).map(|_| " gestrichen".to_string())),
                ('b', _) => Some(ops::keep(store, nr)),
                ('n', true) => Some(ops::follow_up(store, nr)),
                ('z', true) => Some(ops::back_to_me(store, nr)),
                ('h', false) => {
                    println!();
                    Some(ops::put_today(store, nr, None))
                }
                ('w', false) => Some(ops::this_week(store, nr)),
                ('n', false) => {
                    println!();
                    Some(ask_due(store, nr))
                }
                ('t', false) => {
                    println!();
                    Some(ask_topic(store, nr))
                }
                _ => None,
            };
            if let Some(r) = r {
                break r;
            }
        };
        match result {
            Ok(msg) => println!(
                "{}",
                term::dim(&format!("→ {}", msg.trim_start_matches(' ')))
            ),
            Err(e) => println!("{}", term::red(&e)),
        }
        println!();
        store.save()?;
    }
    println!(
        " {}",
        term::green("Fertig. Deine Liste ist wieder ehrlich.")
    );
    Ok(())
}

fn describe(store: &Store, i: usize) -> String {
    let t = &store.tasks[i];
    let mut parts = vec![term::bold(&t.display())];
    if t.status == Status::Waiting {
        let who = t
            .waiting_for
            .as_deref()
            .map(tags::person_label)
            .unwrap_or_default();
        parts.push(format!("wartet auf {who}"));
    } else {
        parts.push(store.topic_label(t).unwrap_or_else(|| "kein Thema".into()));
    }
    if let Some(d) = t.due {
        parts.push(date::due_label(d, store.today));
    }
    parts.push(date::age_label(t.since, store.today));
    parts.join(" · ")
}

fn ask_due(store: &mut Store, nr: u32) -> Result<String, String> {
    loop {
        let answer = term::read_line(
            "   Neuer Termin (morgen, freitag, 3.12., in 2 wochen; ohne = keiner): ",
        )
        .map_err(term::io_err)?
        .unwrap_or_default();
        match answer.to_lowercase().as_str() {
            "" => return Ok(" Termin bleibt.".into()),
            "ohne" | "ohne termin" | "keiner" | "keinen" => {
                return ops::change(store, nr, "ohne termin");
            }
            _ => {
                // Nur einen Termin annehmen, nie aus Versehen den Text überschreiben.
                let parsed = input::parse_change(&answer, store.today);
                if parsed.due.is_some() && parsed.text.trim().is_empty() {
                    return ops::change(store, nr, &answer);
                }
                println!(
                    "   {}",
                    term::red(&format!(
                        "„{answer}“ verstehe ich nicht als Termin. Noch einmal, oder Enter."
                    ))
                );
            }
        }
    }
}

fn ask_topic(store: &mut Store, nr: u32) -> Result<String, String> {
    let all: Vec<String> = store.all_topics().cloned().collect();
    for (n, name) in all.iter().enumerate() {
        println!("     {:>2}  {name}", n + 1);
    }
    let answer = term::read_line("   Thema (Nummer oder neuer Name, Enter bricht ab): ")
        .map_err(term::io_err)?
        .unwrap_or_default();
    if answer.is_empty() {
        return Ok(" kein Thema vergeben".into());
    }
    ops::set_topic(store, nr, &answer)
}
