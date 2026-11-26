//! Eine neue Aufgabe aus dem, was getippt wurde: Text, und am Ende vielleicht ein Termin.

use chrono::NaiveDate;

use crate::{date, tags};

pub struct Parsed {
    pub text: String,
    pub due: Option<NaiveDate>,
}

/// Der Termin steht am Ende (`#thema` und `@person` dürfen dahinter stehen).
/// So bleibt „Termin morgen absagen“ ein Text ohne Termin, und „Präsentation
/// vorbereiten morgen“ ist morgen fällig.
pub fn parse_new(input: &str, today: NaiveDate) -> Parsed {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let words: Vec<usize> = (0..tokens.len())
        .filter(|&i| !tags::is_tag(tokens[i], '#') && !tags::is_tag(tokens[i], '@'))
        .collect();
    for k in (1..=4).rev() {
        // Mindestens ein Wort muss als Text übrig bleiben.
        if words.len() <= k {
            continue;
        }
        let tail = &words[words.len() - k..];
        let phrase: Vec<String> = tail.iter().map(|&i| tokens[i].to_string()).collect();
        if let Some(due) = date::parse_phrase(&phrase, today) {
            let text = (0..tokens.len())
                .filter(|i| !tail.contains(i))
                .map(|i| tokens[i])
                .collect::<Vec<_>>()
                .join(" ");
            return Parsed {
                text,
                due: Some(due),
            };
        }
    }
    Parsed {
        text: tokens.join(" "),
        due: None,
    }
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
}
