# Focus

*[English](README.md) · Deutsch*

**Weniger Liste, mehr erledigt.** Ein kleiner Helfer fürs Terminal auf dem Mac,
der jeden Morgen eine Frage beantwortet: *Was ist heute dran?*

## Die Idee

* Du schreibst Aufgaben auf, wie sie dir einfallen: ein Satz, fertig.
* Focus erkennt selbst, was **dringend** ist (Termin bis übermorgen oder
  verstrichen) und was **wichtig** ist (Thema über der Linie, siehe unten).
* Auf **heute** stehen höchstens **drei** offene Aufgaben, und **wichtig**
  sind höchstens **drei** Themen. Mehr nicht, damit Wählen nötig bleibt. (Beide
  Zahlen lassen sich in `focus einrichten` ändern, von 1 bis 9.)

Du vergibst keine Prioritäten und schätzt keine Stunden.

---

## Typische Situationen

### Morgens: Was ist heute dran?

```
focus
```
```
 Donnerstag, 26. November                                  1 von 3 erledigt
 ──────────────────────────────────────────────────────────────────────────

 HEUTE
     ✓ Angebot an Kunden schicken                                 #arbeit
   2 ▶ Präsentation vorbereiten                    fällig morgen  #arbeit
   4 · Laufschuhe kaufen                                          #gesundheit

 DRINGEND, ABER NICHT WICHTIG   → schnell erledigen oder abgeben
   6 · Reisekosten abrechnen                       fällig morgen  #haus

 4 weitere offen · 2 warten auf andere
 2 liegen schon lange → focus aufräumen
```

Die Zahl ist die **Nummer** der Aufgabe, ▶ zeigt, womit du anfängst. Morgens
stellt Focus die Liste selbst zusammen (dringend und wichtig zuerst); tauschen
kannst du jederzeit.

```
focus heute 5            Aufgabe 5 auf heute
focus heute 5 statt 2    tauschen, wenn heute voll ist
focus später 2           wieder von heute herunter
focus fertig 2           abhaken – dann ist wieder ein Platz frei
```

Bevor eine Aufgabe auf heute kommt, prüft Focus, ob Wichtigeres offen ist. Die
Rangfolge: wichtig und dringend, wichtig, dringend, nichts davon. Steht etwas
weiter oben, nennt Focus die Anzahl, zeigt bis zu drei davon und fragt, ob
diese Aufgabe trotzdem deinem Fokus entspricht:

```
 Aufgabe 4 ist weder wichtig noch dringend.
 Es gibt 3 wichtigere Aufgaben (1 wichtig und dringend, 1 wichtig, 1 dringend):
     1  Steuer abgeben · #arbeit · fällig morgen (Freitag 27.11.)
     2  Website-Texte · #arbeit
     3  Reisekosten abrechnen · ohne Thema · fällig morgen (Freitag 27.11.)
 Entspricht Aufgabe 4 trotzdem deinem Fokus für heute? [j/N]
```

Enter heißt nein. Was im selben Befehl mitkommt (`focus heute 1 2 4`), zählt
nicht als wichtiger. Ohne Terminal zum Fragen, etwa im Browser, legt Focus die
Aufgabe trotzdem auf heute und nennt die Nummern als Hinweis.

### Dir fällt etwas ein

```
focus + Präsentation vorbereiten #arbeit morgen
```
```
 ✓ Aufgabe 1: Präsentation vorbereiten · #arbeit · fällig morgen (Freitag 27.11.)
```

`#thema` und `@person` sind freiwillig, der Termin auch. Er steht **am Ende**
(`heute`, `morgen`, `freitag`, `3.12.`, `in 2 wochen`, `nächste woche`). Ist
etwas **heute** fällig, kommt es gleich auf die Liste für heute, solange dort
Platz ist.

Was regelmäßig ansteht, bekommt eine **Wiederholung** am Ende:

```
focus + Wochenplan schreiben #arbeit jeden montag
focus + Pflanzen gießen alle 2 wochen am freitag
```

Hakst du sie ab, legt Focus gleich die nächste an. Verpasst du einen Termin,
rückt sie zum nächsten weiter, statt sich zu stapeln. Ändern geht mit
`focus ändern 4 alle 4 wochen montag`, beenden mit `ohne wiederholung` oder
`focus streichen 4`.

Bei Zeichen wie `'` oder `?` im Text: nur `focus +` tippen, dann fragt Focus.

### Mehr dazu merken: Notizen und Links

