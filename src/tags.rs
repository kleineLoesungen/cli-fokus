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

/// Ist das Wort ein Thema (`#`) bzw. eine Person (`@`)?
pub fn is_tag(word: &str, sigil: char) -> bool {
    word.starts_with(sigil) && !key(clean(word)).is_empty()
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
