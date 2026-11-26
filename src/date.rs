//! Datum: heute, deutsche Schreibweisen lesen, freundlich anzeigen.

use chrono::{Datelike, Days, Local, Months, NaiveDate, Weekday};

const WEEKDAYS: [&str; 7] = [
    "Montag",
    "Dienstag",
    "Mittwoch",
    "Donnerstag",
    "Freitag",
    "Samstag",
    "Sonntag",
];
const MONTHS: [&str; 12] = [
    "Januar",
    "Februar",
    "März",
    "April",
    "Mai",
    "Juni",
    "Juli",
    "August",
    "September",
    "Oktober",
    "November",
    "Dezember",
];

/// Heute. `FOKUS_HEUTE=2026-11-26` stellt den Tag fest ein (für Tests).
pub fn today() -> NaiveDate {
    if let Ok(s) = std::env::var("FOKUS_HEUTE")
        && let Ok(d) = NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d")
    {
        return d;
    }
    Local::now().date_naive()
}

/// So steht ein Datum in der Datei: `27.11.2026`.
pub fn to_file(d: NaiveDate) -> String {
    d.format("%d.%m.%Y").to_string()
}

pub fn from_file(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s.trim(), "%d.%m.%Y").ok()
}

pub fn weekday(d: NaiveDate) -> &'static str {
    WEEKDAYS[d.weekday().num_days_from_monday() as usize]
}

/// `27.11.`
pub fn short(d: NaiveDate) -> String {
    format!("{}.{}.", d.day(), d.month())
}

/// „Mittwoch, 26. November“
pub fn long(d: NaiveDate) -> String {
    format!(
        "{}, {}. {}",
        weekday(d),
        d.day(),
        MONTHS[d.month0() as usize]
    )
}

/// „Donnerstag 27.11.“ – eindeutig, für Bestätigungen.
pub fn exact(d: NaiveDate) -> String {
    format!("{} {}", weekday(d), short(d))
}

fn short_or_full(d: NaiveDate, today: NaiveDate) -> String {
    if d.year() == today.year() {
        short(d)
    } else {
        to_file(d)
    }
}

/// „fällig morgen“, „fällig Freitag“, „3 Tage überfällig“
pub fn due_label(due: NaiveDate, today: NaiveDate) -> String {
    let n = (due - today).num_days();
    match n {
        i64::MIN..=-2 => format!("{} Tage überfällig", -n),
        -1 => "seit gestern fällig".into(),
        0 => "fällig heute".into(),
        1 => "fällig morgen".into(),
        2 => "fällig übermorgen".into(),
        3..=6 => format!("fällig {}", weekday(due)),
        _ => format!("fällig {}", short_or_full(due, today)),
    }
}

/// Kurzform für enge Spalten: „morgen“, „Freitag“, „überfällig“
pub fn due_short(due: NaiveDate, today: NaiveDate) -> String {
    match (due - today).num_days() {
        i64::MIN..=-1 => "überfällig".into(),
        0 => "heute".into(),
        1 => "morgen".into(),
        2 => "übermorgen".into(),
        3..=6 => weekday(due).into(),
        _ => short_or_full(due, today),
    }
}

/// „seit 5 Tagen“
pub fn age_label(since: NaiveDate, today: NaiveDate) -> String {
    match (today - since).num_days() {
        i64::MIN..=0 => "seit heute".into(),
        1 => "seit gestern".into(),
        n => format!("seit {n} Tagen"),
    }
}

/// Der Sonntag dieser Woche (heute, wenn heute Sonntag ist).
pub fn end_of_week(today: NaiveDate) -> NaiveDate {
    let left = 6 - today.weekday().num_days_from_monday();
    today + Days::new(left as u64)
}

