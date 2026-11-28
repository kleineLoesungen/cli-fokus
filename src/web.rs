//! `fokus web`: dasselbe Fokus im Browser-Tab, im Look des Terminals.
//!
//! Ein kleiner Server nur auf 127.0.0.1, ohne weitere Abhängigkeiten. Jeder
//! Befehl läuft als eigener `fokus`-Aufruf, genau wie im Terminal; die Seite
//! zeigt die Antwort und oben immer die Liste für heute. Dazu: Links und
//! Aufgabennummern zum Anklicken, Tab-Taste, frühere Befehle mit ↑.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::term;

const PAGE: &str = include_str!("web.html");
const DEFAULT_PORT: u16 = 4711;

pub fn run(args: &[String]) -> Result<(), String> {
    let port = match args.first() {
        Some(p) => p
            .parse::<u16>()
            .map_err(|_| format!("„{p}“ ist keine Portnummer. Zum Beispiel: fokus web 4712"))?,
        None => DEFAULT_PORT,
    };
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| {
        format!("Port {port} ist nicht frei ({e}). Läuft Fokus schon? Sonst: fokus web 4712")
    })?;
    let url = format!("http://127.0.0.1:{port}");
    println!(
        " {} Fokus im Browser: {url}\n {}",
        term::green("✓"),
        term::dim("Nur auf diesem Mac erreichbar. Strg+C beendet.")
    );
    let _ = Command::new(term::opener()).arg(&url).status();

    let exe = std::env::current_exe().map_err(|e| format!("Ich finde mich selbst nicht: {e}"))?;
    // Immer nur ein Befehl auf einmal: zwei gleichzeitige könnten sich die Datei überschreiben.
    let lock = Arc::new(Mutex::new(exe));
    for stream in listener.incoming().flatten() {
        let lock = Arc::clone(&lock);
        // Ein eigener Faden je Verbindung: Browser öffnen gern Verbindungen auf Vorrat.
        std::thread::spawn(move || {
            let _ = handle(stream, port, &lock);
        });
    }
    Ok(())
}

struct Request {
    method: String,
    path: String,
    host: String,
    origin: Option<String>,
    fokus_header: bool,
    body: String,
}

fn read_request(stream: &TcpStream) -> std::io::Result<Request> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("").to_string();
    let (mut host, mut origin, mut fokus_header, mut len) = (String::new(), None, false, 0usize);
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h)? == 0 || h.trim().is_empty() {
            break;
        }
        let Some((k, v)) = h.split_once(':') else {
            continue;
        };
        let v = v.trim().to_string();
        match k.trim().to_lowercase().as_str() {
            "host" => host = v,
            "origin" => origin = Some(v),
            "x-fokus" => fokus_header = true,
            "content-length" => len = v.parse().unwrap_or(0).min(64 * 1024),
            _ => {}
        }
    }
    let mut body = vec![0; len];
    reader.read_exact(&mut body)?;
    Ok(Request {
        method,
        path,
        host,
        origin,
        fokus_header,
        body: String::from_utf8_lossy(&body).into_owned(),
    })
}

fn handle(
    mut stream: TcpStream,
    port: u16,
    lock: &Mutex<std::path::PathBuf>,
) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    let req = read_request(&stream)?;
    if !allowed(&req, port) {
        return respond(&mut stream, "403 Forbidden", "text/plain", "Nicht erlaubt.");
    }
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/") => respond(&mut stream, "200 OK", "text/html; charset=utf-8", PAGE),
        ("POST", "/befehl") => {
            let (cols, line) = split_body(&req.body);
            let mut words = words(line);
            if words.first().is_some_and(|w| w == "web") {
                words = vec!["hilfe".into()];
            }
            let out = {
                let exe = lock.lock().unwrap_or_else(|e| e.into_inner());
                fokus(&exe, &words, cols)
            };
            respond(
                &mut stream,
                "200 OK",
                "text/html; charset=utf-8",
                &to_html(&out),
            )
        }
        ("POST", "/ergaenzen") => {
            let (_, line) = split_body(&req.body);
            let mut args = vec!["_complete".to_string()];
            args.extend(words(line));
            if line.is_empty() || line.ends_with(' ') {
                args.push(String::new());
            }
            let exe = lock.lock().unwrap_or_else(|e| e.into_inner());
            let out = fokus(&exe, &args, 80);
            respond(&mut stream, "200 OK", "text/plain; charset=utf-8", &out)
        }
        _ => respond(&mut stream, "404 Not Found", "text/plain", "Gibt es nicht."),
    }
}

