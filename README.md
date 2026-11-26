# Fokus

**Weniger Liste, mehr erledigt.**

Fokus ist ein kleiner Helfer fürs Terminal auf dem Mac. Er beantwortet jeden
Morgen eine einzige Frage: *Was ist heute dran?* Und er hilft dir, das dann
auch zu tun, statt den Tag mit Dringendem zu füllen, das gar nicht wichtig ist.

---

## Die Idee in drei Sätzen

1. Du schreibst Aufgaben auf, so wie sie dir einfallen: ein Satz, fertig.
2. Fokus erkennt selbst, was **dringend** ist (am Termin) und was **wichtig**
   ist (an deinem Thema).
3. Für heute wählst du **höchstens drei** Aufgaben, und für jetzt **genau eine**.

Du musst nichts bewerten, keine Prioritäten vergeben und keine Stunden
schätzen. Fokus fragt nur dann nach, wenn eine Aufgabe zu lange liegt.

---

## So sieht ein Tag aus

Morgens tippst du einfach:

```
fokus
```

und siehst:

```
 Donnerstag, 26. November                                    1 von 3 erledigt
 ────────────────────────────────────────────────────────────────────────────

 HEUTE
     ✓ Angebot an Kunden schicken                                   Arbeit
   2 ▶ Präsentation vorbereiten                      fällig morgen  Arbeit
   4 · Laufschuhe kaufen                                            Gesundheit

 DRINGEND, ABER NICHT WICHTIG   → schnell erledigen oder abgeben
   6 · Reisekosten abrechnen                         fällig morgen  Haus

 6 weitere offen · 1 wartet auf andere → fokus alles
```

Die Zahl vor jeder Aufgabe ist ihre **Nummer**. Mit ihr sagst du Fokus, welche
Aufgabe du meinst: `fokus fertig 2`. Der Pfeil ▶ zeigt, womit du anfangen
solltest.

Mehr brauchst du an den meisten Tagen nicht.

---

## Aufgaben aufschreiben

```
fokus + Präsentation vorbereiten #arbeit morgen
fokus + Urlaub abstimmen @anna #familie
fokus + Laufschuhe kaufen #gesundheit
fokus + Mama zurückrufen
```

* `#arbeit` ist das **Thema**. Das darfst du weglassen, dann landet die
  Aufgabe erst einmal im Eingang.
* `@anna` ist eine **Person**, mit der die Aufgabe zu tun hat. Auch das ist
  freiwillig.
* `morgen`, `freitag`, `3.12.`, `in 2 wochen` oder `nächste woche` ist der
  **Termin**. Den gibst du nur an, wenn es wirklich einen gibt. Er steht **am
  Ende**: „Termin morgen absagen“ hat deshalb keinen Termin, „Präsentation
  vorbereiten morgen“ schon.

Fokus sagt dir sofort, was es verstanden hat:

```
 ✓ Aufgabe 2: Präsentation vorbereiten · Arbeit · fällig morgen (Freitag 27.11.)
```

Enthält dein Text Zeichen wie `'` oder `?`, tippst du nur `fokus +` und
Enter. Dann fragt Fokus nach dem Text, und das Terminal kommt nicht
durcheinander.

Aufschreiben soll nie länger dauern als der Gedanke selbst.

---

## Themen: höchstens drei sind wichtig

Ein Thema ist ein Bereich deines Lebens, zum Beispiel *Arbeit*, *Familie*,
*Gesundheit*, *Haus* oder *Projekt*. Anlegen darfst du so viele, wie du
willst: Ein Thema entsteht, sobald du es zum ersten Mal mit `#` schreibst.

Die Themen stehen in einer **Reihenfolge**, und eine Linie trennt sie:

```
fokus themen
```

```
 DEINE THEMEN   offen    letzte 7 Tage
 1  Arbeit          1    ○ ○ ○ ○ ○ ○ ●
 2  Projekt         1    ○ ○ ○ ○ ○ ○ ○
 3  Gesundheit      2    ○ ○ ○ ○ ○ ○ ○   seit 12 Tagen nichts erledigt
 ─── wichtig ───────────────────────────────
    Haus            2    ○ ○ ○ ○ ○ ○ ○
    Familie         3    ○ ○ ○ ○ ○ ○ ○
    ohne Thema      1
```

Jeder Punkt ist ein Tag, ganz rechts ist heute. Ein voller Punkt heißt: An dem
Tag hast du in diesem Thema etwas erledigt. Bleibt ein wichtiges Thema eine
Woche lang leer, sagt Fokus es dir.