```
focus notiz 5 https://figma.com/file/abc
focus notiz 5 Ansprechpartner Max, Budget steht im Angebot
focus 5
```
```
 5  Website-Relaunch
     #arbeit · fällig 12.12. · seit heute

   5.1  https://figma.com/file/abc
   5.2  Ansprechpartner Max, Budget steht im Angebot
```

Eine Notiz ist eine Zeile Text oder ein Link. `focus notiz 5.2 …` ersetzt
die zweite, `focus notiz 5.2 löschen` löscht sie. `focus notiz 5` allein geht
die Notizen durch: Ziffer wählt eine, dann [ä] ändern oder [l] löschen, [n]
legt eine neue an. Die Tab-Taste zeigt nach `focus notiz 5.` die Notizen mit
Text, du musst dir die Nummern also nicht merken. `focus öffnen 5` öffnet den
Link im Browser (bei mehreren fragt Focus, welchen).

Themen haben auch Notizen: `focus notiz #arbeit Wiki: https://…`. Sie stehen
dann in `focus #arbeit` unten.

In den Listen siehst du nur ein Zeichen hinter dem Text, ↗ bei einem Link,
✎ bei anderen Notizen. In der Datei stehen die Notizen eingerückt unter ihrer
Aufgabe oder ihrem Thema, in der Unterwegs-Datei auch. Sie bleiben, wenn du die
Aufgabe änderst, auf heute legst oder archivierst, und kommen bei einer
Wiederholung mit.

### Etwas hat sich geändert

```
focus ändern 4 #haus @tom freitag     Thema, Person, Termin
focus ändern 4 Keller und Dachboden   neuer Text, der Rest bleibt
focus ändern 4 ohne termin            auch: ohne thema (#ohne), ohne @anna, ohne wiederholung
```

### Jetzt loslegen

```
focus jetzt
```

Der Bildschirm zeigt nur noch die Aufgabe mit ▶ und eine 25-Minuten-Uhr.
`f` fertig, `p` Pause, `s` später, `q` zurück.

### Der Überblick über alles

```
focus alles
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
Focus selbst ins Archiv.

### Versehentlich abgehakt?

```
focus archiv
```
```
 ERLEDIGT
   2 ✓ Pflanzen gießen                                             heute  #haus

 GESTRICHEN
   3 × Reisekosten abrechnen                                       gestern

 Zurückholen: focus zurück <Nr> · nach 14 Tagen löscht Focus endgültig
```

`focus zurück 2` holt die Aufgabe zurück, auf heute, wenn sie dort stand.
Erledigtes und Gestrichenes behält seine Nummer, solange es im Archiv steht,
und ist nach **14 Tagen endgültig weg**.

### Woran hänge ich in einem Thema?

```
focus #arbeit
```
```
 #arbeit   wichtig
   2 ▶ Präsentation vorbereiten                 fällig morgen
   5 · Website-Texte                            fällig 12.12.

 LIEGT BEI ANDEREN
   8 · Vertrag prüfen                                 bei @tom  seit 12 Tagen

 In den letzten 7 Tagen erledigt: Angebot an Kunden schicken
```

Was noch kein Thema hat, zeigt `focus #ohne`. Ein Thema gibst du mit
`focus ändern 3 #haus`; `focus ändern 3 #ohne` nimmt es wieder weg. `#ohne`
heißt überall „kein Thema“, deshalb legt Focus nie ein Thema „Ohne“ an.

### Komme ich bei dem voran, was mir wichtig ist?

```
focus themen
```
```
 DEINE THEMEN    offen    letzte 7 Tage
 1  #arbeit          3    ○ ○ ○ ○ ○ ○ ●
 2  #familie         2    ○ ○ ○ ○ ○ ○ ○
 3  #gesundheit      1    ○ ○ ○ ○ ○ ○ ○
 ─── wichtig ────────────────────────────────
    #haus            2    ○ ○ ○ ○ ○ ○ ○
    #ohne            1    ohne Thema
```

Ein Punkt je Tag, rechts ist heute. Über der Linie stehen höchstens drei
Themen (oder so viele, wie du in `focus einrichten` eingestellt hast); die
ersten, die du anlegst, landen dort von selbst.

```
focus themen ordnen                    fragt, welche wichtig sind
focus themen umbenennen arbiet arbeit  überall; gibt es „arbeit“ schon, wird es eins
```

### Du triffst Anna

