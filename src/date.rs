//! Datum: heute, Termine auf Deutsch und Englisch lesen, freundlich anzeigen.

use chrono::{Datelike, Days, Local, Months, NaiveDate, Weekday};

use crate::i18n::{self, Lang};
use crate::tr;

const WEEKDAYS_EN: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];
const MONTHS_EN: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

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

/// Heute. `FOCUS_HEUTE=2026-11-26` stellt den Tag fest ein (für Tests).
pub fn today() -> NaiveDate {
    if let Ok(s) = std::env::var("FOCUS_HEUTE")
        && let Ok(d) = NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d")
    {
        return d;
    }
    Local::now().date_naive()
}

/// So steht ein Datum in der Datei: `27.11.2026`, auf Englisch `2026-11-27`.
pub fn to_file(d: NaiveDate) -> String {
    match i18n::lang() {
        Lang::De => d.format("%d.%m.%Y").to_string(),
        Lang::En => d.format("%Y-%m-%d").to_string(),
    }
}

/// Liest beide Schreibweisen, damit ein Wechsel der Sprache nichts kaputt macht.
pub fn from_file(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    NaiveDate::parse_from_str(s, "%d.%m.%Y")
        .or_else(|_| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
        .ok()
}

pub fn weekday(d: NaiveDate) -> &'static str {
    let i = d.weekday().num_days_from_monday() as usize;
    match i18n::lang() {
        Lang::De => WEEKDAYS[i],
        Lang::En => WEEKDAYS_EN[i],
    }
}

fn month(d: NaiveDate) -> &'static str {
    match i18n::lang() {
        Lang::De => MONTHS[d.month0() as usize],
        Lang::En => MONTHS_EN[d.month0() as usize],
    }
}

/// `27.11.`, auf Englisch `Nov 27`
pub fn short(d: NaiveDate) -> String {
    match i18n::lang() {
        Lang::De => format!("{}.{}.", d.day(), d.month()),
        Lang::En => format!("{} {}", &month(d)[..3], d.day()),
    }
}

/// „Mittwoch, 26. November“ · „Wednesday, November 26“
pub fn long(d: NaiveDate) -> String {
    match i18n::lang() {
        Lang::De => format!("{}, {}. {}", weekday(d), d.day(), month(d)),
        Lang::En => format!("{}, {} {}", weekday(d), month(d), d.day()),
    }
}

/// „Donnerstag 27.11.“ – eindeutig, für Bestätigungen.
pub fn exact(d: NaiveDate) -> String {
    match i18n::lang() {
        Lang::De => format!("{} {}", weekday(d), short(d)),
        Lang::En => format!("{}, {}", weekday(d), short(d)),
    }
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
        i64::MIN..=-2 => tr!("{} Tage überfällig", "{} days overdue", -n),
        -1 => tr!("seit gestern fällig", "due since yesterday"),
        0 => tr!("fällig heute", "due today"),
        1 => tr!("fällig morgen", "due tomorrow"),
        2 => tr!("fällig übermorgen", "due in 2 days"),
        3..=6 => tr!("fällig {}", "due {}", weekday(due)),
        _ => tr!("fällig {}", "due {}", short_or_full(due, today)),
    }
}

/// Kurzform für enge Spalten: „morgen“, „Freitag“, „überfällig“
pub fn due_short(due: NaiveDate, today: NaiveDate) -> String {
    match (due - today).num_days() {
        i64::MIN..=-1 => tr!("überfällig", "overdue"),
        0 => tr!("heute", "today"),
        1 => tr!("morgen", "tomorrow"),
        2 => tr!("übermorgen", "in 2 days"),
        3..=6 => weekday(due).into(),
        _ => short_or_full(due, today),
    }
}

/// „seit 5 Tagen“
pub fn age_label(since: NaiveDate, today: NaiveDate) -> String {
    match (today - since).num_days() {
        i64::MIN..=0 => tr!("seit heute", "since today"),
        1 => tr!("seit gestern", "since yesterday"),
        n => tr!("seit {n} Tagen", "for {n} days"),
    }
}

