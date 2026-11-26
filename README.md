# Fokus

**Weniger Liste, mehr erledigt.** Ein kleiner Helfer fürs Terminal auf dem Mac,
der jeden Morgen eine Frage beantwortet: *Was ist heute dran?*

## Die Idee

* Du schreibst Aufgaben auf, wie sie dir einfallen: ein Satz, fertig.
* Fokus erkennt selbst, was **dringend** ist (Termin bis übermorgen oder
  verstrichen) und was **wichtig** ist (Thema über der Linie, siehe unten).
* Auf **heute** stehen höchstens **drei** offene Aufgaben, und **wichtig**
  sind höchstens **drei** Themen. Mehr nicht, damit Wählen nötig bleibt.

Du vergibst keine Prioritäten und schätzt keine Stunden.

---

## Typische Situationen

### Morgens: Was ist heute dran?

```
fokus
```
```
 Donnerstag, 26. November                                  1 von 3 erledigt
 ──────────────────────────────────────────────────────────────────────────

 HEUTE
     ✓ Angebot an Kunden schicken                                 Arbeit
   2 ▶ Präsentation vorbereiten                    fällig morgen  Arbeit
   4 · Laufschuhe kaufen                                          Gesundheit

 DRINGEND, ABER NICHT WICHTIG   → schnell erledigen oder abgeben
   6 · Reisekosten abrechnen                       fällig morgen  Haus

 4 weitere offen · 2 warten auf andere
 2 liegen schon lange → fokus aufräumen
```

Die Zahl ist die **Nummer** der Aufgabe, ▶ zeigt, womit du anfängst. Morgens
stellt Fokus die Liste selbst zusammen (dringend und wichtig zuerst); tauschen
kannst du jederzeit.

```
fokus heute 5            Aufgabe 5 auf heute
fokus heute 5 statt 2    tauschen, wenn heute voll ist
fokus später 2           wieder von heute herunter
fokus fertig 2           abhaken – dann ist wieder ein Platz frei
```

### Dir fällt etwas ein

```
fokus + Präsentation vorbereiten #arbeit morgen
```
```
 ✓ Aufgabe 1: Präsentation vorbereiten · Arbeit · fällig morgen (Freitag 27.11.)
```

`#thema` und `@person` sind freiwillig, der Termin auch. Er steht **am Ende**
(`heute`, `morgen`, `freitag`, `3.12.`, `in 2 wochen`, `nächste woche`). Ist
etwas **heute** fällig, kommt es gleich auf die Liste für heute, solange dort
Platz ist.

Bei Zeichen wie `'` oder `?` im Text: nur `fokus +` tippen, dann fragt Fokus.

### Etwas hat sich geändert

```
fokus ändern 4 #haus @tom freitag     Thema, Person, Termin
fokus ändern 4 Keller und Dachboden   neuer Text, der Rest bleibt
fokus ändern 4 ohne termin            auch: ohne thema, ohne @anna
```

### Jetzt loslegen

```
fokus jetzt
```

Der Bildschirm zeigt nur noch die Aufgabe mit ▶ und eine 25-Minuten-Uhr.
`f` fertig, `p` Pause, `s` später, `q` zurück.

### Der Überblick über alles

```
fokus alles
```
```
                       DRINGEND                    NICHT DRINGEND
           ┌──────────────────────────────┬──────────────────────────────┐
   WICHTIG │  2 ▶ Präsentation vo… morgen │  5   Website-Texte    12.12. │
           │                              │  3   Urlaub abstimme…        │
           │                              │  4 ▶ Laufschuhe kauf…        │
           │                          TUN │                       PLANEN │
           ├──────────────────────────────┼──────────────────────────────┤
 UNWICHTIG │  6   Reisekosten abr… morgen │  7   Keller ausmist… 18 Tage │
           │                              │ 10   Mama zurückruf…         │
           │         SCHNELL ODER ABGEBEN │                   STREICHEN? │
           └──────────────────────────────┴──────────────────────────────┘
```

**Planen** ist die wertvollste Ecke: Das drängelt nie und geht darum am
leichtesten unter. Was unter **Streichen?** 30 Tage ohne Termin liegt, räumt
Fokus selbst ins Archiv.

### Woran hänge ich in einem Thema?

```
fokus #arbeit
```
```
 ARBEIT   wichtig
   2 ▶ Präsentation vorbereiten                 fällig morgen
   5 · Website-Texte                            fällig 12.12.

 LIEGT BEI ANDEREN
   8 · Vertrag prüfen                                 bei Tom  seit 12 Tagen

 In den letzten 7 Tagen erledigt: Angebot an Kunden schicken
```

### Komme ich bei dem voran, was mir wichtig ist?

```
fokus themen
```
```
 DEINE THEMEN   offen    letzte 7 Tage
 1  Arbeit          3    ○ ○ ○ ○ ○ ○ ●
 2  Familie         2    ○ ○ ○ ○ ○ ○ ○
 3  Gesundheit      1    ○ ○ ○ ○ ○ ○ ○
 ─── wichtig ───────────────────────────────
    Haus            2    ○ ○ ○ ○ ○ ○ ○
    ohne Thema      1
```