Über der Linie stehen **höchstens drei** Themen. Ihre Aufgaben gelten als
wichtig, alle anderen nicht. Die ersten drei Themen, die du anlegst, landen
von selbst dort; jedes weitere kommt unter die Linie, und Fokus sagt dir das.

Welche drei es sind, bestimmst du jederzeit neu:

```
fokus themen ordnen
```

Fokus zeigt alle Themen mit Nummern und fragt, welche drei dir gerade am
wichtigsten sind. Du tippst sie der Reihe nach, etwa `2 1 4`. Die übrigen
rutschen unter die Linie. Es geht auch in einem Schritt:

```
fokus themen ordnen arbeit projekt gesundheit
```

Ein viertes wichtiges Thema gibt es nicht. Das ist keine Einstellung, sondern
der Kern von Fokus: Wenn alles wichtig ist, ist nichts mehr wichtig, und Fokus
könnte dir nicht mehr helfen, zu wählen.

Ein Thema unter der Linie ist nicht weg. Seine Aufgaben bleiben, sie zählen nur
so lange als „nicht wichtig“, bis du es wieder nach oben holst.

---

## Personen: Was habe ich mit wem?

Schreibst du `@anna` in eine Aufgabe, findest du sie später über die Person
wieder. Das hilft vor allem in dem Moment, in dem du Anna triffst und dich
fragst, was du eigentlich mit ihr besprechen wolltest:

```
fokus @anna
```

```
 ANNA
   8 · Urlaub abstimmen @anna                                       Familie
   9 · Geburtstag Tom planen @anna                  fällig Sonntag  Familie

 WARTE AUF ANNA
  10 · Fotos vom Wochenende schicken                                seit 5 Tagen
```

Alle Personen, bei denen gerade etwas offen ist, siehst du so:

```
fokus personen
```

```
 PERSONEN      mit dir    warte auf
 Anna                2            1
 Tom                 1            –
 Steuerberater       –            1
```

Personen ohne offene Aufgaben tauchen hier nicht auf. Eine Person entsteht,
sobald du sie zum ersten Mal mit `@` schreibst; ein Adressbuch gibt es nicht.

### Abgeben und warten

Manches musst nicht du erledigen. Mit

```
fokus abgeben 4 @tom
```

gibst du Aufgabe 4 an Tom. Sie verschwindet von deiner Liste und steht unter
„Warte auf Tom“. Kommt sie erledigt zurück, hakst du sie wie jede andere ab.
Liegt sie länger als eine Woche, fragt Fokus beim Aufräumen, ob du nachhaken
willst.

Eine Person ändert übrigens nichts daran, ob eine Aufgabe wichtig ist. Das
entscheidet allein das Thema.

---

## Wie Fokus „dringend“ und „wichtig“ erkennt

| | Woran Fokus es erkennt |
|---|---|
| **Dringend** | Der Termin ist heute, morgen oder übermorgen, oder er ist schon verstrichen. |
| **Wichtig** | Die Aufgabe gehört zu einem Thema über der Linie. |

Daraus ergeben sich vier Felder, mit je einem Rat:

```
fokus alles
```

```
                       DRINGEND                     NICHT DRINGEND
           ┌───────────────────────────────┬───────────────────────────────┐
   WICHTIG │  2 ▶ Präsentation vor… morgen │  3   Website fürs Projekt     │
           │                               │  4 ▶ Laufschuhe kaufen        │
           │                               │  5   Laufplan erstellen       │
           │                           TUN │                        PLANEN │
           ├───────────────────────────────┼───────────────────────────────┤
 UNWICHTIG │  6   Reisekosten abre… morgen │  9   Geburtstag Tom … Sonntag │
           │                               │  7   Keller ausmisten 18 Tage │
           │                               │ 11   Mama zurückrufen         │
           │          SCHNELL ODER ABGEBEN │                    STREICHEN? │
           └───────────────────────────────┴───────────────────────────────┘
 ▶ steht auf heute · ohne Thema zählt als nicht wichtig
```

* **Tun:** gehört auf die Liste für heute.
* **Planen:** Das ist die wertvollste Ecke. Diese Aufgaben gehen am leichtesten
  unter, weil sie nie drängeln.
* **Schnell oder abgeben:** bündeln, in zehn Minuten wegarbeiten, oder mit
  `fokus abgeben` jemand anderem geben.
* **Streichen?:** Was hier ohne Termin 30 Tage unberührt liegt, räumt Fokus von
  selbst ins Archiv. Weg ist es nicht: Es steht unten in deiner Datei.

In einem schmalen Fenster stehen die vier Felder untereinander statt
nebeneinander.

---

## Heute: höchstens drei