/// Nur die eigene Seite darf Befehle schicken. Der Host-Kopf schützt vor
/// fremden Seiten, die sich als 127.0.0.1 ausgeben (DNS-Rebinding); der eigene
/// Kopf `X-Fokus` kann von fremden Seiten nicht ohne Rückfrage gesetzt werden,
/// und diese Rückfrage (OPTIONS) beantwortet Fokus nicht.
fn allowed(req: &Request, port: u16) -> bool {
    let hosts = [format!("127.0.0.1:{port}"), format!("localhost:{port}")];
    if !hosts.contains(&req.host) {
        return false;
    }
    if req.method == "GET" {
        return true;
    }
    req.fokus_header
        && req
            .origin
            .as_ref()
            .is_none_or(|o| hosts.iter().any(|h| *o == format!("http://{h}")))
}

/// Erste Zeile: Breite in Zeichen, zweite: der Befehl.
fn split_body(body: &str) -> (usize, &str) {
    match body.split_once('\n') {
        Some((cols, line)) => (
            cols.trim().parse().unwrap_or(80),
            line.trim_end_matches(['\r', '\n']),
        ),
        None => (80, body),
    }
}

/// Die Wörter des Befehls; ein „fokus“ oder „f“ vorne darf mitgetippt werden.
fn words(line: &str) -> Vec<String> {
    let mut w: Vec<String> = line.split_whitespace().map(String::from).collect();
    if w.first().is_some_and(|f| f == "fokus" || f == "f") {
        w.remove(0);
    }
    w
}

fn fokus(exe: &std::path::Path, args: &[String], cols: usize) -> String {
    let out = Command::new(exe)
        .args(args)
        .env("FOKUS_FARBE", "1")
        .env("FOKUS_WEB", "1")
        .env("COLUMNS", cols.to_string())
        .stdin(Stdio::null())
        .output();
    match out {
        Ok(o) => {
            let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
            s.push_str(&String::from_utf8_lossy(&o.stderr));
            s
        }
        Err(e) => format!(" Fokus ließ sich nicht starten: {e}"),
    }
}

fn respond(stream: &mut TcpStream, status: &str, kind: &str, body: &str) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )?;
    stream.flush()
}

/// Terminal-Text als HTML: Farben als Klassen, Links anklickbar, Aufgabennummern
/// am Zeilenanfang (und in den vier Feldern) zeigen beim Klick die Aufgabe.
pub fn to_html(text: &str) -> String {
    let mut out = String::new();
    for (i, line) in text.lines().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&line_html(line));
    }
    out
}