```
focus @anna
```
```
 @anna
   3 · Urlaub abstimmen @anna                                   #familie

 WARTE AUF @anna
   9 · Fotos vom Wochenende schicken                            seit 5 Tagen
```

`focus personen` listet alle, bei denen gerade etwas offen ist.

### Worauf warte ich noch?

```
focus abgeben 8 @tom     Aufgabe 8 liegt jetzt bei @tom
focus warten
```
```
 WARTE AUF   → nachhaken: focus aufräumen
   8 · Vertrag prüfen                                 bei @tom  seit 12 Tagen
   9 · Fotos vom Wochenende schicken                 bei @anna  seit 5 Tagen
```

Das am längsten Liegende steht oben. Kommt etwas erledigt zurück:
`focus fertig 8`.

### Wo war das noch?

```
focus suche angebot
```
```
 GEFUNDEN: ANGEBOT
   4 · Angebot prüfen ✎                               fällig Freitag  #arbeit
```

Focus sucht in Text, Thema, Personen und Notizen. Mehrere Wörter müssen alle
vorkommen, Groß- und Kleinschreibung ist egal, und `pruefen` findet auch
`prüfen`. Was im Archiv passt, steht blass darunter.

### Unterwegs: nachsehen und aufschreiben

```
focus unterwegs ~/Dropbox/Focus
```

Ab jetzt schreibt Focus bei jedem Aufruf den Stand in eine zweite Datei,
`Focus-unterwegs.md`, in diesem Ordner. Welcher Dienst den Ordner zum Handy
bringt, ist dir überlassen: iCloud Drive, Google Drive, Dropbox, OneDrive,
Nextcloud, Syncthing – Focus schreibt nur eine Textdatei. Ein Android-Handy
kommt nicht an iCloud Drive; für iPhone und Android zusammen eignet sich ein
anderer Dienst. Lieber `.txt`? `focus unterwegs ~/Dropbox/Focus/Focus.txt`.

```
# Focus unterwegs

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
unten unter **Neu**, eine Zeile je Aufgabe, wie bei `focus +`. Beim nächsten
Aufruf holt Focus die Zeilen ab, legt sie als Aufgaben an und leert den
Abschnitt. Alles darüber schreibt Focus jedes Mal neu; abhaken und ändern geht
am Computer.

Weil „Neu“ am Ende steht, reicht es, eine Zeile an die Datei anzuhängen. Das
können Kurzbefehle auf dem iPhone und Automatisierungs-Apps unter Android, also
auch per Sprache, ohne die Datei zu öffnen. Zum Lesen und Schreiben von Hand
brauchst du am Handy einen Texteditor, der Dateien direkt im Sync-Ordner öffnet
(auf dem iPhone etwa Runestone); die Dateien-Apps zeigen Textdateien nur an.

Deine `Focus.md` bleibt, wo sie ist: Die Unterwegs-Datei ist nur eine Kopie.
`focus unterwegs` zeigt, wo sie liegt, `focus unterwegs aus` schaltet sie ab.

### Einmal pro Woche: aufräumen

```
focus aufräumen
```

Focus geht durch, was **überfällig** ist (ab dem Tag nach dem Termin) und was
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
| `focus` | was heute dran ist |
| `focus + Text #thema @person termin` | aufschreiben |
| `focus heute 4` · `später 4` · `fertig 4` | Liste für heute |
| `focus ändern 4 …` | Text, Thema, Person, Termin ändern |
| `focus 4` · `notiz 4 …` · `notiz 4.2 …` · `öffnen 4` | eine Aufgabe mit Notizen · Notiz dazu · ändern · Link öffnen |
| `focus streichen 4` | weg damit (14 Tage im Archiv) |
| `focus archiv` · `focus zurück 4` | Erledigtes ansehen · zurückholen |
| `focus jetzt` | nur eine Aufgabe, mit Uhr |
| `focus alles` | die vier Felder |
| `focus #arbeit` · `focus themen` | ein Thema · alle Themen |
| `focus #ohne` | Aufgaben ohne Thema |
| `focus themen ordnen` · `umbenennen a b` | Themen pflegen |
| `focus @anna` · `focus personen` | eine Person · alle Personen |
| `focus abgeben 4 @tom` · `focus warten` | abgeben · was bei anderen liegt |
| `focus suche wort …` | Aufgaben finden, auch in Notizen und im Archiv |
| `focus aufräumen` | Liegengebliebenes durchgehen |
| `focus datei` | die Aufgabendatei im Texteditor öffnen |
| `focus datei verschieben ort` | die Aufgabendatei an einen anderen Ort legen |
| `focus unterwegs ordner` · `unterwegs aus` | Stand fürs Handy, mit „Neu“ zum Aufschreiben |
| `focus web` | Focus im Browser-Tab |
| `focus hilfe` | die kurze Übersicht |
| `focus anleitung` · `anleitung unterwegs` | diese Anleitung, oder nur der passende Abschnitt |