**Morgens schlägt Fokus dir die Liste vor:** zuerst, was dringend und wichtig
ist, dann, was gestern liegen blieb, dann Wichtiges ohne Eile. Unwichtiges
schlägt Fokus nie vor. Über der Liste steht dann „Vorschlag von Fokus“, und du
kannst ihn übernehmen oder austauschen.

```
fokus heute 4
```

legt Aufgabe 4 auf die Liste für heute. Sind schon drei darauf, fragt Fokus,
welche dafür weichen soll. Das ist Absicht: Eine vierte Aufgabe passt nur dann,
wenn eine andere warten darf. Ohne Rückfrage geht es so:

```
fokus heute 4 statt 2
```

Weitere Befehle für die Liste:

```
fokus fertig 2       hakt Aufgabe 2 ab
fokus später 2       nimmt Aufgabe 2 wieder von heute herunter
```

Die Nummer einer erledigten Aufgabe vergibt Fokus erst am nächsten Tag neu.
So erwischst du nie aus Versehen eine andere Aufgabe, wenn du nach einer
älteren Ansicht tippst.

---

## Jetzt: genau eine

```
fokus jetzt
```

Der Bildschirm wird leer, bis auf die Aufgabe mit dem Pfeil ▶:

```
 ╭──────────────────────────────────────────────────────╮
 │                                                      │
 │    Präsentation vorbereiten                          │
 │    Arbeit · fällig morgen                            │
 │                                                      │
 │    ━━━━━━━━━━━━━━━━━━━━━━━━━░░░░░░░░░░░░    18:42    │
 │                                                      │
 │    [f] fertig   [p] Pause   [s] später   [q] zurück  │
 │                                                      │
 ╰──────────────────────────────────────────────────────╯
```

Die Uhr läuft 25 Minuten. Dann klingelt es einmal, und du kannst mit `w`
weitere 25 Minuten anhängen oder aufhören. Hakst du die Aufgabe mit `f` ab,
bietet Fokus dir gleich die nächste von heute an.

`fokus jetzt 5` nimmt dir statt der ersten Aufgabe die Nummer 5 vor.

---

## Aufräumen in zwei Minuten

Einmal pro Woche, oder wenn Fokus dich unten in der Tagesansicht daran
erinnert:

```
fokus aufräumen
```

Fokus zeigt dir jede Aufgabe, die seit zwei Wochen unberührt liegt (oder seit
einer Woche bei jemand anderem), und du drückst eine Taste:

```
 Keller ausmisten · Haus · seit 18 Tagen
 [h] heute  [w] diese Woche  [t] anderes Thema  [b] behalten  [f] erledigt  [x] streichen
```

```
 Fotos vom Wochenende schicken · wartet auf Anna · seit 12 Tagen
 [n] nachhaken (auf heute)  [z] zurück zu mir  [b] weiter warten  [f] erledigt  [x] streichen
```

* **diese Woche** gibt der Aufgabe den Sonntag als Termin.
* **behalten** heißt: Sie bleibt, wie sie ist, und Fokus fragt in zwei Wochen
  wieder.
* **nachhaken** legt „Bei Anna nachhaken: …“ als neue Aufgabe auf heute.
* **Enter** überspringt eine Aufgabe, **q** hört auf.

Danach ist deine Liste wieder ehrlich.

---

## Alle Befehle auf einen Blick

| Du tippst | Fokus … |
|---|---|
| `fokus` | zeigt, was heute dran ist |
| `fokus + Text #thema @person termin` | schreibt eine Aufgabe auf |
| `fokus +` | fragt nach dem Text |
| `fokus heute 4` | legt Aufgabe 4 auf heute |
| `fokus heute 4 statt 2` | tauscht Aufgabe 2 gegen Aufgabe 4 |
| `fokus später 4` | nimmt Aufgabe 4 wieder von heute |
| `fokus fertig 4` | hakt Aufgabe 4 ab |
| `fokus jetzt` | zeigt nur eine Aufgabe, mit Uhr |
| `fokus alles` | zeigt alle Aufgaben in den vier Feldern |
| `fokus themen` | zeigt deine Themen und wie es dort vorangeht |
| `fokus themen ordnen` | wählt die drei wichtigen Themen |
| `fokus @anna` | zeigt alles, was du mit Anna hast |
| `fokus personen` | zeigt alle Personen mit offenen Aufgaben |
| `fokus abgeben 4 @tom` | gibt Aufgabe 4 an Tom und merkt sie als „warte auf“ |
| `fokus streichen 4` | streicht Aufgabe 4 (sie bleibt im Archiv der Datei) |
| `fokus aufräumen` | geht liegengebliebene Aufgaben mit dir durch |
| `fokus datei` | öffnet deine Aufgabendatei im Texteditor |
| `fokus einrichten` | richtet den Kurzbefehl `f` ein |
| `fokus hilfe` | zeigt diese Liste |

