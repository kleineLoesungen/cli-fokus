//! Eine neue Aufgabe aus dem, was getippt wurde: Text, und am Ende vielleicht ein Termin.

use chrono::NaiveDate;

use crate::{date, tags};

pub struct Parsed {
    pub text: String,
    pub due: Option<NaiveDate>,
    /// Wiederholung alle so viele Wochen (der Wochentag ist der des Termins).
    pub every: Option<u32>,
}

/// Der Termin steht am Ende (`#thema` und `@person` dürfen dahinter stehen).
/// So bleibt „Termin morgen absagen“ ein Text ohne Termin, und „Präsentation
/// vorbereiten morgen“ ist morgen fällig.
pub fn parse_new(input: &str, today: NaiveDate) -> Parsed {
    parse(input, today, 1)
}

/// Wie `parse_new`, aber der Termin darf allein stehen: `focus ändern 4 morgen`.
pub fn parse_change(input: &str, today: NaiveDate) -> Parsed {
    parse(input, today, 0)
}

fn parse(input: &str, today: NaiveDate, min_words: usize) -> Parsed {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let words: Vec<usize> = (0..tokens.len())
        .filter(|&i| !tags::is_tag(tokens[i], '#') && !tags::is_tag(tokens[i], '@'))
        .collect();
    // Erst eine Wiederholung („jeden montag“, „alle 2 wochen am freitag“) …
    for k in (1..=5).rev() {
        if words.len() < k + min_words {
            continue;
        }
        let tail = &words[words.len() - k..];
        let phrase: Vec<String> = tail.iter().map(|&i| tokens[i].to_string()).collect();
        if let Some((every, due)) = date::parse_recurrence(&phrase, today) {
            return Parsed {
                text: without(&tokens, tail),
                due: Some(due),
                every: Some(every),
            };
        }
    }
    // … sonst ein einzelner Termin.
    for k in (1..=4).rev() {
        // Beim Aufschreiben muss mindestens ein Wort als Text übrig bleiben.
        if words.len() < k + min_words {
            continue;
        }
        let tail = &words[words.len() - k..];
        let phrase: Vec<String> = tail.iter().map(|&i| tokens[i].to_string()).collect();
        if let Some(due) = date::parse_phrase(&phrase, today) {
            return Parsed {
                text: without(&tokens, tail),
                due: Some(due),
                every: None,
            };
        }
    }
    Parsed {
        text: tokens.join(" "),
        due: None,
        every: None,
    }
}

fn without(tokens: &[&str], skip: &[usize]) -> String {
    (0..tokens.len())
        .filter(|i| !skip.contains(i))
        .map(|i| tokens[i])
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heute() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 26).unwrap()
    }

    fn p(s: &str) -> (String, Option<String>) {
        let r = parse_new(s, heute());
        (r.text, r.due.map(date::to_file))
    }

    #[test]
    fn termin_am_ende_wird_erkannt() {
        assert_eq!(
            p("Präsentation vorbereiten #arbeit morgen"),
            (
                "Präsentation vorbereiten #arbeit".into(),
                Some("27.11.2026".into())
            )
        );
        assert_eq!(
            p("Bericht abgeben bis 3.12. #arbeit"),
            ("Bericht abgeben #arbeit".into(), Some("03.12.2026".into()))
        );
        assert_eq!(
            p("Urlaub buchen in 2 wochen"),
            ("Urlaub buchen".into(), Some("10.12.2026".into()))
        );
    }

    #[test]
    fn termin_mitten_im_satz_bleibt_text() {
        assert_eq!(
            p("Termin morgen absagen"),
            ("Termin morgen absagen".into(), None)
        );
        assert_eq!(
            p("Mit @anna Urlaub abstimmen #familie"),
            ("Mit @anna Urlaub abstimmen #familie".into(), None)
        );
    }

    #[test]
    fn nur_ein_termin_ist_keine_aufgabe() {
        assert_eq!(p("morgen"), ("morgen".into(), None));
    }

    #[test]
    fn wiederholung_am_ende() {
        let r = parse_new("Wochenplan schreiben #arbeit jeden montag", heute());
        assert_eq!(r.text, "Wochenplan schreiben #arbeit");
        assert_eq!(r.every, Some(1));
        assert_eq!(r.due.map(date::to_file), Some("30.11.2026".into()));
        let r = parse_new("Pflanzen gießen alle 2 wochen am freitag", heute());
        assert_eq!(r.text, "Pflanzen gießen");
        assert_eq!(r.every, Some(2));
        let r = parse_change("alle 4 wochen montag", heute());
        assert_eq!(r.text, "");
        assert_eq!(r.every, Some(4));
    }

    #[test]
    fn beim_aendern_darf_der_termin_allein_stehen() {
        let r = parse_change("morgen", heute());
        assert_eq!(r.text, "");
        assert_eq!(r.due.map(date::to_file), Some("27.11.2026".into()));
        let r = parse_change("#haus in 2 wochen", heute());
        assert_eq!(r.text, "#haus");
        assert!(r.due.is_some());
    }
}
