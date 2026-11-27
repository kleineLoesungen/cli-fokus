//! `fokus anleitung`: das README, ins Programm eingebaut. So passt die
//! Anleitung immer zur installierten Version und geht auch ohne Netz.

use std::io::Write;
use std::process::{Command, Stdio};

use crate::term;

const README: &str = include_str!("../README.md");

/// `fokus anleitung` zeigt alles, `fokus anleitung unterwegs` nur, was passt.
pub fn run(args: &[String]) -> Result<(), String> {
    let query = args.join(" ");
    let query = query.trim();
    let text = if query.is_empty() {
        render(README)
    } else {
        let sections = find(README, query);
        if sections.is_empty() {
            let mut msg = format!(
                " Zu „{query}“ steht nichts in der Anleitung. Diese Abschnitte gibt es:\n\n"
            );
            for (level, title) in headings(README) {
                let indent = if level <= 2 { "   " } else { "     " };
                msg.push_str(&format!("{indent}{title}\n"));
            }
            msg.push_str(&format!(
                "\n {}\n",
                term::dim("Alles lesen: fokus anleitung")
            ));
            print!("{msg}");
            return Ok(());
        }
        render(&sections.join("\n"))
    };
    show(&text);
    Ok(())
}

/// Seitenweise mit `less`, wenn es ein Terminal ist und nicht auf eine Seite
/// passt (`-F`); sonst einfach ausgeben.
fn show(text: &str) {
    if term::is_interactive()
        && let Ok(mut child) = Command::new("less")
            .args(["-R", "-F", "-X"])
            .stdin(Stdio::piped())
            .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
        return;
    }
    print!("{text}");
}

