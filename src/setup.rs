//! `fokus einrichten`: Kurzbefehl `f` und, wer mag, die Liste in jedem neuen Terminalfenster.
//! `fokus deinstallieren`: alles davon wieder weg, und auf Wunsch auch das Programm.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::store::{self, Store};
use crate::{away, config, date, term};

const BEGIN: &str = "# >>> fokus >>>";
const END: &str = "# <<< fokus <<<";

pub fn run() -> Result<(), String> {
    if !term::is_interactive() {
        return Err("Bitte direkt in einem Terminalfenster ausführen.".into());
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or("Ich finde dein Benutzerverzeichnis nicht.")?;
    let shell = std::env::var("SHELL").unwrap_or_default();
    let rc = if shell.ends_with("bash") {
        home.join(".bash_profile")
    } else {
        std::env::var_os("ZDOTDIR")
            .map(PathBuf::from)
            .unwrap_or(home)
            .join(".zshrc")
    };
    let exe = std::env::current_exe().map_err(|e| format!("Ich finde mich selbst nicht: {e}"))?;

    println!("\n {}\n", term::bold("Fokus einrichten"));
    println!(
        " {}\n",
        term::dim(&format!(
            "Dafür ändere ich nur die Datei {} und meine Einstellungen in {}.",
            rc.display(),
            config::path().display()
        ))
    );
    if exe.components().any(|c| c.as_os_str() == "target") {
        println!(
            " {}\n",
            term::yellow(
                "Hinweis: Du startest Fokus aus dem Bauordner. Installiere es lieber erst (siehe README)."
            )
        );
    }

    ask_places()?;

    let alias = term::ask_yes(" Soll „f“ als Kurzbefehl für „fokus“ funktionieren? [J/n] ")
        .map_err(term::io_err)?;
    let start =
        term::ask_yes(" Soll jedes neue Terminalfenster die Liste für heute zeigen? [J/n] ")
            .map_err(term::io_err)?;
    let complete =
        term::ask_yes(" Soll die Tab-Taste Befehle, Nummern, Themen und Personen ergänzen? [J/n] ")
            .map_err(term::io_err)?;

    let existing = fs::read_to_string(&rc).unwrap_or_default();
    let wishes = Wishes {
        alias,
        start,
        complete,
    };
    let updated = with_block(&existing, &block(&exe, &wishes, in_path(&exe)));
    if updated == existing {
        println!("\n Kurzbefehl und Tab-Taste waren schon so eingerichtet.");
        return Ok(());
    }
    fs::write(&rc, updated)
        .map_err(|e| format!("Ich kann {} nicht schreiben: {e}", rc.display()))?;
    println!(
        "\n {} Erledigt. Öffne ein neues Terminalfenster, dann ist es aktiv.",
        term::green("✓")
    );
    Ok(())
}

/// `fokus deinstallieren`: erst alles zeigen, dann einmal fragen, dann tun.
/// Die Aufgabendatei bleibt, außer du willst es ausdrücklich; dann kommt sie in
/// den Papierkorb, nicht ins Nichts.
pub fn uninstall() -> Result<(), String> {
    if !term::is_interactive() {
        return Err("Bitte direkt in einem Terminalfenster ausführen.".into());
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or("Ich finde dein Benutzerverzeichnis nicht.")?;
    let exe = std::env::current_exe().map_err(|e| format!("Ich finde mich selbst nicht: {e}"))?;
    let rcs: Vec<PathBuf> = shell_files(&home)
        .into_iter()
        .filter(|p| fs::read_to_string(p).is_ok_and(|s| s.lines().any(|l| l.trim() == BEGIN)))
        .collect();
    let program = Program::of(&exe, &home);
    let data = store::file_path();
    let away_file = away::path();
    let settings = config::path();
    let has_settings = store::play_file().is_none() && settings.exists();

    println!("\n {}\n", term::bold("Fokus deinstallieren"));
    println!(" Das würde ich tun:");
    for rc in &rcs {
        println!("   · den Fokus-Block aus {} entfernen", rc.display());
    }
    match &program {
        Program::Cargo(p) | Program::File(p) => {
            println!("   · das Programm {} löschen", p.display())
        }
        Program::DevBuild(p) => println!(
            "   · {}",
            term::dim(&format!(
                "das Programm bleibt: Es läuft aus dem Bauordner ({})",
                p.display()
            ))
        ),
    }
    if has_settings {
        println!("   · die Einstellungen {} löschen", settings.display());
    }
    if rcs.is_empty() && !has_settings && matches!(program, Program::DevBuild(_)) {
        println!("\n Es ist nichts installiert, das ich entfernen könnte.");
    } else {
        println!();
        if !term::ask_no(" Soll ich das tun? [j/N] ").map_err(term::io_err)? {
            println!("\n Nichts geändert.");
            return Ok(());
        }
    }

    // Die Aufgaben sind deine Daten: Standard ist behalten.
    let mut trash_data = false;
    if data.exists() {
        let open = Store::load(data.clone(), date::today())
            .map(|s| s.tasks.iter().filter(|t| t.is_active()).count())
            .unwrap_or(0);
        println!(
            "\n Deine Aufgaben liegen in {} ({open} offen).",
            data.display()
        );
        trash_data = term::ask_no(" Sollen sie in den Papierkorb? Enter behält sie. [j/N] ")
            .map_err(term::io_err)?;
    }

    println!();
    for rc in &rcs {
        let existing = fs::read_to_string(rc).unwrap_or_default();
        match fs::write(rc, with_block(&existing, "")) {
            Ok(()) => println!(" {} Block aus {} entfernt", term::green("✓"), rc.display()),
            Err(e) => println!(
                " {}",
                term::red(&format!("{} ließ sich nicht ändern: {e}", rc.display()))
            ),
        }
    }
    if has_settings {
        match fs::remove_file(&settings) {
            Ok(()) => {
                if let Some(dir) = settings.parent() {
                    let _ = fs::remove_dir(dir);
                }
                println!(" {} Einstellungen gelöscht", term::green("✓"));
            }
            Err(e) => println!(
                " {}",
                term::red(&format!(
                    "{} ließ sich nicht löschen: {e}",
                    settings.display()
                ))
            ),
        }
    }
    if trash_data {
        match move_to_trash(&data, &home) {
            Ok(to) => println!(
                " {} Aufgaben in den Papierkorb gelegt ({})",
                term::green("✓"),
                to.display()
            ),
            Err(e) => println!(
                " {}",
                term::red(&format!(
                    "Den Papierkorb erreiche ich nicht ({e}). Die Datei liegt noch in {}.",
                    data.display()
                ))
            ),
        }
    } else if data.exists() {
        println!(
            " {} Aufgaben bleiben in {}",
            term::green("✓"),
            data.display()
        );
    }
    if let Some(p) = away_file.filter(|p| p.exists()) {
        println!(
            " {}",
            term::dim(&format!(
                "Die Unterwegs-Datei bleibt in {}; löschen kannst du sie selbst.",
                p.display()
            ))
        );
    }
    match &program {
        Program::Cargo(p) => {
            let ok = Command::new("cargo")
                .args(["uninstall", "fokus"])
                .status()
                .is_ok_and(|s| s.success());
            if ok || fs::remove_file(p).is_ok() {
                println!(" {} Programm gelöscht", term::green("✓"));
            } else {
                println!(
                    " {}",
                    term::red(&format!("{} ließ sich nicht löschen.", p.display()))
                );
            }
        }
        Program::File(p) => match fs::remove_file(p) {
            Ok(()) => println!(" {} Programm gelöscht", term::green("✓")),
            Err(e) => println!(
                " {}",
                term::red(&format!("{} ließ sich nicht löschen: {e}", p.display()))
            ),
        },
        Program::DevBuild(_) => {}
    }
    println!(
        "\n {}",
        term::dim(
            "In schon offenen Terminalfenstern kennt die Shell „f“ noch, bis du sie schließt."
        )
    );
    Ok(())
}

/// `fokus datei verschieben <ort>`: Die Aufgabendatei zieht um, und Fokus merkt
/// sich den Ort. Eine Datei, die dort schon liegt, überschreibt Fokus nie.
pub fn move_data(input: &str) -> Result<String, String> {
    if let Some(p) = store::play_file() {
        return Err(format!(
            "Gerade gilt die Spieldatei FOKUS_DATEI={}. Ohne FOKUS_DATEI noch einmal versuchen.",
            p.display()
        ));
    }
    let typed = config::typed_path(input)
        .ok_or("Wohin? Zum Beispiel: fokus datei verschieben ~/Dokumente/Planung")?;
    let target = data_file_in(typed);
    let current = store::file_path();
    if away::same_file(&target, &current) {
        return Ok(format!(
            " Deine Aufgaben liegen schon in {}",
            current.display()
        ));
    }
    if away::path().is_some_and(|a| away::same_file(&a, &target)) {
        return Err(
            "Dort liegt die Unterwegs-Datei. Deine Aufgaben brauchen einen eigenen Ort.".into(),
        );
    }
    // Am Standardort braucht es keine Einstellung.
    let remember = |p: &Path| {
        let value = (!away::same_file(p, &store::default_path())).then_some(p);
        config::set(config::DATA_KEY, value)
    };

    if target.exists() {
        if !current.exists() {
            remember(&target)?;
            return Ok(format!(
                " {} Fokus nimmt jetzt die Aufgaben aus {}",
                term::green("✓"),
                target.display()
            ));
        }
        if !term::is_interactive() {
            return Err(format!(
                "In {} liegt schon eine Aufgabendatei. Die überschreibe ich nicht.",
                target.display()
            ));
        }
        let open = Store::load(target.clone(), date::today())
            .map(|s| s.tasks.iter().filter(|t| t.is_active()).count())
            .unwrap_or(0);
        println!(
            " In {} liegt schon eine Aufgabendatei ({open} offen).",
            target.display()
        );
        let take = term::ask_no(&format!(
            " Soll Fokus ab jetzt die nehmen? Deine bisherige bleibt unverändert in {}. [j/N] ",
            current.display()
        ))
        .map_err(term::io_err)?;
        if !take {
            return Ok(" Nichts geändert.".into());
        }
        remember(&target)?;
        return Ok(format!(
            " {} Fokus nimmt jetzt die Aufgaben aus {}",
            term::green("✓"),
            target.display()
        ));
    }

    if let Some(dir) = target.parent() {
        fs::create_dir_all(dir)
            .map_err(|e| format!("Ich kann den Ordner {} nicht anlegen: {e}", dir.display()))?;
    }
    // Erst merken, dann umziehen: Geht das Umziehen schief, kommt der alte Ort zurück.
    let before = config::get(config::DATA_KEY);
    remember(&target)?;
    if current.exists()
        && let Err(e) = move_file(&current, &target)
    {
        let _ = config::set(config::DATA_KEY, before.as_deref());
        return Err(format!(
            "Ich kann {} nicht nach {} bringen: {e}",
            current.display(),
            target.display()
        ));
    }
    Ok(format!(
        " {} Deine Aufgaben liegen jetzt in {}",
        term::green("✓"),
        target.display()
    ))
}

/// Ein Ordner bekommt `Fokus.md`, eine `.md`-Datei bleibt, wie sie ist.
fn data_file_in(p: PathBuf) -> PathBuf {
    let is_md = p.extension().is_some_and(|e| e.eq_ignore_ascii_case("md"));
    if is_md && !p.is_dir() {
        p
    } else {
        p.join("Fokus.md")
    }
}

/// Umbenennen; auf ein anderes Laufwerk kopieren, prüfen und erst dann löschen.
fn move_file(from: &Path, to: &Path) -> std::io::Result<()> {
    if fs::rename(from, to).is_ok() {
        return Ok(());
    }
    fs::copy(from, to)?;
    if fs::read(from)? != fs::read(to)? {
        let _ = fs::remove_file(to);
        return Err(std::io::Error::other(
            "die Kopie stimmt nicht mit dem Original überein",
        ));
    }
    fs::remove_file(from)
}

/// Die beiden Fragen nach den Orten in `fokus einrichten`.
fn ask_places() -> Result<(), String> {
    if let Some(p) = store::play_file() {
        println!(
            " {}\n",
            term::dim(&format!(
                "Gerade gilt die Spieldatei FOKUS_DATEI={}; die Orte lasse ich deshalb, wie sie sind.",
                p.display()
            ))
        );
        return Ok(());
    }
    println!(
        " Deine Aufgaben liegen in {}.",
        store::file_path().display()
    );
    let answer = term::read_line(" Anderer Ort? Ordner oder Datei, Enter behält ihn: ")
        .map_err(term::io_err)?
        .unwrap_or_default();
    if !answer.is_empty() {
        match move_data(&answer) {
            Ok(msg) => println!("{msg}"),
            Err(e) => println!(" {}", term::red(&e)),
        }
    }
    println!();

    let tip = format!(
        "{} Tipp: Ordner aus dem Finder hierher ziehen.",
        away::SYNC_HINT
    );
    let answer = match away::path() {
        Some(p) => {
            println!(" Die Unterwegs-Datei liegt in {}.", p.display());
            println!(" {}", term::dim(&tip));
            term::read_line(" Anderer Ordner, „aus“ schaltet sie ab, Enter behält sie: ")
                .map_err(term::io_err)?
                .unwrap_or_default()
        }
        None => {
            let want = term::ask_no(
                " Soll Fokus eine Unterwegs-Datei schreiben, die du am Handy liest und ergänzt? [j/N] ",
            )
            .map_err(term::io_err)?;
            if want {
                println!(" {}", term::dim(&tip));
                term::read_line(" Ordner: ")
                    .map_err(term::io_err)?
                    .unwrap_or_default()
            } else {
                String::new()
            }
        }
    };
    if !answer.is_empty() {
        match away::command(&[answer]) {
            Ok(msg) => println!("{msg}"),
            Err(e) => println!(" {}", term::red(&e)),
        }
    }
    // Gleich den ersten Stand schreiben, damit die Datei am Handy auftaucht.
    if let Some(p) = away::path()
        && let Ok(mut s) = Store::load(store::file_path(), date::today())
        && s.save().is_ok()
        && let Err(e) = away::write(&s, &p, &[])
    {
        println!(" {}", term::red(&e));
    }
    println!();
    Ok(())
}

/// Wo das Programm liegt, bestimmt, wie es weggeht.
enum Program {
    /// Mit `cargo install` installiert: `cargo uninstall` räumt auch dessen Liste auf.
    Cargo(PathBuf),
    File(PathBuf),
    /// Aus dem Bauordner gestartet: Das löscht man nicht von hier aus.
    DevBuild(PathBuf),
}

impl Program {
    fn of(exe: &Path, home: &Path) -> Program {
        let cargo_bin = std::env::var_os("CARGO_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".cargo"))
            .join("bin");
        if exe.components().any(|c| c.as_os_str() == "target") {
            Program::DevBuild(exe.to_path_buf())
        } else if exe.parent() == Some(cargo_bin.as_path()) {
            Program::Cargo(exe.to_path_buf())
        } else {
            Program::File(exe.to_path_buf())
        }
    }
}

/// Die Dateien, in die `fokus einrichten` geschrieben haben könnte.
fn shell_files(home: &Path) -> Vec<PathBuf> {
    let zdot = std::env::var_os("ZDOTDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.to_path_buf());
    let mut v = vec![zdot.join(".zshrc"), home.join(".bash_profile")];
    v.dedup();
    v
}

/// Verschiebt in `~/.Trash`; gibt es den Namen dort schon, mit Zahl dahinter.
fn move_to_trash(file: &Path, home: &Path) -> std::io::Result<PathBuf> {
    let trash = home.join(".Trash");
    let to = free_name(&trash, file);
    fs::rename(file, &to)?;
    Ok(to)
}

fn free_name(dir: &Path, file: &Path) -> PathBuf {
    let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("Fokus");
    let ext = file.extension().and_then(|s| s.to_str()).unwrap_or("md");
    let mut candidate = dir.join(format!("{stem}.{ext}"));
    let mut n = 2;
    while candidate.exists() {
        candidate = dir.join(format!("{stem} {n}.{ext}"));
        n += 1;
    }
    candidate
}

fn in_path(exe: &Path) -> bool {
    let Some(dir) = exe.parent() else {
        return false;
    };
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d == dir))
        .unwrap_or(false)
}

struct Wishes {
    alias: bool,
    start: bool,
    complete: bool,
}

/// Der Block für die Shell-Datei; leer, wenn nichts gewünscht ist.
fn block(exe: &Path, w: &Wishes, found: bool) -> String {
    let exe = format!("'{}'", exe.display().to_string().replace('\'', r"'\''"));
    let mut lines = Vec::new();
    if !found {
        lines.push(format!("alias fokus={exe}"));
    }
    if w.alias {
        lines.push(format!("alias f={exe}"));
    }
    if w.complete {
        lines.push(format!("eval \"$({exe} _zsh)\""));
    }
    if w.start {
        lines.push(exe.clone());
    }
    if lines.is_empty() {
        return String::new();
    }
    format!("{BEGIN}\n{}\n{END}\n", lines.join("\n"))
}

/// Ersetzt einen vorhandenen Fokus-Block oder hängt ihn an.
fn with_block(existing: &str, block: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    let mut inside = false;
    for line in existing.lines() {
        if line.trim() == BEGIN {
            inside = true;
            continue;
        }
        if line.trim() == END {
            inside = false;
            continue;
        }
        if !inside {
            kept.push(line);
        }
    }
    let mut out = kept.join("\n").trim_end().to_string();
    if !block.is_empty() {
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        out.push_str(block);
    } else if !out.is_empty() {
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn der_block_wird_ersetzt_nicht_verdoppelt() {
        let exe = Path::new("/usr/local/bin/fokus");
        let alle = Wishes {
            alias: true,
            start: true,
            complete: true,
        };
        let b = block(exe, &alle, true);
        let einmal = with_block("export X=1\n", &b);
        let zweimal = with_block(&einmal, &b);
        assert_eq!(einmal, zweimal);
        assert_eq!(
            einmal,
            "export X=1\n\n# >>> fokus >>>\nalias f='/usr/local/bin/fokus'\neval \"$('/usr/local/bin/fokus' _zsh)\"\n'/usr/local/bin/fokus'\n# <<< fokus <<<\n"
        );
        assert_eq!(with_block(&einmal, ""), "export X=1\n");
    }

    #[test]
    fn im_papierkorb_wird_nichts_ueberschrieben() {
        let dir = std::env::temp_dir().join(format!("fokus-papierkorb-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let datei = Path::new("/irgendwo/Fokus.md");
        assert_eq!(free_name(&dir, datei), dir.join("Fokus.md"));
        fs::write(dir.join("Fokus.md"), "alt").unwrap();
        assert_eq!(free_name(&dir, datei), dir.join("Fokus 2.md"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn programm_im_bauordner_wird_nicht_geloescht() {
        let home = Path::new("/Users/x");
        assert!(matches!(
            Program::of(Path::new("/p/target/release/fokus"), home),
            Program::DevBuild(_)
        ));
        assert!(matches!(
            Program::of(Path::new("/usr/local/bin/fokus"), home),
            Program::File(_)
        ));
    }

    #[test]
    fn ohne_pfad_gibt_es_auch_fokus_als_alias() {
        let nichts = Wishes {
            alias: false,
            start: false,
            complete: false,
        };
        let b = block(Path::new("/a/fokus"), &nichts, false);
        assert!(b.contains("alias fokus='/a/fokus'"));
    }
}
