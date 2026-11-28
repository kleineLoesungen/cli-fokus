//! `focus aufräumen`: eine Taste je Aufgabe, die überfällig ist oder lange liegt.

use crate::store::{STALE_OPEN_DAYS, Status, Store};
use crate::{date, i18n, input, ops, tags, term, tr, trs};

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
        println!("\n{}", trs!(" Nichts aufzuräumen.", " Nothing to review."));
        match planned {
            0 => println!(
                " {}",
                term::dim(&tr!(
                    "Nichts ist überfällig, nichts liegt länger als {STALE_OPEN_DAYS} Tage.",
                    "Nothing is overdue, nothing has been sitting longer than {STALE_OPEN_DAYS} days."
                ))
            ),
            1 => println!(
                " {}",
                term::dim(trs!(
                    "1 überfällige Aufgabe steht schon auf heute.",
                    "1 overdue task is already on today."
                ))
            ),
            n => println!(
                " {}",
                term::dim(&tr!(
                    "{n} überfällige Aufgaben stehen schon auf heute.",
                    "{n} overdue tasks are already on today."
                ))
            ),
        }
        return Ok(());
    }

    if !term::is_interactive() {
        println!(
            "\n{}\n",
            trs!(
                " Diese Aufgaben sind überfällig oder liegen schon länger:",
                " These tasks are overdue or have been sitting for a while:"
            )
        );
        for nr in &nrs {
            let i = store.find(*nr).unwrap();
            println!("   {:>3}  {}", nr, describe(store, i));
        }
        println!(
            "\n{}",
            trs!(
                " Zum Durchgehen: focus aufräumen in einem Terminalfenster.",
                " To go through them: focus review in a terminal window."
            )
        );
        return Ok(());
    }

    println!(
        "\n {} {}\n {}\n",
        term::bold(&nrs.len().to_string()),
        term::bold(match (i18n::en(), nrs.len() == 1) {
            (false, true) => "Aufgabe ist überfällig oder liegt schon länger.",
            (false, false) => "Aufgaben sind überfällig oder liegen schon länger.",
            (true, true) => "task is overdue or has been sitting for a while.",
            (true, false) => "tasks are overdue or have been sitting for a while.",
        }),
        term::dim(trs!(
            "Eine Taste je Aufgabe. Enter überspringt, q hört auf.",
            "One key per task. Enter skips, q stops."
        ))
    );

    for nr in nrs {
        let Some(i) = store.find(nr) else { continue };
        let waiting = store.tasks[i].status == Status::Waiting;
        println!(" {}", describe(store, i));
        let has_topic = store.tasks[i].topic_tag().is_some();
        let keys = match (i18n::en(), waiting) {
            (false, true) => "[n] nachhaken (auf heute)  [z] zurück zu mir  [b] weiter warten  [f] erledigt  [x] streichen".to_string(),
            (true, true) => "[f] follow up (on today)  [m] back to me  [k] keep waiting  [d] done  [x] drop".to_string(),
            (false, false) => {
                let t = if has_topic { "[t] anderes Thema" } else { "[t] Thema geben" };
                format!(
                    "[h] heute  [n] neuer Termin  [w] diese Woche  {t}\n [b] behalten  [f] erledigt  [x] streichen"
                )
            }
            (true, false) => {
                let t = if has_topic { "[o] other topic" } else { "[o] give a topic" };
                format!("[t] today  [n] new date  [w] this week  {t}\n [k] keep  [d] done  [x] drop")
            }
        };
        print!(" {}  ", term::dim(&keys));

        let result = loop {
            let key = german_key(term::read_key().map_err(term::io_err)?, waiting);
            let r = match (key, waiting) {
                ('\n', _) => Some(Ok(tr!(" übersprungen", " skipped"))),
                ('q' | '\x1b', _) => {
                    println!();
                    println!(
                        "\n{}",
                        trs!(
                            " Gut, für heute reicht das.",
                            " Fine, that's enough for today."
                        )
                    );
                    return Ok(());
                }
                ('f', _) => Some(ops::done(store, nr)),
                ('x', _) => Some(ops::strike(store, nr).map(|_| tr!(" gestrichen", " dropped"))),
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
        term::green(trs!(
            "Fertig. Deine Liste ist wieder ehrlich.",
            "Done. Your list is honest again."
        ))
    );
    Ok(())
}

/// Die englischen Tasten führen zu denselben Schritten wie die deutschen.
fn german_key(key: char, waiting: bool) -> char {
    if !i18n::en() {
        return key;
    }
    match (key, waiting) {
        ('f', true) => 'n',
        ('m', true) => 'z',
        ('k', _) => 'b',
        ('d', _) => 'f',
        ('t', false) => 'h',
        ('o', false) => 't',
        ('f' | 'h' | 'z' | 'b', _) => '?',
        (k, _) => k,
    }
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
        parts.push(tr!("wartet auf {who}", "waiting for {who}"));
    } else {
        parts.push(
            store
                .topic_label(t)
                .unwrap_or_else(|| tr!("kein Thema", "no topic")),
        );
    }
    if let Some(d) = t.due {
        parts.push(date::due_label(d, store.today));
    }
    parts.push(date::age_label(t.since, store.today));
    parts.join(" · ")
}

fn ask_due(store: &mut Store, nr: u32) -> Result<String, String> {
    loop {
        let answer = term::read_line(trs!(
            "   Neuer Termin (morgen, freitag, 3.12., in 2 wochen; ohne = keiner): ",
            "   New date (tomorrow, friday, 2026-12-03, in 2 weeks; none = no date): "
        ))
        .map_err(term::io_err)?
        .unwrap_or_default();
        match answer.to_lowercase().as_str() {
            "" => return Ok(tr!(" Termin bleibt.", " Date stays.")),
            "ohne" | "ohne termin" | "keiner" | "keinen" | "none" | "no date" => {
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
                    term::red(&tr!(
                        "„{answer}“ verstehe ich nicht als Termin. Noch einmal, oder Enter.",
                        "I don't understand “{answer}” as a date. Try again, or Enter."
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
    let answer = term::read_line(trs!(
        "   Thema (Nummer oder neuer Name, Enter bricht ab): ",
        "   Topic (number or new name, Enter cancels): "
    ))
    .map_err(term::io_err)?
    .unwrap_or_default();
    if answer.is_empty() {
        return Ok(tr!(" kein Thema vergeben", " no topic given"));
    }
    ops::set_topic(store, nr, &answer)
}