/// Liest einen Termin wie `morgen`, `freitag`, `3.12.`, `in 2 wochen`, `bis nächste woche`.
/// Die Wörter müssen vollständig einen Termin ergeben, sonst `None`.
pub fn parse_phrase(words: &[String], today: NaiveDate) -> Option<NaiveDate> {
    let mut w: Vec<String> = words
        .iter()
        .map(|s| {
            let s = s.to_lowercase();
            let s = s.trim_end_matches([',', '!', '?', ';', ')']);
            // Ein Punkt am Ende gehört nur bei Zahlen zum Datum („3.12.“), sonst zum Satz.
            if s.chars().any(|c| c.is_ascii_digit()) {
                s.to_string()
            } else {
                s.trim_end_matches('.').to_string()
            }
        })
        .collect();
    if w.len() > 1 && matches!(w[0].as_str(), "bis" | "am" | "zum" | "spätestens") {
        w.remove(0);
    }
    match w.as_slice() {
        [a] => single(a, today),
        [a, b] if is_next(a) => {
            if b == "woche" {
                Some(next_weekday(today, Weekday::Mon, true))
            } else {
                weekday_of(b).map(|wd| next_weekday(today, wd, true))
            }
        }
        [a, b, c] if a == "in" => {
            let n = number(b)?;
            match c.as_str() {
                "tag" | "tage" | "tagen" => today.checked_add_days(Days::new(n)),
                "woche" | "wochen" => today.checked_add_days(Days::new(n * 7)),
                "monat" | "monate" | "monaten" => today.checked_add_months(Months::new(n as u32)),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Liest eine Wiederholung wie `jeden montag`, `jede woche`, `alle 2 wochen freitag`,
/// `alle 4 wochen am montag`. Ergebnis: alle so viele Wochen, ab diesem Termin.
pub fn parse_recurrence(words: &[String], today: NaiveDate) -> Option<(u32, NaiveDate)> {
    let w: Vec<String> = words
        .iter()
        .map(|s| {
            s.to_lowercase()
                .trim_end_matches([',', '.', '!', '?'])
                .to_string()
        })
        .filter(|s| s != "am")
        .collect();
    let at = |wd: &str| weekday_of(wd).map(|d| next_weekday(today, d, false));
    let (n, due) = match w.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["jeden", wd] => (1, at(wd)?),
        ["jede", "woche"] | ["wöchentlich"] => (1, today),
        ["jede", "woche", wd] | ["wöchentlich", wd] => (1, at(wd)?),
        ["alle", n, "wochen" | "woche"] => (number(n)?, today),
        ["alle", n, "wochen" | "woche", wd] => (number(n)?, at(wd)?),
        _ => return None,
    };
    let n = u32::try_from(n).ok().filter(|&n| (1..=52).contains(&n))?;
    Some((n, due))
}

/// „jeden Montag“, „alle 2 Wochen am Freitag“
pub fn recurrence_label(every: u32, due: NaiveDate) -> String {
    if every == 1 {
        format!("jeden {}", weekday(due))
    } else {
        format!("alle {every} Wochen am {}", weekday(due))
    }
}

fn single(a: &str, today: NaiveDate) -> Option<NaiveDate> {
    match a {
        "heute" => Some(today),
        "morgen" => today.checked_add_days(Days::new(1)),
        "übermorgen" | "uebermorgen" => today.checked_add_days(Days::new(2)),
        "wochenende" => Some(next_weekday(today, Weekday::Sat, false)),
        _ => weekday_of(a)
            .map(|wd| next_weekday(today, wd, false))
            .or_else(|| numeric(a, today)),
    }
}

fn is_next(a: &str) -> bool {
    matches!(
        a,
        "nächste" | "nächsten" | "nächster" | "naechste" | "naechsten" | "naechster"
    )
}

/// Wochentage, ausgeschrieben oder kurz. „so“ fehlt absichtlich: Es ist zu oft ein
/// ganz normales Wort am Satzende („… einfach so“).
pub(crate) fn weekday_of(a: &str) -> Option<Weekday> {
    Some(match a {
        "montag" | "mo" => Weekday::Mon,
        "dienstag" | "di" => Weekday::Tue,
        "mittwoch" | "mi" => Weekday::Wed,
        "donnerstag" | "do" => Weekday::Thu,
        "freitag" | "fr" => Weekday::Fri,
        "samstag" | "sa" | "sonnabend" => Weekday::Sat,
        "sonntag" => Weekday::Sun,
        _ => return None,
    })
}

/// Der nächste solche Wochentag; heute zählt mit, außer `strictly` ist gesetzt.
pub(crate) fn next_weekday(today: NaiveDate, wd: Weekday, strictly: bool) -> NaiveDate {
    let now = today.weekday().num_days_from_monday() as i64;
    let target = wd.num_days_from_monday() as i64;
    let mut diff = (target - now).rem_euclid(7);
    if diff == 0 && strictly {
        diff = 7;
    }
    today + Days::new(diff as u64)
}

pub(crate) fn number(s: &str) -> Option<u64> {
    if let Ok(n) = s.parse() {
        return Some(n);
    }
    Some(match s {
        "ein" | "eine" | "einer" | "einem" | "einen" => 1,
        "zwei" => 2,
        "drei" => 3,
        "vier" => 4,
        "fünf" | "fuenf" => 5,
        "sechs" => 6,
        "sieben" => 7,
        "acht" => 8,
        "neun" => 9,
        "zehn" => 10,
        _ => return None,
    })
}

/// `3.12.`, `3.12`, `3.12.26`, `3.12.2026`. Ohne Jahr: das nächste solche Datum.
fn numeric(a: &str, today: NaiveDate) -> Option<NaiveDate> {
    let parts: Vec<&str> = a.trim_end_matches('.').split('.').collect();
    if parts.len() < 2
        || parts.len() > 3
        || parts
            .iter()
            .any(|p| p.is_empty() || !p.chars().all(|c| c.is_ascii_digit()))
    {
        return None;
    }
    let day: u32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    match parts.get(2) {
        Some(y) => {
            let year: i32 = match y.len() {
                2 => 2000 + y.parse::<i32>().ok()?,
                4 => y.parse().ok()?,
                _ => return None,
            };
            NaiveDate::from_ymd_opt(year, month, day)
        }
        None => {
            let d = NaiveDate::from_ymd_opt(today.year(), month, day)?;
            if d < today {
                NaiveDate::from_ymd_opt(today.year() + 1, month, day)
            } else {
                Some(d)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Donnerstag
    fn heute() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 26).unwrap()
    }

    fn p(s: &str) -> Option<NaiveDate> {
        let w: Vec<String> = s.split_whitespace().map(String::from).collect();
        parse_phrase(&w, heute())
    }

    fn d(y: i32, m: u32, t: u32) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(y, m, t)
    }

    #[test]
    fn heute_morgen_uebermorgen() {
        assert_eq!(p("heute"), d(2026, 11, 26));
        assert_eq!(p("morgen"), d(2026, 11, 27));
        assert_eq!(p("Übermorgen"), d(2026, 11, 28));
    }

    #[test]
    fn wochentage_zaehlen_ab_heute() {
        assert_eq!(p("freitag"), d(2026, 11, 27));
        assert_eq!(p("fr"), d(2026, 11, 27));
        assert_eq!(p("donnerstag"), d(2026, 11, 26));
        assert_eq!(p("nächsten donnerstag"), d(2026, 12, 3));
        assert_eq!(p("mittwoch"), d(2026, 12, 2));
        assert_eq!(p("montag"), d(2026, 11, 30));
        assert_eq!(p("bis sonntag"), d(2026, 11, 29));
    }

    #[test]
    fn so_allein_ist_kein_sonntag() {
        assert_eq!(p("so"), None);
    }

    #[test]
    fn datum_ohne_jahr_liegt_in_der_zukunft() {
        assert_eq!(p("3.12."), d(2026, 12, 3));
        assert_eq!(p("3.12"), d(2026, 12, 3));
        assert_eq!(p("2.1."), d(2027, 1, 2));
        assert_eq!(p("am 3.12.2027"), d(2027, 12, 3));
        assert_eq!(p("3.12.27"), d(2027, 12, 3));
        assert_eq!(p("31.2."), None);
    }

    #[test]
    fn in_tagen_wochen_monaten() {
        assert_eq!(p("in 2 wochen"), d(2026, 12, 10));
        assert_eq!(p("in einer woche"), d(2026, 12, 3));
        assert_eq!(p("in drei tagen"), d(2026, 11, 29));
        assert_eq!(p("in einem monat"), d(2026, 12, 26));
        assert_eq!(p("nächste woche"), d(2026, 11, 30));
    }

    #[test]
    fn normale_woerter_sind_kein_termin() {
        assert_eq!(p("absagen"), None);
        assert_eq!(p("in der stadt"), None);
        assert_eq!(p("v2"), None);
    }

    #[test]
    fn wiederholungen() {
        let r = |s: &str| {
            let w: Vec<String> = s.split_whitespace().map(String::from).collect();
            parse_recurrence(&w, heute())
        };
        assert_eq!(r("jeden montag"), Some((1, d(2026, 11, 30).unwrap())));
        assert_eq!(r("jeden donnerstag"), Some((1, heute())));
        assert_eq!(
            r("alle 2 wochen freitag"),
            Some((2, d(2026, 11, 27).unwrap()))
        );
        assert_eq!(
            r("alle 4 Wochen am Montag"),
            Some((4, d(2026, 11, 30).unwrap()))
        );
        assert_eq!(r("alle zwei wochen"), Some((2, heute())));
        assert_eq!(r("jede woche"), Some((1, heute())));
        assert_eq!(r("jeden tag"), None);
        assert_eq!(r("alle 0 wochen"), None);
        assert_eq!(
            recurrence_label(2, d(2026, 11, 27).unwrap()),
            "alle 2 Wochen am Freitag"
        );
    }

    #[test]
    fn anzeige_ist_freundlich() {
        let t = heute();
        assert_eq!(due_label(d(2026, 11, 27).unwrap(), t), "fällig morgen");
        assert_eq!(due_label(d(2026, 11, 30).unwrap(), t), "fällig Montag");
        assert_eq!(due_label(d(2026, 11, 23).unwrap(), t), "3 Tage überfällig");
        assert_eq!(due_label(d(2026, 12, 24).unwrap(), t), "fällig 24.12.");
        assert_eq!(due_label(d(2027, 1, 5).unwrap(), t), "fällig 05.01.2027");
        assert_eq!(long(t), "Donnerstag, 26. November");
        assert_eq!(end_of_week(t), d(2026, 11, 29).unwrap());
    }
}
