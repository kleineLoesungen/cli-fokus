//! `#thema` und `@person` in einem Aufgabentext.

/// Entfernt Zeichen vorn (`#`, `@`) und Satzzeichen hinten: `#arbeit,` → `arbeit`.
pub fn clean(word: &str) -> &str {
    word.trim_start_matches(['#', '@'])
        .trim_end_matches(|c: char| !c.is_alphanumeric())
}

/// Vergleichsschlüssel: klein, nur Buchstaben und Ziffern.
/// So ist `#eigenes-projekt` dasselbe Thema wie „Eigenes Projekt“.
pub fn key(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// `#ohne` ist kein Thema, sondern heißt überall „kein Thema“: `fokus #ohne`
/// zeigt die Aufgaben ohne Thema, `fokus ändern 3 #ohne` nimmt es weg. Ein
/// Thema „Ohne“ legt Fokus deshalb nie an.
pub const NO_TOPIC: &str = "ohne";

/// Ist das Wort `#ohne`?
pub fn is_no_topic(word: &str) -> bool {
    word.starts_with('#') && key(clean(word)) == NO_TOPIC
}

/// Der Text ohne `#ohne`.
pub fn without_no_topic(text: &str) -> String {
    if !text.split_whitespace().any(is_no_topic) {
        return text.to_string();
    }
    text.split_whitespace()
        .filter(|w| !is_no_topic(w))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Ist das Wort ein Thema (`#`) bzw. eine Person (`@`)? `#ohne` ist keins.
pub fn is_tag(word: &str, sigil: char) -> bool {
    word.starts_with(sigil) && !key(clean(word)).is_empty() && !(sigil == '#' && is_no_topic(word))
}

/// Anzeigename: `#eigenes-projekt` → „Eigenes projekt“, `@anna` → „Anna“.
pub fn display(tag: &str) -> String {
    let s = clean(tag).replace(['-', '_'], " ");
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// So zeigt Fokus ein Thema: mit `#`, so wie man es tippt. „Eigenes Projekt“ → `#eigenes-projekt`.
pub fn topic_label(name: &str) -> String {
    format!("#{}", to_tag(&display(name)))
}

/// So zeigt Fokus eine Person: mit `@`, so wie man sie tippt. „Anna“ → `@anna`.
pub fn person_label(raw: &str) -> String {
    format!("@{}", clean(raw).to_lowercase())
}

/// Aus einem Themennamen das Wort für den Aufgabentext: „Eigenes Projekt“ → `eigenes-projekt`.
pub fn to_tag(name: &str) -> String {
    name.split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn themen_und_personen_mit_vorzeichen() {
        assert_eq!(topic_label("Eigenes Projekt"), "#eigenes-projekt");
        assert_eq!(topic_label("arbeit"), "#arbeit");
        assert_eq!(person_label("Anna"), "@anna");
        assert_eq!(person_label("@tom,"), "@tom");
    }

    #[test]
    fn satzzeichen_gehoeren_nicht_zum_namen() {
        assert_eq!(clean("#arbeit,"), "arbeit");
        assert_eq!(clean("@anna!"), "anna");
    }

    #[test]
    fn bindestrich_und_leerzeichen_sind_dasselbe_thema() {
        assert_eq!(key("eigenes-projekt"), key("Eigenes Projekt"));
        assert_eq!(key("Gesundheit"), "gesundheit");
    }

    #[test]
    fn ohne_ist_kein_thema() {
        assert!(is_no_topic("#ohne"));
        assert!(is_no_topic("#Ohne,"));
        assert!(!is_tag("#ohne", '#'));
        assert!(is_tag("@ohne", '@'));
        assert!(!is_no_topic("ohne"));
        assert!(!is_no_topic("#ohnegleichen"));
        assert_eq!(without_no_topic("Brot #ohne kaufen"), "Brot kaufen");
    }

    #[test]
    fn ein_einzelnes_zeichen_ist_kein_tag() {
        assert!(!is_tag("#", '#'));
        assert!(!is_tag("@", '@'));
        assert!(is_tag("#haus", '#'));
        assert!(!is_tag("anna@mail.de", '@'));
    }

    #[test]
    fn anzeigename_beginnt_gross() {
        assert_eq!(display("#eigenes-projekt"), "Eigenes projekt");
        assert_eq!(display("@änne"), "Änne");
        assert_eq!(to_tag("Eigenes Projekt"), "eigenes-projekt");
    }
}
