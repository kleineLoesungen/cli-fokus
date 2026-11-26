//! Terminal: Farben, Breite, eine Taste oder eine Zeile lesen.

use std::io::{self, IsTerminal, Write};
use std::sync::OnceLock;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

static COLOR: OnceLock<bool> = OnceLock::new();

/// Farbe nur im Terminal, und nie, wenn `NO_COLOR` gesetzt ist.
pub fn color() -> bool {
    *COLOR.get_or_init(|| std::env::var_os("NO_COLOR").is_none() && io::stdout().is_terminal())
}

fn paint(code: &str, s: &str) -> String {
    if color() && !s.is_empty() {
        format!("\x1b[{code}m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

pub fn bold(s: &str) -> String {
    paint("1", s)
}
pub fn dim(s: &str) -> String {
    paint("2", s)
}
pub fn red(s: &str) -> String {
    paint("31", s)
}
pub fn yellow(s: &str) -> String {
    paint("33", s)
}
pub fn green(s: &str) -> String {
    paint("32", s)
}
pub fn cyan(s: &str) -> String {
    paint("36", s)
}

/// Breite für die Ansichten. Ohne Terminal (in eine Datei umgeleitet) 80.
pub fn width() -> usize {
    let w = if io::stdout().is_terminal() {
        crossterm::terminal::size().ok().map(|(w, _)| w as usize)
    } else {
        None
    };
    w.or_else(|| std::env::var("COLUMNS").ok()?.parse().ok())
        .unwrap_or(80)
        .clamp(40, 100)
}

pub fn is_interactive() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

pub fn text_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Kürzt auf `max` Spalten, mit „…“ am Ende.
pub fn truncate(s: &str, max: usize) -> String {
    if text_width(s) <= max {
        return s.to_string();
    }
    if max == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut w = 0;
    for c in s.chars() {
        let cw = c.width().unwrap_or(0);
        if w + cw > max - 1 {
            break;
        }
        out.push(c);
        w += cw;
    }
    out.push('…');
    out
}

pub fn pad_right(s: &str, w: usize) -> String {
    let tw = text_width(s);
    if tw >= w {
        s.to_string()
    } else {
        format!("{s}{}", " ".repeat(w - tw))
    }
}

pub fn pad_left(s: &str, w: usize) -> String {
    let tw = text_width(s);
    if tw >= w {
        s.to_string()
    } else {
        format!("{}{s}", " ".repeat(w - tw))
    }
}

/// Genau `w` Spalten: gekürzt oder aufgefüllt.
pub fn fit(s: &str, w: usize) -> String {
    pad_right(&truncate(s, w), w)
}

pub fn center(s: &str, w: usize) -> String {
    let tw = text_width(s);
    if tw >= w {
        return truncate(s, w);
    }
    let left = (w - tw) / 2;
    format!("{}{s}{}", " ".repeat(left), " ".repeat(w - tw - left))
}

/// Eine Taste, klein geschrieben. Enter ist `'\n'`, Esc und Strg-C sind `'\x1b'`.
pub fn read_key() -> io::Result<char> {
    io::stdout().flush()?;
    crossterm::terminal::enable_raw_mode()?;
    let key = loop {
        match event::read() {
            Ok(Event::Key(k)) if k.kind == KeyEventKind::Press => {
                if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('c') {
                    break Ok('\x1b');
                }
                match k.code {
                    KeyCode::Char(c) => break Ok(c.to_lowercase().next().unwrap_or(c)),
                    KeyCode::Enter => break Ok('\n'),
                    KeyCode::Esc => break Ok('\x1b'),
                    _ => {}
                }
            }
            Ok(_) => {}
            Err(e) => break Err(e),
        }
    };
    crossterm::terminal::disable_raw_mode()?;
    key
}

/// Eine Zeile, ohne Leerraum an den Rändern. `None`, wenn die Eingabe zu ist.
pub fn read_line(prompt: &str) -> io::Result<Option<String>> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut s = String::new();
    if io::stdin().read_line(&mut s)? == 0 {
        return Ok(None);
    }
    Ok(Some(s.trim().to_string()))
}

/// Ja oder nein, Enter heißt ja.
pub fn ask_yes(prompt: &str) -> io::Result<bool> {
    print!("{prompt}");
    loop {
        match read_key()? {
            'j' | 'y' | '\n' => {
                println!("ja");
                return Ok(true);
            }
            'n' | '\x1b' => {
                println!("nein");
                return Ok(false);
            }
            _ => {}
        }
    }
}

pub fn io_err(e: io::Error) -> String {
    format!("Das Terminal hat nicht mitgespielt: {e}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kuerzen_beachtet_umlaute() {
        assert_eq!(truncate("Präsentation", 6), "Präse…");
        assert_eq!(fit("Haus", 6), "Haus  ");
        assert_eq!(pad_left("3", 3), "  3");
        assert_eq!(center("ab", 6), "  ab  ");
    }
}
