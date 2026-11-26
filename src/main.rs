//! Fokus – was ist heute dran?

mod date;
mod focus;
mod input;
mod ops;
mod review;
mod setup;
mod store;
mod tags;
mod term;
mod view;

use std::process::ExitCode;

use store::Store;

fn main() -> ExitCode {
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
    let first = args.first().map(|s| s.to_lowercase());
    match first.as_deref() {
        Some("hilfe" | "help" | "-h" | "--help" | "?") => {
            print!("{}", view::help(&store::file_path()));
            return Ok(());
        }
        Some("--version" | "-v" | "version") => {
            println!("fokus {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some("einrichten") => return setup::run(),
        _ => {}
    }

    let mut store = Store::load(store::file_path(), date::today())?;
    let result = dispatch(&mut store, first.as_deref(), args);
    // Auch nach einem Fehler speichern: Was beim Laden geradegerückt wurde, soll bleiben.
    store.save()?;
    for m in &store.messages {
        println!("{}", term::dim(m));
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
            println!();
            print!("{}", view::today(store));
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
                    return Err(
                        "Was soll ich aufschreiben? Zum Beispiel: fokus + Laufschuhe kaufen".into(),
                    );
                }
                println!(
                    " {}",
                    term::dim("Thema mit #, Person mit @, Termin am Ende (morgen, freitag, 3.12.)")
                );
                text = term::read_line(" Was steht an? ")
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
            println!();
            print!("{}", view::today(store));
        }
        Some("später" | "spaeter") => {
            for nr in numbers(rest, "später")? {
                println!("{}", ops::later(store, nr)?);
            }
            println!();
            print!("{}", view::today(store));
        }
        Some("streichen" | "löschen" | "loeschen") => {
            for nr in numbers(rest, "streichen")? {
                println!("{}", ops::strike(store, nr)?);
            }
        }
        Some("abgeben") => {
            let (nr, person) = match rest {
                [nr, person] => (ops::parse_nr(nr)?, person.as_str()),
                [nr] => (ops::parse_nr(nr)?, ""),
                _ => return Err("So geht es: fokus abgeben 4 @tom".into()),
            };
            println!("{}", ops::delegate(store, nr, person)?);
        }
        Some("jetzt") => {
            let nr = rest.first().map(|s| ops::parse_nr(s)).transpose()?;
            focus::run(store, nr)?;
        }
        Some("alles") => print!("{}", view::matrix(store)),
        Some("themen" | "thema") => match rest.first().map(|s| s.to_lowercase()).as_deref() {
            Some("ordnen") if rest.len() > 1 => {
                println!("{}", ops::reorder(store, &rest[1..])?);
                print!("{}", view::topics(store));
            }
            Some("ordnen") => {
                println!("{}", ops::reorder_interactive(store)?);
                print!("{}", view::topics(store));
            }
            _ => print!("{}", view::topics(store)),
        },
        Some("personen" | "leute") => print!("{}", view::persons(store)),
        Some(p) if p.starts_with('@') => print!("{}", view::person(store, &args[0])),
        Some("aufräumen" | "aufraeumen") => review::run(store)?,
        Some("datei") => {
            println!(" Deine Aufgaben liegen in {}", store.path.display());
            if term::is_interactive() {
                store.save()?;
                let _ = std::process::Command::new("open")
                    .arg("-t")
                    .arg(&store.path)
                    .status();
            }
        }
        Some(other) => {
            return Err(format!(
                "„{other}“ kenne ich nicht. Alle Befehle: fokus hilfe\n Eine Aufgabe aufschreiben: fokus + {}",
                args.join(" ")
            ));
        }
    }
    Ok(())
}

fn numbers(args: &[String], verb: &str) -> Result<Vec<u32>, String> {
    if args.is_empty() {
        return Err(format!("Welche Aufgabe? Zum Beispiel: fokus {verb} 2"));
    }
    args.iter().map(|a| ops::parse_nr(a)).collect()
}

/// `4`, `4 7`, `4 statt 2`
fn numbers_with_instead(args: &[String]) -> Result<(Vec<u32>, Option<u32>), String> {
    match args.iter().position(|a| a.eq_ignore_ascii_case("statt")) {
        Some(p) => {
            let nrs = numbers(&args[..p], "heute")?;
            let instead = match &args[p + 1..] {
                [m] => ops::parse_nr(m)?,
                _ => return Err("So geht es: fokus heute 4 statt 2".into()),
            };
            if nrs.len() != 1 {
                return Err("Mit „statt“ geht nur eine Aufgabe auf einmal.".into());
            }
            Ok((nrs, Some(instead)))
        }
        None => Ok((numbers(args, "heute")?, None)),
    }
}
