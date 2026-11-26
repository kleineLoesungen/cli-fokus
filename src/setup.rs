//! `fokus einrichten`: Kurzbefehl `f` und, wer mag, die Liste in jedem neuen Terminalfenster.

use std::fs;
use std::path::{Path, PathBuf};

use crate::term;

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
        term::dim(&format!("Dafür ändere ich nur die Datei {}.", rc.display()))
    );
    if exe.components().any(|c| c.as_os_str() == "target") {
        println!(
            " {}\n",
            term::yellow(
                "Hinweis: Du startest Fokus aus dem Bauordner. Installiere es lieber erst (siehe README)."
            )
        );
    }

    let alias = term::ask_yes(" Soll „f“ als Kurzbefehl für „fokus“ funktionieren? [J/n] ")
        .map_err(term::io_err)?;
    let start =
        term::ask_yes(" Soll jedes neue Terminalfenster die Liste für heute zeigen? [J/n] ")
            .map_err(term::io_err)?;

    let existing = fs::read_to_string(&rc).unwrap_or_default();
    let updated = with_block(&existing, &block(&exe, alias, start, in_path(&exe)));
    if updated == existing {
        println!("\n Schon alles so eingerichtet.");
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

fn in_path(exe: &Path) -> bool {
    let Some(dir) = exe.parent() else {
        return false;
    };
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d == dir))
        .unwrap_or(false)
}

/// Der Block für die Shell-Datei; leer, wenn nichts gewünscht ist.
fn block(exe: &Path, alias: bool, start: bool, found: bool) -> String {
    let exe = format!("'{}'", exe.display().to_string().replace('\'', r"'\''"));
    let mut lines = Vec::new();
    if !found {
        lines.push(format!("alias fokus={exe}"));
    }
    if alias {
        lines.push(format!("alias f={exe}"));
    }
    if start {
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
        let b = block(exe, true, true, true);
        let einmal = with_block("export X=1\n", &b);
        let zweimal = with_block(&einmal, &b);
        assert_eq!(einmal, zweimal);
        assert_eq!(
            einmal,
            "export X=1\n\n# >>> fokus >>>\nalias f='/usr/local/bin/fokus'\n'/usr/local/bin/fokus'\n# <<< fokus <<<\n"
        );
        assert_eq!(with_block(&einmal, ""), "export X=1\n");
    }

    #[test]
    fn ohne_pfad_gibt_es_auch_fokus_als_alias() {
        let b = block(Path::new("/a/fokus"), false, false, false);
        assert!(b.contains("alias fokus='/a/fokus'"));
    }
}
