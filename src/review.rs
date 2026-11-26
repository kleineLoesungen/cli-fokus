//! `fokus aufräumen`: eine Taste je liegengebliebener Aufgabe.

use crate::store::{STALE_OPEN_DAYS, Status, Store};
use crate::{date, ops, tags, term};

pub fn run(store: &mut Store) -> Result<(), String> {
    let mut open: Vec<usize> = (0..store.tasks.len())
        .filter(|&i| store.tasks[i].status == Status::Open && store.is_stale(&store.tasks[i]))
        .collect();
    open.sort_by_key(|&i| store.tasks[i].since);
    let mut waiting: Vec<usize> = (0..store.tasks.len())
        .filter(|&i| store.tasks[i].status == Status::Waiting && store.is_stale(&store.tasks[i]))
        .collect();
    waiting.sort_by_key(|&i| store.tasks[i].since);
    let nrs: Vec<u32> = open
        .into_iter()
        .chain(waiting)
        .map(|i| store.tasks[i].nr)
        .collect();

    if nrs.is_empty() {
        println!(
            "\n Alles frisch. Nichts liegt länger als {} Tage herum.",
            STALE_OPEN_DAYS
        );
        return Ok(());
    }

    if !term::is_interactive() {
        println!("\n Diese Aufgaben liegen schon länger:\n");
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
            "Aufgabe liegt schon länger."
        } else {
            "Aufgaben liegen schon länger."
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
            format!("[h] heute  [w] diese Woche  {t}  [b] behalten  [f] erledigt  [x] streichen")
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
            .map(tags::display)
            .unwrap_or_default();
        parts.push(format!("wartet auf {who}"));
    } else {
        parts.push(store.topic_name(t).unwrap_or_else(|| "kein Thema".into()));
    }
    if let Some(d) = t.due {
        parts.push(date::due_label(d, store.today));
    }
    parts.push(date::age_label(t.since, store.today));
    parts.join(" · ")
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
