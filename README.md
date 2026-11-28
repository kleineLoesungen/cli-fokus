# Focus

*English · [Deutsch](README.de.md)*

**Less list, more done.** A small terminal helper for macOS and Linux that
answers one question every morning: *What's on for today?*

## The idea

* You write tasks down as they come to you: one sentence, done.
* Focus works out on its own what is **urgent** (due within two days or
  overdue) and what is **important** (its topic is above the line, see below).
* **Today** holds at most **three** open tasks, and at most **three** topics
  are **important**. No more, so that choosing stays necessary. (You can change
  both numbers in `focus setup`, from 1 to 9.)

You don't set priorities and you don't estimate hours.

---

## Typical situations

### In the morning: what's on for today?

```
focus
```
```
 Thursday, November 26                                         1 of 3 done
 ──────────────────────────────────────────────────────────────────────────

 TODAY
     ✓ send offer to client                                         #work
   2 ▶ prepare presentation                         due tomorrow    #work
   4 · buy running shoes                                            #health

 URGENT, BUT NOT IMPORTANT   → do it quickly or delegate
   6 · file travel expenses                         due tomorrow    #home

 4 more open · 2 waiting on others
 2 have been sitting for a while → focus review
```

The number is the task's **number**, ▶ shows what to start with. In the
morning Focus puts the list together itself (urgent and important first); you
can swap any time.

```
focus today 5             task 5 on today
focus today 5 instead 2   swap when today is full
focus later 2             take it off today again
focus done 2              check it off – that frees up a spot
```

Before a task goes onto today, Focus checks whether something more important
is open. The order is: important and urgent, important, urgent, neither. If
there's anything higher up, Focus says how many and shows up to three, and
asks whether this task still matches your focus:

```
 Task 4 is neither important nor urgent.
 There are 3 more important tasks (1 important and urgent, 1 important, 1 urgent):
     1  file taxes · #work · due tomorrow (Friday, Nov 27)
     2  website copy · #work
     3  travel expenses · no topic · due tomorrow (Friday, Nov 27)
 Does task 4 still match your focus for today? [y/N]
```

Enter means no. Tasks from the same command (`focus today 1 2 4`) don't count
as more important. Without a terminal to ask in, for example in the browser,
Focus does it anyway and adds a note with the numbers.

### Something comes to mind

```
focus + prepare presentation #work tomorrow
```
```
 ✓ Task 1: prepare presentation · #work · due tomorrow (Friday, Nov 27)
```

`#topic` and `@person` are optional, and so is the date. It goes **at the
end** (`today`, `tomorrow`, `friday`, `next friday`, `2026-12-03`, `in 2 weeks`,
`next week`). If something is due **today**, it goes straight onto today's
list, as long as there's room.

Things that come up regularly get a **repeat** at the end:

```
focus + write weekly plan #work every monday
focus + water the plants every 2 weeks on friday
```

When you check one off, Focus creates the next one right away. If you miss a
date, it moves on to the next one instead of piling up. Change it with
`focus edit 4 every 4 weeks monday`, end it with `no repeat` or
`focus drop 4`.

With characters like `'` or `?` in the text: just type `focus +` and Focus asks.

### Keeping more about it: notes and links

```
focus note 5 https://figma.com/file/abc
focus note 5 contact is Max, budget is in the offer
focus 5
```
```
 5  website relaunch
     #work · due Dec 12 · since today

   5.1  https://figma.com/file/abc
   5.2  contact is Max, budget is in the offer
```

A note is one line of text or a link. `focus note 5.2 …` replaces the second
one, `focus note 5.2 delete` deletes it. `focus note 5` on its own goes through
the notes: a digit picks one, then [e] edit or [d] delete, [n] adds a new one.
After `focus note 5.` the Tab key shows the notes with their text, so you don't
have to remember the numbers. `focus open 5` opens the link in the browser (if
there are several, Focus asks which).

Topics have notes too: `focus note #work Wiki: https://…`. They then appear at
the bottom of `focus #work`.

In the lists you only see a mark after the text, ↗ for a link, ✎ for other
notes. In the file the notes are indented below their task or topic, and the
same goes for the mobile file. They stay when you edit the task, put it on
today or archive it, and come along with a repeat.

### Something changed

```
focus edit 4 #home @tom friday        topic, person, date
focus edit 4 basement and attic       new text, the rest stays
focus edit 4 no date                  also: no topic (#none), no @anna, no repeat
```