/// Wie viele Tage bis `due`, mit oder ohne Samstag und Sonntag. Ohne zählen
/// nur Montag bis Freitag zwischen morgen und dem Termin: Am Freitag ist eine
/// Aufgabe für Montag dann in einem Tag fällig, nicht in drei. Ein Termin am
/// Wochenende zählt wie der Freitag davor.
pub fn days_until(today: NaiveDate, due: NaiveDate, weekend: bool) -> i64 {
    let n = (due - today).num_days();
    if weekend || n <= 0 {
        return n;
    }
    (1..=n)
        .filter(|&k| {
            (today + Days::new(k as u64))
                .weekday()
                .num_days_from_monday()
                < 5
        })
        .count() as i64
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
    if w.len() > 1
        && matches!(
            w[0].as_str(),
            "bis" | "am" | "zum" | "spätestens" | "by" | "on" | "until" | "due"
        )
    {
        w.remove(0);
    }
    match w.as_slice() {
        [a] => single(a, today),
        [a, b] if is_next(a) => {
            if b == "woche" || b == "week" {
                Some(next_weekday(today, Weekday::Mon, true))
            } else {
                weekday_of(b).map(|wd| next_weekday(today, wd, true))
            }
        }
        [a, b, c] if a == "day" && b == "after" && c == "tomorrow" => {
            today.checked_add_days(Days::new(2))
        }
        [a, b, c] if a == "in" => {
            let n = number(b)?;
            match c.as_str() {
                "tag" | "tage" | "tagen" | "day" | "days" => today.checked_add_days(Days::new(n)),
                "woche" | "wochen" | "week" | "weeks" => today.checked_add_days(Days::new(n * 7)),
                "monat" | "monate" | "monaten" | "month" | "months" => {
                    today.checked_add_months(Months::new(n as u32))
                }
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
        .filter(|s| s != "am" && s != "on")
        .collect();
    let at = |wd: &str| weekday_of(wd).map(|d| next_weekday(today, d, false));
    let (n, due) = match w.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["jeden", wd] => (1, at(wd)?),
        ["jede", "woche"] | ["wöchentlich"] => (1, today),
        ["jede", "woche", wd] | ["wöchentlich", wd] => (1, at(wd)?),
        ["alle", n, "wochen" | "woche"] => (number(n)?, today),
        ["alle", n, "wochen" | "woche", wd] => (number(n)?, at(wd)?),
        ["every", "week"] | ["weekly"] => (1, today),
        ["every", "week", wd] | ["weekly", wd] => (1, at(wd)?),
        ["every", "other", "week"] => (2, today),
        ["every", "other", "week", wd] => (2, at(wd)?),
        ["every", n, "weeks" | "week"] => (number(n)?, today),
        ["every", n, "weeks" | "week", wd] => (number(n)?, at(wd)?),
        ["every", wd] => (1, at(wd)?),
        _ => return None,
    };
    let n = u32::try_from(n).ok().filter(|&n| (1..=52).contains(&n))?;
    Some((n, due))
}

/// „jeden Montag“, „alle 2 Wochen am Freitag“
pub fn recurrence_label(every: u32, due: NaiveDate) -> String {
    if every == 1 {
        tr!("jeden {}", "every {}", weekday(due))
    } else {
        tr!(
            "alle {every} Wochen am {}",
            "every {every} weeks on {}",
            weekday(due)
        )
    }
}

fn single(a: &str, today: NaiveDate) -> Option<NaiveDate> {
    match a {
        "heute" | "today" => Some(today),
        "morgen" | "tomorrow" => today.checked_add_days(Days::new(1)),
        "übermorgen" | "uebermorgen" => today.checked_add_days(Days::new(2)),
        "wochenende" | "weekend" => Some(next_weekday(today, Weekday::Sat, false)),
        _ => weekday_of(a)
            .map(|wd| next_weekday(today, wd, false))
            .or_else(|| numeric(a, today))
            .or_else(|| iso(a)),
    }
}

/// `2026-12-03`
fn iso(a: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(a, "%Y-%m-%d").ok()
}

fn is_next(a: &str) -> bool {
    matches!(
        a,
        "nächste" | "nächsten" | "nächster" | "naechste" | "naechsten" | "naechster" | "next"
    )
}

/// Wochentage, ausgeschrieben in beiden Sprachen. Abkürzungen nur in der
/// gewählten Sprache: Auf Englisch sind „do“ und „mi“ ganz normale Wörter, auf
/// Deutsch „so“ („… einfach so“). „sat“ und „sun“ fehlen aus demselben Grund.
pub(crate) fn weekday_of(a: &str) -> Option<Weekday> {
    let full = match a {
        "montag" | "monday" => Some(Weekday::Mon),
        "dienstag" | "tuesday" => Some(Weekday::Tue),
        "mittwoch" | "wednesday" => Some(Weekday::Wed),
        "donnerstag" | "thursday" => Some(Weekday::Thu),
        "freitag" | "friday" => Some(Weekday::Fri),
        "samstag" | "sonnabend" | "saturday" => Some(Weekday::Sat),
        "sonntag" | "sunday" => Some(Weekday::Sun),
        _ => None,
    };
    if full.is_some() {
        return full;
    }
    match i18n::lang() {
        Lang::De => match a {
            "mo" => Some(Weekday::Mon),
            "di" => Some(Weekday::Tue),
            "mi" => Some(Weekday::Wed),
            "do" => Some(Weekday::Thu),
            "fr" => Some(Weekday::Fri),
            "sa" => Some(Weekday::Sat),
            _ => None,
        },
        Lang::En => match a {
            "mon" => Some(Weekday::Mon),
            "tue" | "tues" => Some(Weekday::Tue),
            "wed" => Some(Weekday::Wed),
            "thu" | "thur" | "thurs" => Some(Weekday::Thu),
            "fri" => Some(Weekday::Fri),
            _ => None,
        },
    }
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
        "a" | "an" | "one" => 1,
        "two" => 2,
        "three" => 3,
        "four" => 4,
        "five" => 5,
        "six" => 6,
        "seven" => 7,
        "eight" => 8,
        "nine" => 9,
        "ten" => 10,
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

    #[test]
    fn ohne_wochenende_zaehlen_nur_werktage() {
        // Freitag, 27.11.
        let fr = d(2026, 11, 27).unwrap();
        let mo = d(2026, 11, 30).unwrap();
        assert_eq!(days_until(fr, mo, true), 3);
        assert_eq!(days_until(fr, mo, false), 1);
        // Termin am Sonntag: wie der Freitag davor, also heute.
        assert_eq!(days_until(fr, d(2026, 11, 29).unwrap(), false), 0);
        // Überfälliges bleibt überfällig.
        assert_eq!(days_until(fr, d(2026, 11, 25).unwrap(), false), -2);
        // Donnerstag bis Dienstag: Fr, Mo, Di.
        assert_eq!(days_until(heute(), d(2026, 12, 1).unwrap(), false), 3);
    }
}
