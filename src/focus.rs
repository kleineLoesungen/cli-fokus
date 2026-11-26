//! `fokus jetzt`: der Bildschirm wird leer, bis auf eine Aufgabe und eine Uhr.

use std::io::{self, Write, stdout};
use std::time::{Duration, Instant};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{execute, queue, style::Print};

use crate::store::{Status, Store};
use crate::{date, ops, term};

const SESSION: Duration = Duration::from_secs(25 * 60);

/// Stellt das Terminal wieder her, auch wenn unterwegs etwas schiefgeht.
struct Screen;

impl Screen {
    fn enter() -> io::Result<Screen> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, Hide)?;
        Ok(Screen)
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        let _ = execute!(stdout(), Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

enum Outcome {
    Done,
    Later,
    Quit,
}

pub fn run(store: &mut Store, nr: Option<u32>) -> Result<(), String> {
    let first_today = |s: &Store| s.today_open().first().map(|&i| s.tasks[i].nr);
    let mut current = match nr {
        Some(n) => {
            store.find(n).ok_or_else(|| ops::not_found(n))?;
            n
        }
        None => first_today(store)
            .ok_or("Für heute steht nichts an. Leg eine Aufgabe fest: fokus heute <Nr>")?,
    };
    if !term::is_interactive() {
        return Err("Der Fokusmodus braucht ein Terminalfenster.".into());
    }

    let mut report = Vec::new();
    {
        let _screen = Screen::enter().map_err(term::io_err)?;
        loop {
            match session(store, current).map_err(term::io_err)? {
                Outcome::Done => {
                    report.push(ops::done(store, current)?);
                    store.save()?;
                    match first_today(store) {
                        Some(next) if ask_next(store, next).map_err(term::io_err)? => {
                            current = next
                        }
                        _ => break,
                    }
                }
                Outcome::Later => {
                    let i = store.find(current).ok_or_else(|| ops::not_found(current))?;
                    if store.tasks[i].status == Status::Today {
                        report.push(ops::later(store, current)?);
                        store.save()?;
                    }
                    break;
                }
                Outcome::Quit => break,
            }
        }
    }
    for line in report {
        println!("{line}");
    }
    Ok(())
}

fn session(store: &Store, nr: u32) -> io::Result<Outcome> {
    let i = store.find(nr).expect("Aufgabe ist vorhanden");
    let t = &store.tasks[i];
    let title = t.display();
    let mut sub: Vec<String> = Vec::new();
    if let Some(n) = store.topic_name(t) {
        sub.push(n);
    }
    if let Some(d) = t.due {
        sub.push(date::due_label(d, store.today));
    }
    let sub = sub.join(" · ");

    let mut remaining = SESSION;
    let mut last = Instant::now();
    let mut paused = false;
    let mut over = false;
    loop {
        let now = Instant::now();
        if !paused && !over {
            remaining = remaining.saturating_sub(now - last);
            if remaining.is_zero() {
                over = true;
                print!("\x07");
            }
        }
        last = now;
        draw(&title, &sub, remaining, paused, over)?;

        if event::poll(Duration::from_millis(250))?
            && let Event::Key(k) = event::read()?
            && k.kind == KeyEventKind::Press
        {
            if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('c') {
                return Ok(Outcome::Quit);
            }
            match k.code {
                KeyCode::Char('f' | 'F') => return Ok(Outcome::Done),
                KeyCode::Char('s' | 'S') => return Ok(Outcome::Later),
                KeyCode::Char('q' | 'Q') | KeyCode::Esc => return Ok(Outcome::Quit),
                KeyCode::Char('p' | 'P') if !over => paused = !paused,
                KeyCode::Char('w' | 'W') if over => {
                    remaining = SESSION;
                    over = false;
                }
                _ => {}
            }
        }
    }
}

/// Ein Kasten in der Mitte des Bildschirms.
fn draw_box(lines: &[String]) -> io::Result<()> {
    let (cols, rows) = terminal::size()?;
    let (cols, rows) = (cols as usize, rows as usize);
    let bw = 56.min(cols.saturating_sub(4)).max(20);
    let inner = bw - 2;
    let left = (cols.saturating_sub(bw)) / 2;
    let top = (rows.saturating_sub(lines.len() + 2)) / 2;
    let mut out = stdout();
    queue!(out, Clear(ClearType::All))?;
    let edge = |l: &str, r: &str| term::dim(&format!("{l}{}{r}", "─".repeat(inner)));
    queue!(out, MoveTo(left as u16, top as u16), Print(edge("╭", "╮")))?;
    for (n, line) in lines.iter().enumerate() {
        queue!(
            out,
            MoveTo(left as u16, (top + 1 + n) as u16),
            Print(term::dim("│")),
            Print(line),
            MoveTo((left + bw - 1) as u16, (top + 1 + n) as u16),
            Print(term::dim("│")),
        )?;
    }
    queue!(
        out,
        MoveTo(left as u16, (top + 1 + lines.len()) as u16),
        Print(edge("╰", "╯"))
    )?;
    out.flush()
}

fn inner_width() -> usize {
    let cols = terminal::size().map(|(c, _)| c as usize).unwrap_or(80);
    56.min(cols.saturating_sub(4)).max(20) - 2
}

fn draw(title: &str, sub: &str, remaining: Duration, paused: bool, over: bool) -> io::Result<()> {
    let inner = inner_width();
    let text_w = inner.saturating_sub(8);
    let pad = |s: String| format!("    {s}");
    let secs = remaining.as_secs();
    let clock = if over {
        "Zeit um".to_string()
    } else if paused {
        "Pause".to_string()
    } else {
        format!("{:02}:{:02}", secs / 60, secs % 60)
    };
    let bar_w = inner.saturating_sub(8 + 10);
    let elapsed = SESSION.as_secs() - secs;
    let filled = (elapsed as usize * bar_w) / SESSION.as_secs() as usize;
    let bar = format!(
        "{}{}",
        term::cyan(&"━".repeat(filled)),
        term::dim(&"░".repeat(bar_w - filled))
    );
    let keys = if over {
        "[f] fertig   [w] weitere 25 Min.   [s] später"
    } else if paused {
        "[f] fertig   [p] weiter   [s] später   [q] zurück"
    } else {
        "[f] fertig   [p] Pause   [s] später   [q] zurück"
    };
    let lines = vec![
        String::new(),
        pad(term::bold(&term::truncate(title, text_w))),
        pad(term::dim(&term::truncate(sub, text_w))),
        String::new(),
        pad(format!("{bar}  {}", term::pad_left(&clock, 7))),
        if over {
            pad(term::yellow("Kurz durchatmen. Weiter oder Schluss?"))
        } else {
            String::new()
        },
        pad(term::dim(&term::truncate(keys, text_w))),
        String::new(),
    ];
    draw_box(&lines)
}

/// Nach dem Abhaken: die nächste Aufgabe anbieten.
fn ask_next(store: &Store, next: u32) -> io::Result<bool> {
    let i = store.find(next).expect("Aufgabe ist vorhanden");
    let text_w = inner_width().saturating_sub(8);
    let pad = |s: String| format!("    {s}");
    let lines = vec![
        String::new(),
        pad(term::green("✓ Geschafft.")),
        String::new(),
        pad(term::dim("Als Nächstes:")),
        pad(term::bold(&term::truncate(
            &store.tasks[i].display(),
            text_w,
        ))),
        String::new(),
        pad(term::dim("[Enter] los   [q] Schluss für jetzt")),
        String::new(),
    ];
    draw_box(&lines)?;
    loop {
        if let Event::Key(k) = event::read()?
            && k.kind == KeyEventKind::Press
        {
            match k.code {
                KeyCode::Enter => return Ok(true),
                KeyCode::Char('q' | 'Q') | KeyCode::Esc => return Ok(false),
                KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(false);
                }
                _ => {}
            }
        }
    }
}