/// Eine Zeile des README: Überschrift (mit Ebene) oder Text, und ob sie in
/// einem Codeblock steht. Überschriften in Codeblöcken sind keine.
enum Line<'a> {
    Heading(usize, &'a str),
    Fence,
    Code(&'a str),
    Text(&'a str),
}

fn lines(md: &str) -> Vec<Line<'_>> {
    let mut inside = false;
    md.lines()
        .map(|l| {
            if l.trim_start().starts_with("```") {
                inside = !inside;
                return Line::Fence;
            }
            if inside {
                return Line::Code(l);
            }
            let level = l.chars().take_while(|&c| c == '#').count();
            if (1..=6).contains(&level) && l[level..].starts_with(' ') {
                Line::Heading(level, l[level..].trim())
            } else {
                Line::Text(l)
            }
        })
        .collect()
}

fn headings(md: &str) -> Vec<(usize, &str)> {
    lines(md)
        .into_iter()
        .filter_map(|l| match l {
            Line::Heading(level, t) if level >= 2 => Some((level, t)),
            _ => None,
        })
        .collect()
}

/// Die Abschnitte, deren Überschrift passt; sonst die, in deren Text es steht.
/// Ein Abschnitt reicht bis zur nächsten Überschrift derselben oder höheren Ebene.
fn find(md: &str, query: &str) -> Vec<String> {
    let q = fold(query);
    let all: Vec<&str> = md.lines().collect();
    let parsed = lines(md);
    let mut by_heading = Vec::new();
    let mut by_text = Vec::new();
    for (i, l) in parsed.iter().enumerate() {
        let Line::Heading(level, title) = l else {
            continue;
        };
        if *level < 2 {
            continue;
        }
        let end = parsed[i + 1..]
            .iter()
            .position(|l| matches!(l, Line::Heading(lv, _) if lv <= level))
            .map_or(all.len(), |p| i + 1 + p);
        let section = all[i..end].join("\n");
        if fold(title).contains(&q) {
            by_heading.push(section);
        } else if *level >= 3 && fold(&section).contains(&q) {
            by_text.push(section);
        }
    }
    if by_heading.is_empty() {
        by_text
    } else {
        by_heading
    }
}

/// Klein und ohne Umlaute: „Aufräumen“ findet sich auch mit „aufraeumen“.
fn fold(s: &str) -> String {
    s.to_lowercase()
        .replace('ä', "ae")
        .replace('ö', "oe")
        .replace('ü', "ue")
        .replace('ß', "ss")
}

/// Markdown fürs Terminal: Überschriften fett, Code eingerückt, Tabellen als
/// Spalten, Links mit Adresse, `Code` und **fett** ohne die Zeichen.
fn render(md: &str) -> String {
    let parsed = lines(md);
    let mut out = String::from("\n");
    let mut i = 0;
    while i < parsed.len() {
        match &parsed[i] {
            Line::Heading(level, t) => {
                let title = inline(t);
                let title = if *level <= 2 {
                    title.to_uppercase()
                } else {
                    title
                };
                if !out.ends_with("\n\n") {
                    out.push('\n');
                }
                out.push_str(&format!(" {}\n\n", term::bold(&title)));
            }
            Line::Fence => {}
            Line::Code(l) => out.push_str(&format!("     {}\n", term::cyan(l))),
            Line::Text(l) if l.trim_start().starts_with('|') => {
                let start = i;
                while i < parsed.len()
                    && matches!(&parsed[i], Line::Text(l) if l.trim_start().starts_with('|'))
                {
                    i += 1;
                }
                let rows: Vec<&str> = parsed[start..i]
                    .iter()
                    .filter_map(|l| match l {
                        Line::Text(t) => Some(*t),
                        _ => None,
                    })
                    .collect();
                out.push_str(&table(&rows));
                continue;
            }
            Line::Text(l) => {
                let t = l.trim_end();
                // Trennlinien braucht es zwischen fetten Überschriften nicht.
                let rule = t.len() >= 3 && t.chars().all(|c| matches!(c, '-' | '*' | '_'));
                // Aufzählungen mit Punkt statt Sternchen.
                let bullet = t
                    .strip_prefix("* ")
                    .or_else(|| t.strip_prefix("- "))
                    .map(|r| format!("• {r}"));
                if rule {
                } else if let Some(b) = bullet {
                    out.push_str(&format!(" {}\n", inline(&b)));
                } else if t.is_empty() {
                    if !out.ends_with("\n\n") {
                        out.push('\n');
                    }
                } else {
                    out.push_str(&format!(" {}\n", inline(t)));
                }
            }
        }
        i += 1;
    }
    out.trim_end().to_string() + "\n\n"
}

/// `| a | b |` als Spalten. Die Trennzeile `|---|---|` fällt weg, eine leere
/// Kopfzeile auch.
fn table(rows: &[&str]) -> String {
    let cells: Vec<Vec<String>> = rows
        .iter()
        .map(|r| {
            r.trim()
                .trim_matches('|')
                .split('|')
                .map(|c| inline(c.trim()))
                .collect::<Vec<_>>()
        })
        .filter(|cs| {
            !cs.iter()
                .all(|c| c.chars().all(|ch| matches!(ch, '-' | ':' | ' ')))
        })
        .collect();
    let first = cells.first().map(|c| c.len()).unwrap_or(0);
    let w = cells
        .iter()
        .filter_map(|c| c.first())
        .map(|c| visible_width(c))
        .max()
        .unwrap_or(0);
    let mut out = String::new();
    for (n, row) in cells.iter().enumerate() {
        let head = n == 0 && first > 1;
        let rest = row[1..].join("  ");
        let pad = " ".repeat(w.saturating_sub(visible_width(&row[0])));
        let line = format!("   {}{pad}  {}", row[0], rest);
        if head {
            if rest.trim().is_empty() && row[0].trim() == "Befehl" {
                continue;
            }
            out.push_str(&format!("{}\n", term::dim(line.trim_end())));
        } else {
            out.push_str(&format!("{}\n", line.trim_end()));
        }
    }
    out
}

/// Breite ohne Farbcodes.
fn visible_width(s: &str) -> usize {
    let mut plain = String::new();
    let mut esc = false;
    for c in s.chars() {
        match (esc, c) {
            (false, '\x1b') => esc = true,
            (true, 'm') => esc = false,
            (true, _) => {}
            (false, c) => plain.push(c),
        }
    }
    term::text_width(&plain)
}

/// `code` und **fett** ohne Zeichen, [Text](Adresse) als „Text (Adresse)“.
fn inline(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(c) = rest.chars().next() {
        if c == '`'
            && let Some(end) = rest[1..].find('`')
        {
            out.push_str(&term::cyan(&rest[1..1 + end]));
            rest = &rest[end + 2..];
            continue;
        }
        if rest.starts_with("**")
            && let Some(end) = rest[2..].find("**")
        {
            out.push_str(&term::bold(&rest[2..2 + end]));
            rest = &rest[end + 4..];
            continue;
        }
        // *kursiv* ohne Sternchen (ein „* “ am Anfang ist eine Aufzählung).
        if c == '*'
            && !rest.starts_with("* ")
            && let Some(end) = rest[1..].find('*')
            && end > 0
        {
            out.push_str(&rest[1..1 + end]);
            rest = &rest[end + 2..];
            continue;
        }
        if c == '['
            && let Some(mid) = rest.find("](")
            && let Some(end) = rest[mid..].find(')')
        {
            let text = &rest[1..mid];
            let url = &rest[mid + 2..mid + end];
            if url.starts_with("http") {
                out.push_str(&format!("{text} ({url})"));
            } else {
                out.push_str(text);
            }
            rest = &rest[mid + end + 1..];
            continue;
        }
        out.push(c);
        rest = &rest[c.len_utf8()..];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const MD: &str = "# Fokus\n\nKurz.\n\n## Typische Situationen\n\n### Morgens\n\n```\nfokus\n## Neu\n```\n\nText mit `fokus heute`.\n\n### Einmal pro Woche: aufräumen\n\nGeht mit **Ruhe**.\n\n## Alle Befehle\n\n| Befehl | |\n|---|---|\n| `fokus` | was dran ist |\n| `fokus alles` | vier Felder |\n";

    #[test]
    fn ueberschriften_in_codebloecken_zaehlen_nicht() {
        let titles: Vec<&str> = headings(MD).into_iter().map(|(_, t)| t).collect();
        assert_eq!(
            titles,
            vec![
                "Typische Situationen",
                "Morgens",
                "Einmal pro Woche: aufräumen",
                "Alle Befehle"
            ]
        );
    }

    #[test]
    fn ein_stichwort_findet_den_abschnitt() {
        let found = find(MD, "aufraeumen");
        assert_eq!(found.len(), 1);
        assert!(found[0].starts_with("### Einmal pro Woche"));
        assert!(!found[0].contains("Alle Befehle"));
        // Eine höhere Überschrift nimmt ihre Unterabschnitte mit.
        let found = find(MD, "situationen");
        assert!(found[0].contains("### Morgens") && found[0].contains("Ruhe"));
        // Steht es nur im Text, kommt der Abschnitt drumherum.
        let found = find(MD, "fokus heute");
        assert!(found[0].starts_with("### Morgens"));
        assert!(find(MD, "gibtsnicht").is_empty());
    }

    #[test]
    fn markdown_wird_lesbar() {
        assert_eq!(inline("mit `fokus` und **fett**"), "mit fokus und fett");
        assert_eq!(inline("*Was ist dran?*"), "Was ist dran?");
        assert_eq!(
            inline("[Rust](https://rustup.rs) und [LICENSE](LICENSE)"),
            "Rust (https://rustup.rs) und LICENSE"
        );
        let t = render(MD);
        assert!(t.contains(" TYPISCHE SITUATIONEN\n"));
        assert!(t.contains("     ## Neu\n"));
        assert!(t.contains("   fokus        was dran ist\n"), "{t}");
        assert!(t.contains("   fokus alles  vier Felder\n"));
        assert!(!t.contains("Befehl"));
        assert!(!t.contains("|---"));
        assert!(!t.contains("```"));
    }
}