Englische Befehle gehen immer mit (`focus today 4`, `focus done 4`, `tomorrow`,
`every monday`), auch wenn Focus Deutsch spricht.

Statt `focus` geht auch `f` (siehe Einrichten). Die **Tab-Taste** ergänzt
Befehle, Nummern samt Text, `#themen`, `@personen`, Termine und Wiederholungen
– zweimal Tab zeigt die Liste, nach Gruppen geordnet:

```
focus fertig ⇥⇥
Heute
7   Reisekosten  · heute
1   Präsentation vorbereiten  · morgen
#arbeit · wichtig
2   Website-Texte  · 12.12.
Ohne Thema
5   Mama zurückrufen
Bei anderen
6   Fotos schicken  · bei @anna
```

---

## Deine Daten

Alles steht in einer Textdatei, anfangs `Dokumente/Focus.md`. Du kannst sie mit
jedem Editor lesen und ändern (`focus datei`); eine neue Zeile `- Fahrrad
reparieren #haus` reicht, den Rest ergänzt Focus. Eingerückte Zeilen darunter
sind Notizen zu dieser Aufgabe. Erledigtes und Gestrichenes
löscht Focus nach 14 Tagen. Kein Konto, keine Cloud – außer dein Mac sichert
den Ordner Dokumente in iCloud, oder du legst die Datei selbst in einen
Sync-Ordner oder schaltest die Unterwegs-Datei ein.

Woanders hin geht sie mit `focus datei verschieben ~/Planung` (ein Ordner
bekommt `Focus.md`, eine `.md`-Datei bleibt, wie sie heißt). Focus zieht die
Datei um und merkt sich den Ort. Liegt dort schon eine Aufgabendatei, fragt
Focus, ob es die nehmen soll, und überschreibt nichts. Die Orte stehen in
`~/.config/focus/einstellungen`; `focus einrichten` fragt auch danach.

## Im Browser: `focus web`

Wer ohnehin viel im Browser arbeitet: `focus web` öffnet Focus als Tab, im
selben Look wie im Terminal. Oben steht immer die Liste für heute, darunter
die Antwort auf deinen letzten Befehl, ganz unten tippst du wie gewohnt
(`+ Bericht #arbeit freitag`, ein `focus` vorne darf mit). Dazu:

- **Anklicken:** Eine Aufgabennummer zeigt die Aufgabe mit ihren Notizen,
  `#thema` alles im Thema, `@person` alles mit der Person. Ein Link öffnet
  sich in einem neuen Tab.
- **Tab-Taste** ergänzt wie in der Shell, **↑** holt frühere Befehle.
- Kommst du zurück in den Tab, ist die Liste aktuell, auch nach Änderungen im
  Terminal.
- **Übersicht anpassen:** „▾ Übersicht“ oben links klappt die Liste ein, die
  Linie darunter ändert beim Ziehen ihre Höhe (Doppelklick klappt auch ein).
- **Hell oder dunkel:** Der Knopf oben rechts schaltet zwischen auto (wie das
  System), hell und dunkel. Der Browser merkt sich beides.
- Im Hintergrund fällt ein leiser Zeichenregen aus Ziffern und Focus-Zeichen.
  `regen` in der Eingabe schaltet ihn aus und wieder an. Wer am Mac „Bewegung
  reduzieren“ eingestellt hat, bekommt ihn nicht.

Der Tab läuft, solange `focus web` im Terminal läuft (Strg+C beendet). Focus
hört nur auf `127.0.0.1:4711`, ist also nur auf deinem Mac erreichbar; ein
anderer Port geht mit `focus web 4712`. Was im Terminal nachfragt (Notizen
durchgehen, Themen ordnen, aufräumen), geht im Browser nicht; dort sagt Focus,
wie es mit einem Befehl geht.

## Was Focus absichtlich nicht kann

Keine Prioritätsstufen, keine Unteraufgaben, kein Adressbuch, keine
Mitteilungen, kein Teilen mit anderen (die Unterwegs-Datei ist für dich). Und kein viertes wichtiges Thema. Alles davon wäre
etwas, das gepflegt werden will.