Hinter `fertig`, `später`, `streichen` und `heute` dürfen auch mehrere Nummern
stehen: `fokus fertig 2 5`.

---

## Wo deine Aufgaben liegen

In einer einzigen Textdatei: `Dokumente/Fokus.md`. `fokus datei` öffnet sie
im Texteditor. Du kannst sie jederzeit lesen und ändern; eine neue Zeile wie
`- Fahrrad reparieren #haus` unter „Offen“ reicht, Nummer und Datum ergänzt
Fokus beim nächsten Aufruf. Was Fokus nicht versteht, landet unter „Notizen“,
statt verloren zu gehen.

```markdown
## Themen
1. Arbeit
2. Projekt
3. Gesundheit
---
- Haus
- Familie

## Heute · 26.11.2026
- [x] Angebot an Kunden schicken #arbeit  [Nr. 1 · erledigt 26.11.2026]
- [ ] Präsentation vorbereiten #arbeit  [Nr. 2 · fällig 27.11.2026 · seit 26.11.2026]
- [ ] Laufschuhe kaufen #gesundheit  [Nr. 4 · seit 26.11.2026]

## Warte auf
- [ ] Fotos vom Wochenende schicken #familie  [Nr. 10 · wartet auf @anna · seit 26.11.2026]

## Eingang
- [ ] Mama zurückrufen  [Nr. 11 · seit 26.11.2026]

## Offen
- [ ] Website fürs Projekt #projekt  [Nr. 3 · seit 26.11.2026]
- [ ] Reisekosten abrechnen #haus  [Nr. 6 · fällig 27.11.2026 · seit 26.11.2026]
```

Darunter folgen, sobald es sie gibt, „Erledigt“ und „Archiv“.

Es gibt kein Konto, keine Cloud und kein Abo. Sichert dein Mac den Ordner
Dokumente in iCloud, liegt die Datei allerdings auch dort; das entscheidet dein
Mac, nicht Fokus. Wenn du Fokus eines Tages nicht mehr magst, bleibt dir eine
lesbare Liste.

---

## Was Fokus absichtlich nicht kann

* **Keine Prioritätsstufen** wie A/B/C oder 1 bis 5. Das ergibt sich aus Thema
  und Termin.
* **Mehr als drei wichtige Themen gibt es nicht.** Das ist der Kern von Fokus,
  keine Einstellung.
* **Keine Unteraufgaben, Projekte oder Abhängigkeiten.** Wird eine Aufgabe zu
  groß, schreib den nächsten Schritt als eigene Aufgabe auf.
* **Kein Adressbuch.** Eine Person ist nur ein Name an einer Aufgabe, ohne
  Telefonnummer, Rolle oder Zuständigkeit.
* **Keine Erinnerungen per Mitteilung.** Fokus meldet sich, wenn du ihn
  aufrufst, nicht dazwischen.
* **Kein Teilen und kein Abgleich.** Es ist deine Liste. „Abgeben“ heißt nur,
  dass du dir merkst, wer gerade dran ist; Fokus schickt niemandem etwas.

Jede dieser Funktionen wäre eine weitere Sache, die gepflegt werden will. Genau
das soll Fokus dir abnehmen.

---

## Installieren

Einmalig brauchst du dafür Rust, die Sprache, in der Fokus geschrieben ist. Die
Anleitung steht auf [rustup.rs](https://rustup.rs); es ist ein einziger Befehl
im Terminal.

Dann, in diesem Ordner:

```bash
cargo install --path .
```

```bash
fokus einrichten
```

`fokus einrichten` fragt zwei Dinge: ob `f` als Kurzbefehl funktionieren soll
und ob jedes neue Terminalfenster gleich die Liste für heute zeigt. Es ändert
dafür nur deine `.zshrc`, in einem markierten Block, den es beim nächsten
Aufruf ersetzt statt verdoppelt.

---

## Für Neugierige

* Fokus ist ein einziges kleines Programm ohne Hintergrunddienst. Es läuft nur,
  während du es aufrufst.
* `FOKUS_DATEI=/pfad/zur/datei.md` nimmt eine andere Datei, etwa zum
  Ausprobieren.
* Tests: `cargo test`. Sie setzen den Tag mit `FOKUS_HEUTE=2026-11-26` fest.

---

## Lizenz

GPL-3.0, siehe [LICENSE](LICENSE).
