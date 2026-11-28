//! `focus einrichten`: Kurzbefehl `f` und, wer mag, die Liste in jedem neuen Terminalfenster.
//! `focus deinstallieren`: alles davon wieder weg, und auf Wunsch auch das Programm.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::i18n::{self, Lang};
use crate::store::{self, Store};
use crate::{away, config, date, term, tr, trs};

const BEGIN: &str = "# >>> focus >>>";
const END: &str = "# <<< focus <<<";
/// So hieß der Block, als Focus noch „fokus“ hieß. Er wird ersetzt.
const OLD_BEGIN: &str = "# >>> fokus >>>";
const OLD_END: &str = "# <<< fokus <<<";

fn is_begin(line: &str) -> bool {
    matches!(line.trim(), BEGIN | OLD_BEGIN)
}

fn is_end(line: &str) -> bool {
    matches!(line.trim(), END | OLD_END)
}

pub fn run() -> Result<(), String> {
    if !term::is_interactive() {
        return Err(not_interactive());
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(no_home)?;
    let shell = std::env::var("SHELL").unwrap_or_default();
    let zsh = !shell.ends_with("bash");
    let rc = if !zsh {
        bash_rc(&home)
    } else {
        std::env::var_os("ZDOTDIR")
            .map(PathBuf::from)
            .unwrap_or(home)
            .join(".zshrc")
    };
    let exe = std::env::current_exe().map_err(|e| self_not_found(&e))?;

    ask_language()?;

    println!(
        "\n {}\n",
        term::bold(trs!("Focus einrichten", "Set up Focus"))
    );
    println!(
        " {}\n",
        term::dim(&tr!(
            "Dafür ändere ich nur die Datei {} und meine Einstellungen in {}.",
            "For this I only change the file {} and my settings in {}.",
            rc.display(),
            config::path().display()
        ))
    );
    if exe.components().any(|c| c.as_os_str() == "target") {
        println!(
            " {}\n",
            term::yellow(trs!(
                "Hinweis: Du startest Focus aus dem Bauordner. Installiere es lieber erst (siehe README).",
                "Note: you're running Focus from the build folder. Better install it first (see README)."
            ))
        );
    }

    ask_places()?;

    let alias = term::ask_yes(trs!(
        " Soll „f“ als Kurzbefehl für „focus“ funktionieren? [J/n] ",
        " Should “f” work as a shortcut for “focus”? [Y/n] "
    ))
    .map_err(term::io_err)?;
    let start = term::ask_yes(trs!(
        " Soll jedes neue Terminalfenster die Liste für heute zeigen? [J/n] ",
        " Should every new terminal window show today's list? [Y/n] "
    ))
    .map_err(term::io_err)?;
    // Die Tab-Ergänzung gibt es nur für zsh.
    let complete = zsh
        && term::ask_yes(trs!(
            " Soll die Tab-Taste Befehle, Nummern, Themen und Personen ergänzen? [J/n] ",
            " Should the Tab key complete commands, numbers, topics and people? [Y/n] "
        ))
        .map_err(term::io_err)?;
    ask_clear()?;

    let existing = fs::read_to_string(&rc).unwrap_or_default();
    let wishes = Wishes {
        alias,
        start,
        complete,
        zsh,
    };
    let updated = with_block(&existing, &block(&exe, &wishes, in_path(&exe)));
    if updated == existing {
        println!("\n {} {}", term::green("✓"), trs!("Erledigt.", "Done."));
        return Ok(());
    }
    fs::write(&rc, updated).map_err(|e| {
        tr!(
            "Ich kann {} nicht schreiben: {e}",
            "I can't write {}: {e}",
            rc.display()
        )
    })?;
    println!(
        "\n {} {}",
        term::green("✓"),
        trs!(
            "Erledigt. Öffne ein neues Terminalfenster, dann ist es aktiv.",
            "Done. Open a new terminal window and it's active."
        )
    );
    Ok(())
}

fn not_interactive() -> String {
    tr!(
        "Bitte direkt in einem Terminalfenster ausführen.",
        "Please run this directly in a terminal window."
    )
}

fn no_home() -> String {
    tr!(
        "Ich finde dein Benutzerverzeichnis nicht.",
        "I can't find your home folder."
    )
}

fn self_not_found(e: &std::io::Error) -> String {
    tr!(
        "Ich finde mich selbst nicht: {e}",
        "I can't find my own program: {e}"
    )
}

/// Die erste Frage, in beiden Sprachen: Ab der Antwort spricht Focus so.
fn ask_language() -> Result<(), String> {
    let now = i18n::lang();
    println!(
        "\n {}",
        term::bold("Language / Sprache: [e] English · [d] Deutsch")
    );
    print!(
        " {}",
        term::dim(match now {
            Lang::En => "Enter keeps English / Enter behält Englisch ",
            Lang::De => "Enter keeps German / Enter behält Deutsch ",
        })
    );
    let chosen = loop {
        match term::read_key().map_err(term::io_err)? {
            'e' => break Lang::En,
            'd' | 'g' => break Lang::De,
            '\n' | '\x1b' => break now,
            _ => {}
        }
    };
    println!(
        "{}",
        if chosen == Lang::En {
            "English"
        } else {
            "Deutsch"
        }
    );
    i18n::set(chosen);
    config::set_str(i18n::KEY, Some(i18n::code(chosen)))
}

/// `focus deinstallieren`: erst alles zeigen, dann einmal fragen, dann tun.
/// Die Aufgabendatei bleibt, außer du willst es ausdrücklich; dann kommt sie in
/// den Papierkorb, nicht ins Nichts.
pub fn uninstall() -> Result<(), String> {
    if !term::is_interactive() {
        return Err(not_interactive());
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(no_home)?;
    let exe = std::env::current_exe().map_err(|e| self_not_found(&e))?;
    let rcs: Vec<PathBuf> = shell_files(&home)
        .into_iter()
        .filter(|p| fs::read_to_string(p).is_ok_and(|s| s.lines().any(is_begin)))
        .collect();
    let program = Program::of(&exe, &home);
    let data = store::file_path();
    let away_file = away::path();
    let settings = config::path();
    let has_settings = store::play_file().is_none() && settings.exists();

    println!(
        "\n {}\n",
        term::bold(trs!("Focus deinstallieren", "Uninstall Focus"))
    );
    println!("{}", trs!(" Das würde ich tun:", " Here's what I'd do:"));
    for rc in &rcs {
        println!(
            "{}",
            tr!(
                "   · den Focus-Block aus {} entfernen",
                "   · remove the Focus block from {}",
                rc.display()
            )
        );
    }
    match &program {
        Program::Cargo(p) | Program::File(p) => println!(
            "{}",
            tr!(
                "   · das Programm {} löschen",
                "   · delete the program {}",
                p.display()
            )
        ),
        Program::DevBuild(p) => println!(
            "   · {}",
            term::dim(&tr!(
                "das Programm bleibt: Es läuft aus dem Bauordner ({})",
                "the program stays: it runs from the build folder ({})",
                p.display()
            ))
        ),
    }
    if has_settings {
        println!(
            "{}",
            tr!(
                "   · die Einstellungen {} löschen",
                "   · delete the settings {}",
                settings.display()
            )
        );
    }
    if rcs.is_empty() && !has_settings && matches!(program, Program::DevBuild(_)) {
        println!(
            "\n{}",
            trs!(
                " Es ist nichts installiert, das ich entfernen könnte.",
                " Nothing is installed that I could remove."
            )
        );
    } else {
        println!();
        if !term::ask_no(trs!(
            " Soll ich das tun? [j/N] ",
            " Should I do that? [y/N] "
        ))
        .map_err(term::io_err)?
        {
            println!("\n{}", trs!(" Nichts geändert.", " Nothing changed."));
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
            "\n{}",
            tr!(
                " Deine Aufgaben liegen in {} ({open} offen).",
                " Your tasks are in {} ({open} open).",
                data.display()
            )
        );
        trash_data = term::ask_no(trs!(
            " Sollen sie in den Papierkorb? Enter behält sie. [j/N] ",
            " Move them to the trash? Enter keeps them. [y/N] "
        ))
        .map_err(term::io_err)?;
    }

    println!();
    for rc in &rcs {
        let existing = fs::read_to_string(rc).unwrap_or_default();
        match fs::write(rc, with_block(&existing, "")) {
            Ok(()) => println!(
                " {} {}",
                term::green("✓"),
                tr!(
                    "Block aus {} entfernt",
                    "Removed the block from {}",
                    rc.display()
                )
            ),
            Err(e) => println!(
                " {}",
                term::red(&tr!(
                    "{} ließ sich nicht ändern: {e}",
                    "Couldn't change {}: {e}",
                    rc.display()
                ))
            ),
        }
    }
    if has_settings {
        match fs::remove_file(&settings) {
            Ok(()) => {
                if let Some(dir) = settings.parent() {
                    let _ = fs::remove_dir(dir);
                }
                println!(
                    " {} {}",
                    term::green("✓"),
                    trs!("Einstellungen gelöscht", "Settings deleted")
                );
            }
            Err(e) => println!(" {}", term::red(&cant_delete(&settings, Some(&e)))),
        }
    }
    if trash_data {
        match move_to_trash(&data, &home) {
            Ok(to) => println!(
                " {} {}",
                term::green("✓"),
                tr!(
                    "Aufgaben in den Papierkorb gelegt ({})",
                    "Moved tasks to the trash ({})",
                    to.display()
                )
            ),
            Err(e) => println!(
                " {}",
                term::red(&tr!(
                    "Den Papierkorb erreiche ich nicht ({e}). Die Datei liegt noch in {}.",
                    "I can't reach the trash ({e}). The file is still in {}.",
                    data.display()
                ))
            ),
        }
    } else if data.exists() {
        println!(
            " {} {}",
            term::green("✓"),
            tr!("Aufgaben bleiben in {}", "Tasks stay in {}", data.display())
        );
    }
    if let Some(p) = away_file.filter(|p| p.exists()) {
        println!(
            " {}",
            term::dim(&tr!(
                "Die Unterwegs-Datei bleibt in {}; löschen kannst du sie selbst.",
                "The mobile file stays in {}; you can delete it yourself.",
                p.display()
            ))
        );
    }
    match &program {
        Program::Cargo(p) => {
            let ok = Command::new("cargo")
                .args(["uninstall", "focus"])
                .status()
                .is_ok_and(|s| s.success());
            if ok || fs::remove_file(p).is_ok() {
                println!(
                    " {} {}",
                    term::green("✓"),
                    trs!("Programm gelöscht", "Program deleted")
                );
            } else {
                println!(" {}", term::red(&cant_delete(p, None)));
            }
        }
        Program::File(p) => match fs::remove_file(p) {
            Ok(()) => println!(
                " {} {}",
                term::green("✓"),
                trs!("Programm gelöscht", "Program deleted")
            ),
            Err(e) => println!(" {}", term::red(&cant_delete(p, Some(&e)))),
        },
        Program::DevBuild(_) => {}
    }
    println!(
        "\n {}",
        term::dim(trs!(
            "In schon offenen Terminalfenstern kennt die Shell „f“ noch, bis du sie schließt.",
            "Terminal windows that are already open still know “f” until you close them."
        ))
    );
    Ok(())
}

fn cant_delete(p: &Path, e: Option<&std::io::Error>) -> String {
    match e {
        Some(e) => tr!(
            "{} ließ sich nicht löschen: {e}",
            "Couldn't delete {}: {e}",
            p.display()
        ),
        None => tr!(
            "{} ließ sich nicht löschen.",
            "Couldn't delete {}.",
            p.display()
        ),
    }
}

/// `focus datei verschieben <ort>`: Die Aufgabendatei zieht um, und Focus merkt
/// sich den Ort. Eine Datei, die dort schon liegt, überschreibt Focus nie.
pub fn move_data(input: &str) -> Result<String, String> {
    if let Some(p) = store::play_file() {
        return Err(tr!(
            "Gerade gilt die Spieldatei FOCUS_DATEI={}. Ohne FOCUS_DATEI noch einmal versuchen.",
            "The practice file FOCUS_DATEI={} is in effect. Try again without FOCUS_DATEI.",
            p.display()
        ));
    }
    let typed = config::typed_path(input).ok_or_else(|| {
        tr!(
            "Wohin? Zum Beispiel: focus datei verschieben ~/Dokumente/Planung",
            "Where to? For example: focus file move ~/Documents/Planning"
        )
    })?;
    let target = data_file_in(typed);
    let current = store::file_path();
    if away::same_file(&target, &current) {
        return Ok(tr!(
            " Deine Aufgaben liegen schon in {}",
            " Your tasks are already in {}",
            current.display()
        ));
    }
    if away::path().is_some_and(|a| away::same_file(&a, &target)) {
        return Err(tr!(
            "Dort liegt die Unterwegs-Datei. Deine Aufgaben brauchen einen eigenen Ort.",
            "That's where the mobile file is. Your tasks need a place of their own."
        ));
    }
    // Am Standardort braucht es keine Einstellung.
    let remember = |p: &Path| {
        let value = (!away::same_file(p, &store::default_path())).then_some(p);
        config::set(config::DATA_KEY, value)
    };

    if target.exists() {
        if !current.exists() {
            remember(&target)?;
            return Ok(takes_from(&target));
        }
        if !term::is_interactive() {
            return Err(tr!(
                "In {} liegt schon eine Aufgabendatei. Die überschreibe ich nicht.",
                "There's already a task file in {}. I won't overwrite it.",
                target.display()
            ));
        }
        let open = Store::load(target.clone(), date::today())
            .map(|s| s.tasks.iter().filter(|t| t.is_active()).count())
            .unwrap_or(0);
        println!(
            "{}",
            tr!(
                " In {} liegt schon eine Aufgabendatei ({open} offen).",
                " There's already a task file in {} ({open} open).",
                target.display()
            )
        );
        let take = term::ask_no(&tr!(
            " Soll Focus ab jetzt die nehmen? Deine bisherige bleibt unverändert in {}. [j/N] ",
            " Should Focus use that one from now on? Your current one stays unchanged in {}. [y/N] ",
            current.display()
        ))
        .map_err(term::io_err)?;
        if !take {
            return Ok(tr!(" Nichts geändert.", " Nothing changed."));
        }
        remember(&target)?;
        return Ok(takes_from(&target));
    }

    if let Some(dir) = target.parent() {
        fs::create_dir_all(dir).map_err(|e| cant_create(dir, &e))?;
    }
    // Erst merken, dann umziehen: Geht das Umziehen schief, kommt der alte Ort zurück.
    let before = config::get(config::DATA_KEY);
    remember(&target)?;
    if current.exists()
        && let Err(e) = move_file(&current, &target)
    {
        let _ = config::set(config::DATA_KEY, before.as_deref());
        return Err(tr!(
            "Ich kann {} nicht nach {} bringen: {e}",
            "I can't move {} to {}: {e}",
            current.display(),
            target.display()
        ));
    }
    Ok(tr!(
        " {} Deine Aufgaben liegen jetzt in {}",
        " {} Your tasks are now in {}",
        term::green("✓"),
        target.display()
    ))
}

fn takes_from(target: &Path) -> String {
    tr!(
        " {} Focus nimmt jetzt die Aufgaben aus {}",
        " {} Focus now uses the tasks from {}",
        term::green("✓"),
        target.display()
    )
}

pub fn cant_create(dir: &Path, e: &std::io::Error) -> String {
    tr!(
        "Ich kann den Ordner {} nicht anlegen: {e}",
        "I can't create the folder {}: {e}",
        dir.display()
    )
}

/// Ein Ordner bekommt `Focus.md`, eine `.md`-Datei bleibt, wie sie ist.
fn data_file_in(p: PathBuf) -> PathBuf {
    let is_md = p.extension().is_some_and(|e| e.eq_ignore_ascii_case("md"));
    if is_md && !p.is_dir() {
        p
    } else {
        p.join("Focus.md")
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
        return Err(std::io::Error::other(trs!(
            "die Kopie stimmt nicht mit dem Original überein",
            "the copy doesn't match the original"
        )));
    }
    fs::remove_file(from)
}

/// Die beiden Fragen nach den Orten in `focus einrichten`.
/// Fenster leeren vor jeder Ausgabe: gut, wenn ein Terminalfenster nur für Focus da ist.
fn ask_clear() -> Result<(), String> {
    println!(
        " {}",
        term::dim(trs!(
            "Nutzt du ein Fenster nur für Focus? Dann kann Focus es vor jedem Befehl leeren; oben steht der Befehl, Hochscrollen zeigt das Alte.",
            "Do you keep a window just for Focus? Then Focus can clear it before each command; the command stays on top, scrolling up shows the old output."
        ))
    );
    let before = config::flag(config::CLEAR_KEY);
    let on = if before {
        term::ask_yes(trs!(
            " Soll Focus das Fenster vor jeder Ausgabe leeren? [J/n] ",
            " Should Focus clear the window before each output? [Y/n] "
        ))
    } else {
        term::ask_no(trs!(
            " Soll Focus das Fenster vor jeder Ausgabe leeren? [j/N] ",
            " Should Focus clear the window before each output? [y/N] "
        ))
    }
    .map_err(term::io_err)?;
    if on != before {
        config::set_flag(config::CLEAR_KEY, on)?;
    }
    Ok(())
}

fn ask_places() -> Result<(), String> {
    if let Some(p) = store::play_file() {
        println!(
            " {}\n",
            term::dim(&tr!(
                "Gerade gilt die Spieldatei FOCUS_DATEI={}; die Orte lasse ich deshalb, wie sie sind.",
                "The practice file FOCUS_DATEI={} is in effect, so I'm leaving the places as they are.",
                p.display()
            ))
        );
        return Ok(());
    }
    println!(
        "{}",
        tr!(
            " Deine Aufgaben liegen in {}.",
            " Your tasks are in {}.",
            store::file_path().display()
        )
    );
    let answer = term::read_line(trs!(
        " Anderer Ort? Ordner oder Datei, Enter behält ihn: ",
        " Somewhere else? Folder or file, Enter keeps it: "
    ))
    .map_err(term::io_err)?
    .unwrap_or_default();
    if !answer.is_empty() {
        match move_data(&answer) {
            Ok(msg) => println!("{msg}"),
            Err(e) => println!(" {}", term::red(&e)),
        }
    }
    println!();

    let tip = tr!(
        "{} Tipp: Ordner aus dem Finder hierher ziehen.",
        "{} Tip: drag the folder from Finder in here.",
        away::sync_hint()
    );
    let answer = match away::path() {
        Some(p) => {
            println!(
                "{}",
                tr!(
                    " Die Unterwegs-Datei liegt in {}.",
                    " The mobile file is in {}.",
                    p.display()
                )
            );
            println!(" {}", term::dim(&tip));
            term::read_line(trs!(
                " Anderer Ordner, „aus“ schaltet sie ab, Enter behält sie: ",
                " Another folder, “off” turns it off, Enter keeps it: "
            ))
            .map_err(term::io_err)?
            .unwrap_or_default()
        }
        None => {
            let want = term::ask_no(trs!(
                " Soll Focus eine Unterwegs-Datei schreiben, die du am Handy liest und ergänzt? [j/N] ",
                " Should Focus write a mobile file you can read and add to on your phone? [y/N] "
            ))
            .map_err(term::io_err)?;
            if want {
                println!(" {}", term::dim(&tip));
                term::read_line(trs!(" Ordner: ", " Folder: "))
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

/// Die Dateien, in die `focus einrichten` geschrieben haben könnte.
fn shell_files(home: &Path) -> Vec<PathBuf> {
    let zdot = std::env::var_os("ZDOTDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.to_path_buf());
    let mut v = vec![
        zdot.join(".zshrc"),
        home.join(".bash_profile"),
        home.join(".bashrc"),
    ];
    v.dedup();
    v
}

/// Wo bash ihre Einstellungen liest: Terminal.app startet Login-Shells
/// (`.bash_profile`), Linux-Terminals meist nicht (`.bashrc`).
fn bash_rc(home: &Path) -> PathBuf {
    if cfg!(target_os = "macos") {
        home.join(".bash_profile")
    } else {
        home.join(".bashrc")
    }
}

/// Verschiebt in den Papierkorb; gibt es den Namen dort schon, mit Zahl dahinter.
/// Am Mac `~/.Trash`, sonst der Papierkorb nach freedesktop.org, den Dateimanager
/// wie Nautilus oder Dolphin zeigen und aus dem sie wiederherstellen können.
fn move_to_trash(file: &Path, home: &Path) -> std::io::Result<PathBuf> {
    if cfg!(target_os = "macos") {
        let to = free_name(&home.join(".Trash"), file);
        move_file(file, &to)?;
        return Ok(to);
    }
    let trash = std::env::var_os("XDG_DATA_HOME")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local").join("share"))
        .join("Trash");
    let files = trash.join("files");
    let info = trash.join("info");
    fs::create_dir_all(&files)?;
    fs::create_dir_all(&info)?;
    let to = free_name(&files, file);
    let name = to
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Focus.md");
    let from = fs::canonicalize(file).unwrap_or_else(|_| file.to_path_buf());
    fs::write(
        info.join(format!("{name}.trashinfo")),
        format!(
            "[Trash Info]\nPath={}\nDeletionDate={}\n",
            from.display(),
            chrono::Local::now().format("%Y-%m-%dT%H:%M:%S")
        ),
    )?;
    move_file(file, &to)?;
    Ok(to)
}

fn free_name(dir: &Path, file: &Path) -> PathBuf {
    let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("Focus");
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
    /// zsh bricht bei `?`/`*` ohne Treffer ab, bevor Focus startet (z. B. bei
    /// Links mit `?msg=…`). `noglob` im Alias reicht die Wörter unverändert durch.
    zsh: bool,
}

/// Der Block für die Shell-Datei; leer, wenn nichts gewünscht ist.
fn block(exe: &Path, w: &Wishes, found: bool) -> String {
    let exe = sq(&exe.display().to_string());
    let mut lines = Vec::new();
    if w.zsh {
        // Eine Funktion statt des Pfads im Alias: Die Tab-Taste sieht dann den
        // Befehl „focus“ und nicht einen Pfad samt Anführungszeichen. `function`
        // davor, damit ein schon gesetzter Alias beim erneuten Laden nicht stört.
        lines.push(format!("function focus {{ {exe} \"$@\"; }}"));
        lines.push("alias focus='noglob focus'".to_string());
        if w.alias {
            lines.push("alias f='noglob focus'".to_string());
        }
    } else {
        if !found {
            lines.push(format!("alias focus={exe}"));
        }
        if w.alias {
            lines.push(format!("alias f={exe}"));
        }
    }
    if w.complete && w.zsh {
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

/// Setzt `s` in einfache Anführungszeichen, sicher für die Shell.
fn sq(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// Ersetzt einen vorhandenen Focus-Block oder hängt ihn an.
fn with_block(existing: &str, block: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    let mut inside = false;
    for line in existing.lines() {
        if is_begin(line) {
            inside = true;
            continue;
        }
        if is_end(line) {
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
        let exe = Path::new("/usr/local/bin/focus");
        let alle = Wishes {
            alias: true,
            start: true,
            complete: true,
            zsh: false,
        };
        let b = block(exe, &alle, true);
        let einmal = with_block("export X=1\n", &b);
        let zweimal = with_block(&einmal, &b);
        assert_eq!(einmal, zweimal);
        assert_eq!(
            einmal,
            "export X=1\n\n# >>> focus >>>\nalias f='/usr/local/bin/focus'\n'/usr/local/bin/focus'\n# <<< focus <<<\n"
        );
        // In bash gibt es keine Tab-Ergänzung von Focus: das zsh-Skript bliebe dort nur ein Fehler.
        assert!(!b.contains("_zsh"));
        assert_eq!(with_block(&einmal, ""), "export X=1\n");
        // Der Block aus der Zeit, als Focus „fokus“ hieß, wird ersetzt, nicht verdoppelt.
        let alt =
            "export X=1\n\n# >>> fokus >>>\nalias f='/usr/local/bin/fokus'\n# <<< fokus <<<\n";
        assert_eq!(with_block(alt, &b), einmal);
    }

    #[test]
    fn im_papierkorb_wird_nichts_ueberschrieben() {
        let dir = std::env::temp_dir().join(format!("focus-papierkorb-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let datei = Path::new("/irgendwo/Focus.md");
        assert_eq!(free_name(&dir, datei), dir.join("Focus.md"));
        fs::write(dir.join("Focus.md"), "alt").unwrap();
        assert_eq!(free_name(&dir, datei), dir.join("Focus 2.md"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn programm_im_bauordner_wird_nicht_geloescht() {
        let home = Path::new("/Users/x");
        assert!(matches!(
            Program::of(Path::new("/p/target/release/focus"), home),
            Program::DevBuild(_)
        ));
        assert!(matches!(
            Program::of(Path::new("/usr/local/bin/focus"), home),
            Program::File(_)
        ));
    }

    #[test]
    fn ohne_pfad_gibt_es_auch_focus_als_alias() {
        let nichts = Wishes {
            alias: false,
            start: false,
            complete: false,
            zsh: false,
        };
        let b = block(Path::new("/a/focus"), &nichts, false);
        assert!(b.contains("alias focus='/a/focus'"));
    }

    #[test]
    fn in_zsh_stoeren_fragezeichen_in_links_nicht() {
        let w = Wishes {
            alias: true,
            start: false,
            complete: false,
            zsh: true,
        };
        let b = block(Path::new("/Users/x y/focus"), &w, true);
        assert!(b.contains(r#"function focus { '/Users/x y/focus' "$@"; }"#));
        assert!(b.contains("alias focus='noglob focus'"));
        assert!(b.contains("alias f='noglob focus'"));
    }
}