Ein Punkt je Tag, rechts ist heute. Über der Linie stehen höchstens drei
Themen; die ersten drei, die du anlegst, landen dort von selbst.

```
fokus themen ordnen                    fragt, welche drei wichtig sind
fokus themen umbenennen arbiet arbeit  überall; gibt es „arbeit“ schon, wird es eins
```

### Du triffst Anna

```
fokus @anna
```
```
 ANNA
   3 · Urlaub abstimmen @anna                                   Familie

 WARTE AUF ANNA
   9 · Fotos vom Wochenende schicken                            seit 5 Tagen
```

`fokus personen` listet alle, bei denen gerade etwas offen ist.

### Worauf warte ich noch?

```
fokus abgeben 8 @tom     Aufgabe 8 liegt jetzt bei Tom
fokus warten
```
```
 WARTE AUF   → nachhaken: fokus aufräumen
   8 · Vertrag prüfen                                 bei Tom  seit 12 Tagen
   9 · Fotos vom Wochenende schicken                 bei Anna  seit 5 Tagen
```

Das am längsten Liegende steht oben. Kommt etwas erledigt zurück:
`fokus fertig 8`.

### Einmal pro Woche: aufräumen

```
fokus aufräumen
```

Fokus geht alles durch, was seit zwei Wochen liegt (oder seit einer Woche bei
jemand anderem). Eine Taste je Aufgabe:

```
 Keller ausmisten · Haus · seit 18 Tagen
 [h] heute  [w] diese Woche  [t] anderes Thema  [b] behalten  [f] erledigt  [x] streichen
```

Bei Abgegebenem gibt es zusätzlich `[n] nachhaken` (legt „Bei Tom nachhaken: …“
auf heute) und `[z] zurück zu mir`.

---

## Alle Befehle

| Befehl | |
|---|---|
| `fokus` | was heute dran ist |
| `fokus + Text #thema @person termin` | aufschreiben |
| `fokus heute 4` · `später 4` · `fertig 4` | Liste für heute |
| `fokus ändern 4 …` | Text, Thema, Person, Termin ändern |
| `fokus streichen 4` | weg damit (bleibt im Archiv der Datei) |
| `fokus jetzt` | nur eine Aufgabe, mit Uhr |
| `fokus alles` | die vier Felder |
| `fokus #arbeit` · `fokus themen` | ein Thema · alle Themen |
| `fokus themen ordnen` · `umbenennen a b` | Themen pflegen |
| `fokus @anna` · `fokus personen` | eine Person · alle Personen |
| `fokus abgeben 4 @tom` · `fokus warten` | abgeben · was bei anderen liegt |
| `fokus aufräumen` | Liegengebliebenes durchgehen |
| `fokus datei` | die Aufgabendatei im Texteditor öffnen |
| `fokus hilfe` | diese Liste |

Statt `fokus` geht auch `f` (siehe Einrichten). Die **Tab-Taste** ergänzt
Befehle, Nummern samt Text, `#themen`, `@personen` und Termine.

---

## Deine Daten

Alles steht in einer Textdatei: `Dokumente/Fokus.md`. Du kannst sie mit jedem
Editor lesen und ändern (`fokus datei`); eine neue Zeile `- Fahrrad
reparieren #haus` reicht, den Rest ergänzt Fokus. Kein Konto, keine Cloud –
außer dein Mac sichert den Ordner Dokumente in iCloud.

## Was Fokus absichtlich nicht kann

Keine Prioritätsstufen, keine Unteraufgaben, kein Adressbuch, keine
Mitteilungen, kein Teilen. Und kein viertes wichtiges Thema. Alles davon wäre
etwas, das gepflegt werden will.

---

## Installieren, aktualisieren, entfernen

Einmalig brauchst du [Rust](https://rustup.rs). Dann im Projektordner:

```bash
cargo install --path .
```

```bash
fokus einrichten
```

`fokus einrichten` fragt, ob `f` als Kurzbefehl gehen soll, ob jedes neue
Terminalfenster die Liste für heute zeigt und ob die Tab-Taste ergänzen soll
(zsh). Es schreibt dafür einen markierten Block in deine `.zshrc`.

**Aktualisieren:** `cargo install --path .` noch einmal. Deine Aufgaben bleiben.

**Entfernen:** `fokus deinstallieren`. Es zeigt erst, was es tut, und fragt
nach. Deine Aufgaben bleiben, außer du sagst ausdrücklich ja – dann kommen sie
in den Papierkorb.

Zum Ausprobieren mit einer Spieldatei: `export FOKUS_DATEI=/tmp/spiel.md`.
Tests: `cargo test`.

## Lizenz

GPL-3.0, siehe [LICENSE](LICENSE).
