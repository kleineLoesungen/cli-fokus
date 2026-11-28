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

Was regelmäßig ansteht, bekommt eine **Wiederholung** am Ende:

```
fokus + Wochenplan schreiben #arbeit jeden montag
fokus + Pflanzen gießen alle 2 wochen am freitag
```

Hakst du sie ab, legt Fokus gleich die nächste an. Verpasst du einen Termin,
rückt sie zum nächsten weiter, statt sich zu stapeln. Ändern geht mit
`fokus ändern 4 alle 4 wochen montag`, beenden mit `ohne wiederholung` oder
`fokus streichen 4`.

Bei Zeichen wie `'` oder `?` im Text: nur `fokus +` tippen, dann fragt Fokus.

### Mehr dazu merken: Notizen und Links

```
fokus notiz 5 https://figma.com/file/abc
fokus notiz 5 Ansprechpartner Max, Budget steht im Angebot
fokus 5
```
```
 5  Website-Relaunch
     Arbeit · fällig 12.12. · seit heute

   5.1  https://figma.com/file/abc
   5.2  Ansprechpartner Max, Budget steht im Angebot
```

Eine Notiz ist eine Zeile Text oder ein Link. `fokus notiz 5.2 …` ersetzt
die zweite, `fokus notiz 5.2 löschen` löscht sie. `fokus notiz 5` allein geht
die Notizen durch: Ziffer wählt eine, dann [ä] ändern oder [l] löschen, [n]
legt eine neue an. Die Tab-Taste zeigt nach `fokus notiz 5.` die Notizen mit
Text, du musst dir die Nummern also nicht merken. `fokus öffnen 5` öffnet den
Link im Browser (bei mehreren fragt Fokus, welchen).

Themen haben auch Notizen: `fokus notiz #arbeit Wiki: https://…`. Sie stehen
dann in `fokus #arbeit` unten.

In den Listen siehst du nur ein Zeichen hinter dem Text, ↗ bei einem Link,
✎ bei anderen Notizen. In der Datei stehen die Notizen eingerückt unter ihrer
Aufgabe oder ihrem Thema, in der Unterwegs-Datei auch. Sie bleiben, wenn du die
Aufgabe änderst, auf heute legst oder archivierst, und kommen bei einer
Wiederholung mit.

### Etwas hat sich geändert

```
fokus ändern 4 #haus @tom freitag     Thema, Person, Termin
fokus ändern 4 Keller und Dachboden   neuer Text, der Rest bleibt
fokus ändern 4 ohne termin            auch: ohne thema (#ohne), ohne @anna, ohne wiederholung
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

### Versehentlich abgehakt?

```
fokus archiv
```
```
 ERLEDIGT
   2 ✓ Pflanzen gießen                                             heute  Haus

 GESTRICHEN
   3 × Reisekosten abrechnen                                       gestern

 Zurückholen: fokus zurück <Nr> · nach 14 Tagen löscht Fokus endgültig
```

`fokus zurück 2` holt die Aufgabe zurück, auf heute, wenn sie dort stand.
Erledigtes und Gestrichenes behält seine Nummer, solange es im Archiv steht,
und ist nach **14 Tagen endgültig weg**.

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

Was noch kein Thema hat, zeigt `fokus #ohne`. Ein Thema gibst du mit
`fokus ändern 3 #haus`; `fokus ändern 3 #ohne` nimmt es wieder weg. `#ohne`
heißt überall „kein Thema“, deshalb legt Fokus nie ein Thema „Ohne“ an.

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

### Unterwegs: nachsehen und aufschreiben

```
fokus unterwegs ~/Dropbox/Fokus
```

Ab jetzt schreibt Fokus bei jedem Aufruf den Stand in eine zweite Datei,
`Fokus-unterwegs.md`, in diesem Ordner. Welcher Dienst den Ordner zum Handy
bringt, ist dir überlassen: iCloud Drive, Google Drive, Dropbox, OneDrive,
Nextcloud, Syncthing – Fokus schreibt nur eine Textdatei. Ein Android-Handy
kommt nicht an iCloud Drive; für iPhone und Android zusammen eignet sich ein
anderer Dienst. Lieber `.txt`? `fokus unterwegs ~/Dropbox/Fokus/Fokus.txt`.

```
# Fokus unterwegs

Stand: Freitag 27.11., 06:10

Neues schreibst du ganz unten unter „Neu“ …

## Heute · Freitag 27.11.
- 2 Präsentation vorbereiten · Arbeit · fällig Samstag 28.11.

## Warte auf
- 8 Vertrag prüfen · bei Tom seit 12.11.

## Offen

### Arbeit · wichtig
- 5 Website-Texte · fällig Samstag 12.12.

## Neu
- Trikots bestellen #verein freitag
Idee für den Talentpfad
```

Oben steht der Stand zum Lesen, mit Datum statt „morgen“, weil man ihn
vielleicht erst später liest. Was dir unterwegs einfällt, schreibst du ganz
unten unter **Neu**, eine Zeile je Aufgabe, wie bei `fokus +`. Beim nächsten
Aufruf holt Fokus die Zeilen ab, legt sie als Aufgaben an und leert den
Abschnitt. Alles darüber schreibt Fokus jedes Mal neu; abhaken und ändern geht
am Computer.

Weil „Neu“ am Ende steht, reicht es, eine Zeile an die Datei anzuhängen. Das
können Kurzbefehle auf dem iPhone und Automatisierungs-Apps unter Android, also
auch per Sprache, ohne die Datei zu öffnen. Zum Lesen und Schreiben von Hand
brauchst du am Handy einen Texteditor, der Dateien direkt im Sync-Ordner öffnet
(auf dem iPhone etwa Runestone); die Dateien-Apps zeigen Textdateien nur an.