fn line_html(line: &str) -> String {
    // Erst in Abschnitte mit gleicher Farbe zerlegen.
    let mut spans: Vec<(Vec<&'static str>, String)> = Vec::new();
    let mut classes: Vec<&'static str> = Vec::new();
    let mut rest = line;
    while let Some(p) = rest.find("\x1b[") {
        spans.push((classes.clone(), rest[..p].to_string()));
        let after = &rest[p + 2..];
        let end = after.find('m').unwrap_or(after.len());
        for code in after[..end].split(';') {
            match code {
                "0" | "" => classes.clear(),
                "1" => classes.push("b"),
                "2" => classes.push("d"),
                "31" => classes.push("red"),
                "32" => classes.push("green"),
                "33" => classes.push("yellow"),
                "36" => classes.push("cyan"),
                _ => {}
            }
        }
        rest = after.get(end + 1..).unwrap_or("");
    }
    spans.push((classes, rest.to_string()));

    let plain: String = spans.iter().map(|(_, s)| s.as_str()).collect();
    let nrs = task_nrs(&plain);
    let mut out = String::new();
    let mut pos = 0;
    for (cls, s) in spans {
        if s.is_empty() {
            continue;
        }
        let inner = linkify(&s, pos, &nrs);
        pos += s.len();
        if cls.is_empty() {
            out.push_str(&inner);
        } else {
            out.push_str(&format!("<span class=\"{}\">{inner}</span>", cls.join(" ")));
        }
    }
    out
}

/// Wo in der Zeile Aufgabennummern stehen: Byte-Bereich und Nummer.
/// Erkannt wird eine Zahl, vor der nur Leerraum steht (am Zeilenanfang oder
/// nach einem „│“ der vier Felder) und nach der zwei Leerzeichen oder ein
/// Zeichen wie ▶ · ✓ × folgen.
fn task_nrs(line: &str) -> Vec<(usize, usize, String)> {
    let starts = std::iter::once(0).chain(line.match_indices('│').map(|(p, m)| p + m.len()));
    let mut found = Vec::new();
    for from in starts {
        let seg = &line[from..];
        let lead = seg.len() - seg.trim_start().len();
        let digits: String = seg[lead..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if digits.is_empty() || digits.len() > 4 {
            continue;
        }
        let after = &seg[lead + digits.len()..];
        let ok = after.starts_with("  ")
            || ["▶", "·", "✓", "×", "↻"]
                .iter()
                .any(|m| after.strip_prefix(' ').is_some_and(|a| a.starts_with(m)));
        if ok {
            let s = from + lead;
            found.push((s, s + digits.len(), digits));
        }
    }
    found
}

/// HTML-sicher machen; Links, Aufgabennummern, `#themen` und `@personen` werden anklickbar.
/// `offset` ist die Lage von `s` in der ganzen Zeile.
fn linkify(s: &str, offset: usize, nrs: &[(usize, usize, String)]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < s.len() {
        let here = &s[i..];
        if let Some((a, b, n)) = nrs
            .iter()
            .find(|(a, b, _)| offset + i == *a && offset + s.len() >= *b)
        {
            out.push_str(&format!("<a href=\"#\" data-cmd=\"{n}\">{n}</a>"));
            i += b - a;
            continue;
        }
        if here.starts_with("https://") || here.starts_with("http://") {
            let end = here
                .find(|c: char| c.is_whitespace() || c == '<' || c == '"')
                .unwrap_or(here.len());
            let url = here[..end].trim_end_matches(['.', ',', ')', '“', '”']);
            let url = escape(url);
            out.push_str(&format!(
                "<a href=\"{url}\" target=\"_blank\" rel=\"noopener\">{url}</a>"
            ));
            i += here[..end]
                .trim_end_matches(['.', ',', ')', '“', '”'])
                .len();
            continue;
        }
        let at_word_start = s[..i]
            .chars()
            .next_back()
            .is_none_or(|p| p.is_whitespace() || p == '(');
        if at_word_start && let Some(tag) = tag_at(here) {
            let tag = escape(tag);
            out.push_str(&format!("<a href=\"#\" data-cmd=\"{tag}\">{tag}</a>"));
            i += tag.len();
            continue;
        }
        let c = here.chars().next().unwrap_or(' ');
        out.push_str(&escape(&c.to_string()));
        i += c.len_utf8();
    }
    out
}

/// `#thema` oder `@person` am Anfang von `s`, ohne Satzzeichen dahinter.
/// Die Platzhalter aus den Hinweisen (`@name`, `#thema`) zählen nicht.
fn tag_at(s: &str) -> Option<&str> {
    if !s.starts_with(['#', '@']) {
        return None;
    }
    // Ein Punkt gehört dazu, wenn danach noch etwas kommt: `@michael.l`, aber nicht `@tom.`
    let end = s[1..]
        .find(|c: char| !(c.is_alphanumeric() || matches!(c, '-' | '_' | '.')))
        .map_or(s.len(), |p| p + 1);
    let tag = s[..end].trim_end_matches(['-', '_', '.']);
    // Angeschnitten („@michael…“): lieber kein Link als einer zum falschen Namen.
    if s[tag.len()..].starts_with('…') {
        return None;
    }
    (tag.len() > 1 && !matches!(tag, "@name" | "#thema")).then_some(tag)
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn farben_werden_zu_klassen() {
        assert_eq!(
            to_html(" \x1b[32m✓\x1b[0m Aufgabe <1>"),
            " <span class=\"green\">✓</span> Aufgabe &lt;1&gt;"
        );
    }

    #[test]
    fn links_und_nummern_sind_anklickbar() {
        let h = to_html(" Siehe https://notion.so/x?a=1&b=2.");
        assert!(
            h.contains("<a href=\"https://notion.so/x?a=1&amp;b=2\""),
            "{h}"
        );
        assert!(h.ends_with("</a>."), "{h}");
        let h = to_html("   2 · Bericht      fällig morgen");
        assert!(
            h.starts_with("   <a href=\"#\" data-cmd=\"2\">2</a> · Bericht"),
            "{h}"
        );
        let h = to_html("   WICHTIG │ 12   Bericht   morgen │  7 ▶ Keller │");
        assert!(h.contains("data-cmd=\"12\""), "{h}");
        assert!(h.contains("data-cmd=\"7\""), "{h}");
    }

    #[test]
    fn themen_und_personen_sind_anklickbar() {
        let h = to_html("   3 · Keller ausmisten    bei @tom  #eigenes-projekt");
        assert!(
            h.contains("<a href=\"#\" data-cmd=\"@tom\">@tom</a>"),
            "{h}"
        );
        assert!(h.contains("data-cmd=\"#eigenes-projekt\""), "{h}");
        let h = to_html(" Aufgabe 1 liegt jetzt bei @michael.l: Retro");
        assert!(h.contains("data-cmd=\"@michael.l\""), "{h}");
        assert!(to_html(" Frag @tom.").contains("data-cmd=\"@tom\">@tom</a>."));
        assert!(!to_html(" Retro @michael…").contains("data-cmd"));
        let h = to_html(" Neues Thema #haus, über der Linie.");
        assert!(h.contains(">#haus</a>,"), "{h}");
        assert!(!to_html(" Alles zu einer Person: fokus @name").contains("data-cmd"));
        assert!(!to_html(" C#-Kurs").contains("data-cmd"));
        let h = to_html(" https://x.de/a#teil");
        assert_eq!(h.matches("<a ").count(), 1, "{h}");
    }

    #[test]
    fn gewoehnliche_zahlen_bleiben_text() {
        assert!(!to_html(" 1 wartet auf andere → fokus alles").contains("data-cmd"));
        assert!(!to_html(" Samstag, 28. November   0 von 1 erledigt").contains("data-cmd"));
    }

    #[test]
    fn fremde_seiten_duerfen_nichts_schicken() {
        let req = |host: &str, origin: Option<&str>, header: bool| Request {
            method: "POST".into(),
            path: "/befehl".into(),
            host: host.into(),
            origin: origin.map(String::from),
            fokus_header: header,
            body: String::new(),
        };
        assert!(allowed(
            &req("127.0.0.1:4711", Some("http://127.0.0.1:4711"), true),
            4711
        ));
        assert!(!allowed(
            &req("127.0.0.1:4711", Some("http://127.0.0.1:4711"), false),
            4711
        ));
        assert!(!allowed(
            &req("127.0.0.1:4711", Some("https://boese.example"), true),
            4711
        ));
        assert!(!allowed(&req("boese.example:4711", None, true), 4711));
    }

    #[test]
    fn fokus_vorne_darf_mitgetippt_werden() {
        assert_eq!(words("f + Brot"), ["+", "Brot"]);
        assert_eq!(words("fokus fertig 2"), ["fertig", "2"]);
        assert_eq!(split_body("90\nheute 3"), (90, "heute 3"));
    }
}