### Getting started right now

```
focus now
```

The screen shows only the task with ▶ and a 25-minute clock.
`d` done, `p` pause, `l` later, `q` back.

### The overview of everything

```
focus all
```
```
                         URGENT                         NOT URGENT
           ┌────────────────────────────────┬────────────────────────────────┐
 IMPORTANT │  2 ▶ prepare present… tomorrow │  5   website copy       Dec 12 │
           │                                │  3   plan vacation             │
           │                                │  4 ▶ buy running shoes         │
           │                             DO │                           PLAN │
           ├────────────────────────────────┼────────────────────────────────┤
     MINOR │  6   file travel exp… tomorrow │  7   clear out basem…  18 days │
           │                                │ 10   call mom back             │
           │              QUICK OR DELEGATE │                          DROP? │
           └────────────────────────────────┴────────────────────────────────┘
```

**Plan** is the most valuable corner: it never pushes, and that's why it's
the easiest to lose. Whatever sits under **Drop?** for 30 days without a date,
Focus moves to the archive on its own.

### Checked off by accident?

```
focus archive
```
```
 DONE
   2 ✓ water the plants                                           today  #home

 DROPPED
   3 × file travel expenses                                   yesterday

 Bring back: focus restore <no> · after 14 days Focus deletes for good
```

`focus restore 2` brings the task back, onto today if it was there. Done and
dropped tasks keep their number while they're in the archive, and are **gone
for good after 14 days**.

### Where am I stuck in a topic?

```
focus #work
```
```
 #work   important
   2 ▶ prepare presentation                      due tomorrow
   5 · website copy                                due Dec 12

 WITH OTHERS
   8 · review contract                              with @tom  for 12 days

 Done in the last 7 days: send offer to client
```

`focus #none` shows what doesn't have a topic yet. Give it one with
`focus edit 3 #home`; `focus edit 3 #none` takes it away again. `#none` (and
the German `#ohne`) means "no topic" everywhere, so Focus never creates a topic
called "None".

### Am I making progress on what matters to me?

```
focus topics
```
```
 YOUR TOPICS      open    last 7 days
 1  #work            3    ○ ○ ○ ○ ○ ○ ●
 2  #family          2    ○ ○ ○ ○ ○ ○ ○
 3  #health          1    ○ ○ ○ ○ ○ ○ ○
 ─── important ──────────────────────────────
    #home            2    ○ ○ ○ ○ ○ ○ ○
    #none            1    no topic
```

One dot per day, today is on the right. At most three topics are above the
line (or as many as you set in `focus setup`); the first ones you create land
there on their own.

```
focus topics order                 asks which ones are important
focus topics rename wrok work      everywhere; if "work" exists, they merge
```

### You're meeting Anna

```
focus @anna
```
```
 @anna
   3 · plan vacation @anna                                      #family

 WAITING FOR @anna
   9 · send weekend photos                                      for 5 days
```

`focus people` lists everyone you have something open with.

### What am I still waiting for?

```
focus delegate 8 @tom     task 8 is now with @tom
focus waiting
```
```
 WAITING FOR   → follow up: focus review
   8 · review contract                              with @tom  for 12 days
   9 · send weekend photos                         with @anna  for 5 days
```

What's been waiting longest is on top. When something comes back done:
`focus done 8`.

### Where was that again?

```
focus search offer
```
```
 FOUND: OFFER
   4 · check offer ✎                                    due Friday  #work
```

Focus searches text, topic, people and notes. Several words must all appear,
case doesn't matter, and `pruefen` also finds `prüfen`. Matches in the archive
appear faded below.

### On the go: look things up and write them down

```
focus mobile ~/Dropbox/Focus
```

From now on Focus writes the current state to a second file,
`Focus-mobile.md`, in that folder every time it runs. Which service brings the
folder to your phone is up to you: iCloud Drive, Google Drive, Dropbox,
OneDrive, Nextcloud, Syncthing – Focus only writes a text file. An Android
phone can't reach iCloud Drive; for iPhone and Android together, pick another
service. Prefer `.txt`? `focus mobile ~/Dropbox/Focus/Focus.txt`.

```
# Focus mobile

As of: Friday, Nov 27, 06:10

Write new things at the very bottom under “New” …

## Today · Friday, Nov 27
- 2 prepare presentation · Work · due Saturday, Nov 28

## Waiting for
- 8 review contract · with Tom since Nov 12

## Open

### Work · important
- 5 website copy · due Saturday, Dec 12

## New
- order jerseys #club friday
idea for the training path
```