Deine `Fokus.md` bleibt, wo sie ist: Die Unterwegs-Datei ist nur eine Kopie.
`fokus unterwegs` zeigt, wo sie liegt, `fokus unterwegs aus` schaltet sie ab.

### Einmal pro Woche: aufräumen

```
fokus aufräumen
```

Fokus geht durch, was **überfällig** ist (ab dem Tag nach dem Termin) und was
seit zwei Wochen liegt (oder seit einer Woche bei jemand anderem). Eine Taste
je Aufgabe:

```
 Steuer abgeben · kein Thema · seit gestern fällig · seit gestern
 [h] heute  [n] neuer Termin  [w] diese Woche  [t] Thema geben
 [b] behalten  [f] erledigt  [x] streichen
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
| `fokus 4` · `notiz 4 …` · `notiz 4.2 …` · `öffnen 4` | eine Aufgabe mit Notizen · Notiz dazu · ändern · Link öffnen |
| `fokus streichen 4` | weg damit (14 Tage im Archiv) |
| `fokus archiv` · `fokus zurück 4` | Erledigtes ansehen · zurückholen |
| `fokus jetzt` | nur eine Aufgabe, mit Uhr |
| `fokus alles` | die vier Felder |
| `fokus #arbeit` · `fokus themen` | ein Thema · alle Themen |
| `fokus #ohne` | Aufgaben ohne Thema |
| `fokus themen ordnen` · `umbenennen a b` | Themen pflegen |
| `fokus @anna` · `fokus personen` | eine Person · alle Personen |
| `fokus abgeben 4 @tom` · `fokus warten` | abgeben · was bei anderen liegt |
| `fokus aufräumen` | Liegengebliebenes durchgehen |
| `fokus datei` | die Aufgabendatei im Texteditor öffnen |
| `fokus datei verschieben ort` | die Aufgabendatei an einen anderen Ort legen |
| `fokus unterwegs ordner` · `unterwegs aus` | Stand fürs Handy, mit „Neu“ zum Aufschreiben |
| `fokus hilfe` | die kurze Übersicht |
| `fokus anleitung` · `anleitung unterwegs` | diese Anleitung, oder nur der passende Abschnitt |

Statt `fokus` geht auch `f` (siehe Einrichten). Die **Tab-Taste** ergänzt
Befehle, Nummern samt Text, `#themen`, `@personen`, Termine und Wiederholungen
– zweimal Tab zeigt die Liste, nach Gruppen geordnet:

```
fokus fertig ⇥⇥
Heute
7   Reisekosten  · heute
1   Präsentation vorbereiten  · morgen
Arbeit · wichtig
2   Website-Texte  · 12.12.
Ohne Thema
5   Mama zurückrufen
Bei anderen
6   Fotos schicken  · bei Anna
```

---

## Deine Daten

Alles steht in einer Textdatei, anfangs `Dokumente/Fokus.md`. Du kannst sie mit
jedem Editor lesen und ändern (`fokus datei`); eine neue Zeile `- Fahrrad
reparieren #haus` reicht, den Rest ergänzt Fokus. Eingerückte Zeilen darunter
sind Notizen zu dieser Aufgabe. Erledigtes und Gestrichenes
löscht Fokus nach 14 Tagen. Kein Konto, keine Cloud – außer dein Mac sichert
den Ordner Dokumente in iCloud, oder du legst die Datei selbst in einen
Sync-Ordner oder schaltest die Unterwegs-Datei ein.

Woanders hin geht sie mit `fokus datei verschieben ~/Planung` (ein Ordner
bekommt `Fokus.md`, eine `.md`-Datei bleibt, wie sie heißt). Fokus zieht die
Datei um und merkt sich den Ort. Liegt dort schon eine Aufgabendatei, fragt
Fokus, ob es die nehmen soll, und überschreibt nichts. Die Orte stehen in
`~/.config/fokus/einstellungen`; `fokus einrichten` fragt auch danach.

## Was Fokus absichtlich nicht kann

Keine Prioritätsstufen, keine Unteraufgaben, kein Adressbuch, keine
Mitteilungen, kein Teilen mit anderen (die Unterwegs-Datei ist für dich). Und kein viertes wichtiges Thema. Alles davon wäre
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

`fokus einrichten` fragt, wo deine Aufgaben liegen sollen, ob es eine
Unterwegs-Datei geben soll, ob `f` als Kurzbefehl gehen soll, ob jedes neue
Terminalfenster die Liste für heute zeigt und ob die Tab-Taste ergänzen soll
(zsh). Es schreibt dafür einen markierten Block in deine `.zshrc` und die Orte
in `~/.config/fokus/einstellungen`. In zsh stehen `fokus` und `f` dort mit
`noglob`, damit Links mit `?` oder `*` nicht an zsh scheitern („no matches
found“). Einen Ordner kannst du aus dem Finder ins
Terminal ziehen, statt den Pfad zu tippen.

**Aktualisieren:** `cargo install --path .` noch einmal. Deine Aufgaben bleiben.

**Entfernen:** `fokus deinstallieren`. Es zeigt erst, was es tut, und fragt
nach. Deine Aufgaben bleiben, außer du sagst ausdrücklich ja – dann kommen sie
in den Papierkorb. Die Einstellungen löscht es mit, die Unterwegs-Datei nicht.

Zum Ausprobieren mit einer Spieldatei: `export FOKUS_DATEI=/tmp/spiel.md`.
Solange die gilt, lässt Fokus deine Unterwegs-Datei in Ruhe (außer du setzt
auch `FOKUS_UNTERWEGS`).
Tests: `cargo test`.

## Lizenz

GPL-3.0, siehe [LICENSE](LICENSE).