---

## Installieren, aktualisieren, entfernen

Focus läuft auf macOS (Apple Silicon und Intel) und Linux.

**Mit Homebrew** (macOS und Linux):

```bash
brew install kleineLoesungen/tap/focus
```

**Ohne Homebrew** lädt ein Skript das passende Programm von GitHub nach
`~/.cargo/bin`:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/kleineLoesungen/cli-focus/releases/latest/download/focus-installer.sh | sh
```

**Aus dem Quelltext:** Mit [Rust](https://rustup.rs) im Projektordner
`cargo install --path .`.

Danach einmal:

```bash
focus einrichten
```

`focus einrichten` fragt zuerst nach der Sprache (Englisch oder Deutsch; neue
Installationen starten auf Englisch, bestehende mit deutscher Aufgabendatei
bleiben Deutsch), dann, wo deine Aufgaben liegen sollen, ob es eine
Unterwegs-Datei geben soll, wie viele Aufgaben auf heute und wie viele wichtige
Themen es geben darf (1 bis 9, ohne Angabe 3), ob Samstag und Sonntag bei
„fällig in … Tagen“ mitzählen (ohne sie ist am Freitag eine Aufgabe für Montag
schon dringend), ob `f` als Kurzbefehl gehen soll, ob jedes neue
Terminalfenster die Liste für heute zeigt, ob die Tab-Taste ergänzen soll
(zsh) und ob Focus das Fenster vor jedem Befehl leeren soll. Es schreibt dafür
einen markierten Block in deine `.zshrc` und die Einstellungen in
`~/.config/focus/einstellungen`. In zsh stehen `focus` und `f` dort mit
`noglob`, damit Links mit `?` oder `*` nicht an zsh scheitern („no matches
found“).

**Ein Fenster nur für Focus:** Mit `leeren = ja` in den Einstellungen leert
Focus vor jedem Befehl den sichtbaren Bereich. Oben steht grau der Befehl
(`› focus + Bericht morgen`), darunter die Antwort. Der Verlauf bleibt,
Hochscrollen oder `⌘↑` zeigt das Alte. Leitest du die Ausgabe um
(`focus | grep …`), wird nichts geleert.

Einen Ordner kannst du aus dem Finder ins
Terminal ziehen, statt den Pfad zu tippen.

**Unter Linux** schreibt `focus einrichten` bei bash in die `.bashrc`. Die
Tab-Ergänzung gibt es nur für zsh. Links und `focus datei` öffnet Focus mit
`xdg-open`, beim Deinstallieren landen die Aufgaben auf Wunsch im Papierkorb
des Desktops (`~/.local/share/Trash`).

**Aktualisieren:** `brew upgrade focus`, das Skript noch einmal, oder
`cargo install --path .` – je nachdem, wie du installiert hast. Deine Aufgaben
bleiben.

**Von „fokus“ auf „focus“:** Früher hießen Programm und Datei `fokus` und
`Fokus.md`. Focus benutzt eine vorhandene `Dokumente/Fokus.md` und die
Einstellungen aus `~/.config/fokus/` einfach weiter. Danach einmal
`focus einrichten`, damit der Block in der `.zshrc` und der Kurzbefehl `f` auf
das neue Programm zeigen, und das alte entfernen: `cargo uninstall fokus`.

**Entfernen:** `focus deinstallieren`. Es zeigt erst, was es tut, und fragt
nach. Deine Aufgaben bleiben, außer du sagst ausdrücklich ja – dann kommen sie
in den Papierkorb. Die Einstellungen löscht es mit, die Unterwegs-Datei nicht.

Zum Ausprobieren mit einer Spieldatei: `export FOCUS_DATEI=/tmp/spiel.md`.
Solange die gilt, lässt Focus deine Unterwegs-Datei in Ruhe (außer du setzt
auch `FOCUS_UNTERWEGS`).
Tests: `cargo test`. Eine neue Version erscheint mit einem Tag: erst in
`Cargo.toml` die `version` erhöhen (etwa auf `0.3.0`) und mergen, dann
`git tag v0.3.0 && git push --tags`. Tag und Version müssen gleich sein, sonst
bricht der Release ab. GitHub baut dann die Programme für macOS und Linux,
hängt sie an ein Release und aktualisiert die Homebrew-Formel.

## Lizenz

GPL-3.0, siehe [LICENSE](LICENSE).