The state to read is on top, with dates instead of "tomorrow", because you
might read it later. Whatever comes to mind on the go, you write at the very
bottom under **New**, one line per task, like with `focus +`. The next time it
runs, Focus picks up the lines, creates them as tasks and empties the section.
Focus rewrites everything above each time; checking off and changing happens
on the computer.

Because "New" is at the end, appending a line to the file is enough. iPhone
Shortcuts and Android automation apps can do that, so you can even use your
voice without opening the file. To read and write by hand you need a text
editor on your phone that opens files directly in the sync folder (on the
iPhone, for example, Runestone); the Files apps only display text files.

Your `Focus.md` stays where it is: the mobile file is just a copy.
`focus mobile` shows where it is, `focus mobile off` turns it off.

### Once a week: review

```
focus review
```

Focus goes through what is **overdue** (from the day after the due date) and
what has been sitting for two weeks (or for one week with someone else). One
key per task:

```
 file taxes · no topic · due since yesterday · since yesterday
 [t] today  [n] new date  [w] this week  [o] give a topic
 [k] keep  [d] done  [x] drop
```

For delegated tasks there's also `[f] follow up` (puts "Follow up with Tom: …"
on today) and `[m] back to me`.

---

## All commands

| Command | |
|---|---|
| `focus` | what's on for today |
| `focus + text #topic @person date` | write it down |
| `focus today 4` · `later 4` · `done 4` | today's list |
| `focus edit 4 …` | change text, topic, person, date |
| `focus 4` · `note 4 …` · `note 4.2 …` · `open 4` | a task with its notes · add a note · change one · open a link |
| `focus drop 4` | get rid of it (14 days in the archive) |
| `focus archive` · `focus restore 4` | see what's done · bring it back |
| `focus now` | just one task, with a clock |
| `focus all` | the four quadrants |
| `focus #work` · `focus topics` | one topic · all topics |
| `focus #none` | tasks without a topic |
| `focus topics order` · `rename a b` | look after your topics |
| `focus @anna` · `focus people` | one person · everyone |
| `focus delegate 4 @tom` · `focus waiting` | hand it off · what's with others |
| `focus search word …` | find tasks, also in notes and the archive |
| `focus review` | go through what got stuck |
| `focus file` | open the task file in a text editor |
| `focus file move place` | move the task file somewhere else |
| `focus mobile folder` · `mobile off` | state for your phone, with "New" to write in |
| `focus web` | Focus in a browser tab |
| `focus setup` | language, places, shortcut `f`, Tab key |
| `focus help` | the short overview |
| `focus manual` · `manual mobile` | this manual, or just the matching section |

The German commands always work too (`focus heute 4`, `focus fertig 4`,
`morgen`, `jeden montag`), even when Focus speaks English.

Instead of `focus` you can type `f` (see setup). The **Tab key** completes
commands, numbers with their text, `#topics`, `@people`, dates and repeats –
pressing Tab twice shows the list, grouped:

```
focus done ⇥⇥
Today
7   travel expenses  · today
1   prepare presentation  · tomorrow
#work · important
2   website copy  · Dec 12
No topic
5   call mom back
With others
6   send photos  · with @anna
```

---

## Your data

Everything is in one text file, at first `Documents/Focus.md`. You can read
and edit it with any editor (`focus file`); a new line `- fix bike #home` is
enough, Focus fills in the rest. Indented lines below a task are notes for
that task. Done and dropped tasks are deleted after 14 days. No account, no
cloud – unless your Mac backs up the Documents folder to iCloud, or you put the
file into a sync folder yourself or turn on the mobile file.

Move it elsewhere with `focus file move ~/Planning` (a folder gets `Focus.md`,
a `.md` file keeps its name). Focus moves the file and remembers the place. If
there's already a task file there, Focus asks whether to use it and overwrites
nothing. The places are stored in `~/.config/focus/einstellungen`;
`focus setup` asks about them too.

The file is written in the language Focus speaks, and read in both: switching
languages changes the headings, nothing gets lost.

## In the browser: `focus web`

If you work in the browser a lot anyway: `focus web` opens Focus as a tab,
looking just like the terminal. Today's list is always on top, below it the
answer to your last command, and at the very bottom you type as usual
(`+ report #work friday`, a leading `focus` is fine). Also:

- **Click:** a task number shows the task with its notes, `#topic` everything
  in the topic, `@person` everything with that person. A link opens in a new
  tab.
- **Tab key** completes like in the shell, **↑** brings back earlier commands.
- When you come back to the tab, the list is up to date, even after changes in
  the terminal.
- **Adjust the overview:** "▾ Overview" at the top left collapses the list,
  dragging the line below it changes its height (double-click collapses too).
- **Light or dark:** the button at the top right switches between auto (like
  the system), light and dark. The browser remembers both.
- A quiet rain of digits and Focus characters falls in the background. Typing
  `regen` turns it off and on again. If you've set "Reduce motion" on your Mac,
  you won't get it.

The tab works as long as `focus web` runs in the terminal (Ctrl+C stops it).
Focus only listens on `127.0.0.1:4711`, so it's only reachable from your
computer; use another port with `focus web 4712`. Things that ask questions in
the terminal (going through notes, ordering topics, review) don't work in the
browser; there Focus tells you how to do it with a command.

## What Focus deliberately can't do

No priority levels, no subtasks, no address book, no notifications, no
sharing with others (the mobile file is for you). And no fourth important
topic. Each of those would be something that needs looking after.

---

## Install, update, remove

Focus runs on macOS (Apple Silicon and Intel) and Linux.

**With Homebrew** (macOS and Linux):

```bash
brew install kleineLoesungen/tap/focus
```

**Without Homebrew** a script downloads the right program from GitHub to
`~/.cargo/bin`:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/kleineLoesungen/cli-focus/releases/latest/download/focus-installer.sh | sh
```

**From source:** with [Rust](https://rustup.rs), in the project folder
`cargo install --path .`.

Then once:

```bash
focus setup
```

`focus setup` first asks for the language (English or German; new
installations start in English, existing ones with a German task file stay
German), then where your tasks should live, whether there should be a mobile
file, how many tasks today and how many important topics there may be (1 to 9,
3 if you don't say), whether Saturday and Sunday count for "due in … days"
(without them, on Friday a task for Monday is already urgent), whether `f` should work as a shortcut, whether every new terminal window
shows today's list, whether the Tab key should complete (zsh) and whether
Focus should clear the window before each command. For that it writes a marked
block into your `.zshrc` and the settings into `~/.config/focus/einstellungen`.
In zsh, `focus` and `f` are set up there with `noglob`, so links with `?` or
`*` don't trip up zsh ("no matches found").

**A window just for Focus:** with `leeren = ja` in the settings, Focus clears
the visible area before each command. The command is shown on top in grey
(`› focus + report tomorrow`), the answer below. The history stays; scrolling
up or `⌘↑` shows the old output. If you redirect the output
(`focus | grep …`), nothing is cleared.

You can drag a folder from Finder into the terminal instead of typing the
path.

**On Linux**, `focus setup` writes to `.bashrc` for bash. Tab completion is
only available for zsh. Focus opens links and `focus file` with `xdg-open`;
when uninstalling, your tasks go to the desktop trash if you want
(`~/.local/share/Trash`).

**Update:** `brew upgrade focus`, the script again, or `cargo install --path .`
– depending on how you installed. Your tasks stay.

**From "fokus" to "focus":** the program and file used to be called `fokus`
and `Fokus.md`. Focus simply keeps using an existing `Documents/Fokus.md` and
the settings from `~/.config/fokus/`, and stays in German if that file is
German. Afterwards run `focus setup` once, so the block in `.zshrc` and the
shortcut `f` point to the new program, and remove the old one:
`cargo uninstall fokus`.

**Remove:** `focus uninstall`. It first shows what it will do and asks. Your
tasks stay unless you explicitly say yes – then they go to the trash. It
deletes the settings too, but not the mobile file.

To try things out with a practice file: `export FOCUS_DATEI=/tmp/practice.md`.
While that's set, Focus leaves your mobile file alone (unless you also set
`FOCUS_UNTERWEGS`). `FOCUS_SPRACHE=de` or `en` sets the language for one call.
Tests: `cargo test`. A new version comes out with a tag: first raise the
`version` in `Cargo.toml` (for example to `0.3.0`) and merge, then
`git tag v0.3.0 && git push --tags`. Tag and version must match, otherwise the
release stops. GitHub then builds the programs for macOS and Linux, attaches
them to a release and updates the Homebrew formula.

## License

GPL-3.0, see [LICENSE](LICENSE).
