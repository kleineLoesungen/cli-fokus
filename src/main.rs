//! Focus – was ist heute dran?

mod away;
mod complete;
mod config;
mod date;
mod focus;
mod i18n;
mod input;
mod manual;
mod notes;
mod ops;
mod review;
mod setup;
mod store;
mod tags;
mod term;
mod view;
mod web;

use std::process::ExitCode;

use store::Store;

fn main() -> ExitCode {
    // `focus | head` schließt die Ausgabe vorzeitig. Das ist kein Fehler, nur Schluss.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let payload = info.payload();
        let msg = payload
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| payload.downcast_ref::<&str>().copied())
            .unwrap_or("");
        if msg.contains("Broken pipe") {
            std::process::exit(0);
        }
        default_hook(info);
    }));

    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!(" {}", term::red(&msg));
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<(), String> {
    // Englische Befehle heißen intern wie die deutschen.
    let first = args.first().map(|s| i18n::canonical(s));
    if !matches!(first.as_deref(), Some("_complete" | "_zsh")) && config::flag(config::CLEAR_KEY) {
        fresh_screen(args);
    }
    match first.as_deref() {
        Some("hilfe" | "help" | "-h" | "--help" | "?") => {
            print!(
                "{}",
                view::help(&store::file_path(), away::path().as_deref())
            );
            return Ok(());
        }
        Some("--version" | "-v" | "version") => {
            println!("focus {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some("einrichten") => return setup::run(),
        Some("web" | "browser") => return web::run(&args[1..]),
        // Vor dem Laden: Lesen soll keine Aufgabendatei anlegen.
        Some("anleitung" | "handbuch" | "readme") => return manual::run(&args[1..]),
        // Für die Tab-Taste: nur lesen, nie schreiben, nie eine Fehlermeldung.
        Some("_complete") => {
            if let Ok(store) = Store::load(store::file_path(), date::today()) {
                for i in complete::candidates(&store, &args[1..]) {
                    println!("{}\t{}\t{}", i.group, i.value, i.shown);
                }
            }
            return Ok(());
        }
        Some("_zsh") => {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            print!("{}", complete::zsh_script(&exe));
            return Ok(());
        }
        // Vor dem Laden: Deinstallieren soll keine leere Aufgabendatei anlegen.
        Some("deinstallieren" | "entfernen" | "uninstall") => return setup::uninstall(),
        // Vor dem Laden: Sonst schriebe Focus am Ende die alte Datei wieder hin.
        Some("datei")
            if args
                .get(1)
                .is_some_and(|a| matches!(i18n::word(a).as_str(), "verschieben" | "nach")) =>
        {
            println!("{}", setup::move_data(&args[2..].join(" "))?);
            return Ok(());
        }
        _ => {}
    }

    let mut store = Store::load(store::file_path(), date::today())?;

    // Was unterwegs dazukam, kommt zuerst in die Liste.
    let away_before = away::path();
    let mut pickup = away::Pickup::default();
    if let Some(p) = &away_before {
        match away::pick_up(&mut store, p) {
            Ok(got) => pickup = got,
            Err(e) => println!(" {}", term::yellow(&e)),
        }
        if pickup.pending {
            for m in &pickup.messages {
                println!(" {}", term::dim(m));
            }
        } else if !pickup.messages.is_empty() {
            println!(
                " {}",
                term::bold(&tr!("Von unterwegs:", "From your phone:"))
            );
            for m in &pickup.messages {
                println!("{m}");
            }
            println!();
        }
    }

    let result = dispatch(&mut store, first.as_deref(), args);
    // Auch nach einem Fehler speichern: Was beim Laden geradegerückt wurde, soll bleiben.
    store.save()?;
    for m in &store.messages {
        println!("{}", term::dim(m));
    }

    // Erst wenn die Aufgaben sicher gespeichert sind, verschwinden die abgeholten
    // Zeilen aus der Unterwegs-Datei. Wurde der Ort eben geändert, bekommen der
    // alte und der neue Ort den Stand.
    let away_after = away::path();
    if let Some(p) = &away_before
        && !pickup.pending
        && let Err(e) = away::write(&store, p, &pickup.taken)
    {
        println!(" {}", term::yellow(&e));
    }
    if let Some(p) = away_after.filter(|a| away_before.as_ref() != Some(a))
        && let Err(e) = away::write(&store, &p, &[])
    {
        println!(" {}", term::yellow(&e));
    }
    result
}

fn dispatch(store: &mut Store, first: Option<&str>, args: &[String]) -> Result<(), String> {
    let rest = args.get(1..).unwrap_or_default();
    match first {
        None => print!("{}", view::today(store)),
        Some("heute") if rest.is_empty() => print!("{}", view::today(store)),
        Some("heute") => {
            let (nrs, instead) = numbers_with_instead(rest)?;
            for nr in nrs {
                println!("{}", ops::put_today(store, nr, instead)?);
            }
            list_after_change(store);
        }
        Some(f) if f.starts_with('+') || f == "neu" => {
            let mut words: Vec<String> = Vec::new();
            let head = args[0].trim_start_matches('+');
            if f != "neu" && !head.is_empty() {
                words.push(head.to_string());
            }
            words.extend(rest.iter().cloned());
            let mut text = words.join(" ");
            if text.trim().is_empty() {
                if !term::is_interactive() {
                    return Err(tr!(
                        "Was soll ich aufschreiben? Zum Beispiel: focus + Laufschuhe kaufen",
                        "What should I write down? For example: focus + buy running shoes"
                    ));
                }
                println!(
                    " {}",
                    term::dim(&tr!(
                        "Thema mit #, Person mit @, Termin am Ende (morgen, freitag, 3.12.)",
                        "Topic with #, person with @, date at the end (tomorrow, friday, 2026-12-03)"
                    ))
                );
                text = term::read_line(&tr!(" Was steht an? ", " What needs doing? "))
                    .map_err(term::io_err)?
                    .unwrap_or_default();
                if text.is_empty() {
                    return Ok(());
                }
            }
            println!("{}", ops::add(store, &text)?);
        }
        Some("fertig" | "erledigt") => {
            let nrs = numbers(rest, "fertig")?;
            for nr in nrs {
                println!("{}", ops::done(store, nr)?);
            }
            list_after_change(store);
        }
        Some("später" | "spaeter") => {
            for nr in numbers(rest, "später")? {
                println!("{}", ops::later(store, nr)?);
            }
            list_after_change(store);
        }
        Some("zurück" | "zurueck" | "rückgängig") => {
            for nr in numbers(rest, "zurück")? {
                println!("{}", ops::restore(store, nr)?);
            }
        }
        Some("archiv") => print!("{}", view::archive(store)),
        Some("streichen" | "löschen" | "loeschen") => {
            for nr in numbers(rest, "streichen")? {
                println!("{}", ops::strike(store, nr)?);
            }
        }
        Some("ändern" | "aendern" | "bearbeiten") => {
            let Some(first) = rest.first() else {
                return Err(tr!(
                    "Welche Aufgabe? Zum Beispiel: focus ändern 4 morgen",
                    "Which task? For example: focus edit 4 tomorrow"
                ));
            };
            let nr = ops::parse_nr(first)?;
            let mut change = rest[1..].join(" ");
            if change.trim().is_empty() {
                if !term::is_interactive() {
                    return Err(tr!(
                        "Was soll sich ändern? Zum Beispiel: focus ändern {nr} morgen",
                        "What should change? For example: focus edit {nr} tomorrow"
                    ));
                }
                let i = store.find(nr).ok_or_else(|| ops::not_found(nr))?;
                println!(
                    "{}",
                    tr!(
                        " Aufgabe {nr}: {}",
                        " Task {nr}: {}",
                        view::summary(store, &store.tasks[i])
                    )
                );
                println!(
                    " {}",
                    term::dim(&tr!(
                        "Schreib nur, was sich ändert: neuer Text, #thema, @person, Termin, ohne termin, ohne thema, ohne @name",
                        "Type only what changes: new text, #topic, @person, date, no date, no topic, no @name"
                    ))
                );
                change = term::read_line(&tr!(" Neu: ", " New: "))
                    .map_err(term::io_err)?
                    .unwrap_or_default();
                if change.is_empty() {
                    println!("{}", tr!(" Nichts geändert.", " Nothing changed."));
                    return Ok(());
                }
            }
            println!("{}", ops::change(store, nr, &change)?);
        }
        Some("abgeben") => {
            let (nr, person) = match rest {
                [nr, person] => (ops::parse_nr(nr)?, person.as_str()),
                [nr] => (ops::parse_nr(nr)?, ""),
                _ => {
                    return Err(tr!(
                        "So geht es: focus abgeben 4 @tom",
                        "Like this: focus delegate 4 @tom"
                    ));
                }
            };
            println!("{}", ops::delegate(store, nr, person)?);
        }
        Some("jetzt") => {
            let nr = rest.first().map(|s| ops::parse_nr(s)).transpose()?;
            focus::run(store, nr)?;
        }
        Some("alles") => print!("{}", view::matrix(store)),
        Some("themen" | "thema") => match rest.first().map(|s| i18n::word(s)).as_deref() {
            Some("ordnen") if rest.len() > 1 => {
                println!("{}", ops::reorder(store, &rest[1..])?);
                print!("{}", view::topics(store));
            }
            Some("ordnen") => {
                println!("{}", ops::reorder_interactive(store)?);
                print!("{}", view::topics(store));
            }
            Some("umbenennen") => match &rest[1..] {
                [old, new @ ..] if !new.is_empty() => {
                    println!("{}", ops::rename_topic(store, old, &new.join(" "))?);
                    print!("{}", view::topics(store));
                }
                _ => {
                    return Err(tr!(
                        "So geht es: focus themen umbenennen haus garten",
                        "Like this: focus topics rename house garden"
                    ));
                }
            },
            Some(other) => {
                return Err(tr!(
                    "„{other}“ kenne ich bei Themen nicht. Es gibt: focus themen ordnen, focus themen umbenennen",
                    "I don't know “{other}” for topics. There is: focus topics order, focus topics rename"
                ));
            }
            None => print!("{}", view::topics(store)),
        },
        Some("personen" | "leute") => print!("{}", view::persons(store)),
        Some(p) if p.starts_with('@') => print!("{}", view::person(store, &args[0])),
        Some(p) if p.starts_with('#') => print!("{}", view::topic(store, &args[0])),
        Some("warten" | "wartet" | "abgegeben") => print!("{}", view::waiting(store)),
        Some("suche" | "suchen" | "finde" | "finden") => {
            if rest.is_empty() {
                return Err(tr!(
                    "Wonach suchen? Zum Beispiel: focus suche angebot",
                    "Search for what? For example: focus search offer"
                ));
            }
            print!("{}", view::search(store, rest));
        }
        Some("aufräumen" | "aufraeumen") => review::run(store)?,
        Some("unterwegs") => println!("{}", away::command(rest)?),
        Some("notiz" | "notizen" | "note") => {
            let msg = notes::command(store, rest)?;
            if !msg.is_empty() {
                println!("{msg}");
            }
        }
        Some("öffnen" | "oeffnen" | "open" | "link") => println!("{}", notes::open(store, rest)?),
        // `focus 5` oder `focus 5.2`: eine Aufgabe mit ihren Notizen.
        Some(n) if notes::task_nr(n).is_some() => {
            print!("{}", view::detail(store, notes::task_nr(n).unwrap_or(0)));
        }
        Some("datei") => {
            println!(
                "{}",
                tr!(
                    " Deine Aufgaben liegen in {}",
                    " Your tasks are in {}",
                    store.path.display()
                )
            );
            if term::is_interactive() {
                store.save()?;
                // Am Mac im Texteditor, sonst mit dem Standardprogramm für .md.
                let mut open = std::process::Command::new(term::opener());
                if cfg!(target_os = "macos") {
                    open.arg("-t");
                }
                let _ = open.arg(&store.path).status();
            }
        }
        Some(other) => {
            return Err(tr!(
                "„{other}“ kenne ich nicht. Alle Befehle: focus hilfe\n Eine Aufgabe aufschreiben: focus + {}",
                "I don't know “{other}”. All commands: focus help\n To write down a task: focus + {}",
                args.join(" ")
            ));
        }
    }
    Ok(())
}

/// Leeres Fenster, oben der Befehl, der gerade läuft. Wer nur dieses Fenster für
/// Focus nutzt, sieht so immer nur den letzten Befehl und seine Antwort.
fn fresh_screen(args: &[String]) {
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        return;
    }
    term::clear_screen();
    println!(" {}\n", term::dim(&header(args)));
}

fn header(args: &[String]) -> String {
    let mut s = String::from("› focus");
    for a in args {
        s.push(' ');
        s.push_str(a);
    }
    s
}

/// Nach einer Änderung die Liste für heute. Im Browser steht sie schon oben.
fn list_after_change(store: &Store) {
    if std::env::var_os("FOCUS_WEB").is_none() {
        println!();
        print!("{}", view::today(store));
    }
}

fn numbers(args: &[String], verb: &str) -> Result<Vec<u32>, String> {
    if args.is_empty() {
        return Err(tr!(
            "Welche Aufgabe? Zum Beispiel: focus {} 2",
            "Which task? For example: focus {} 2",
            i18n::cmd(verb)
        ));
    }
    args.iter().map(|a| ops::parse_nr(a)).collect()
}

/// `4`, `4 7`, `4 statt 2`
fn numbers_with_instead(args: &[String]) -> Result<(Vec<u32>, Option<u32>), String> {
    match args.iter().position(|a| i18n::word(a) == "statt") {
        Some(p) => {
            let nrs = numbers(&args[..p], "heute")?;
            let instead = match &args[p + 1..] {
                [m] => ops::parse_nr(m)?,
                _ => {
                    return Err(tr!(
                        "So geht es: focus heute 4 statt 2",
                        "Like this: focus today 4 instead 2"
                    ));
                }
            };
            if nrs.len() != 1 {
                return Err(tr!(
                    "Mit „statt“ geht nur eine Aufgabe auf einmal.",
                    "“instead” works with one task at a time."
                ));
            }
            Ok((nrs, Some(instead)))
        }
        None => Ok((numbers(args, "heute")?, None)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn die_kopfzeile_zeigt_den_befehl() {
        assert_eq!(header(&[]), "› focus");
        let args: Vec<String> = ["+", "Test?", "morgen"].map(String::from).into();
        assert_eq!(header(&args), "› focus + Test? morgen");
    }
}
