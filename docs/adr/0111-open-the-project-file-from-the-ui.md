# 0111 – Opening the project file from the UI

Status: accepted (the button "Open .codeskimr.toml" was released with its mock by the product owner on 2026-11-26). Labelled `security-sensitive`: it starts a program of the operating system.

## Context

The screen Rules shows rules that live in `.codeskimr.toml`, next to the project folder. Whoever wants to change a rule has to find that file first. A button that opens it with the program the system names for `.toml` files closes that gap. It is the first command that starts another program, so the questions "which path", "who may" and "from where" have to be answered once, here.

## Options

1. A command `project_file.open` that builds the path itself and hands it to the system opener.
2. A command that takes a path from the caller (refused: a path from outside would let any caller open any file).
3. Only show the path and let the user open it (works, but is the gap this ADR closes).

## Decision

Option 1.

- **The path is never from the caller.** The input is the project id. The path is the registered folder of the project plus the fixed name `.codeskimr.toml` (`PROJECT_CONFIG_FILE`). A Git project has no folder and is refused with `PRJ-101`; an unknown project is `PRJ-201`.
- **No links, only plain files.** The file is looked at with `symlink_metadata`. A link, a folder or anything that is not a regular file is refused with `CFG-102` and nothing is started; a link is never followed. This is the rule of reading and writing the file (ADR 0065).
- **A missing file is created empty**, with an exclusive create (`create_new`), so nothing existing is ever followed or overwritten. If that fails, the answer is `CFG-104`. The answer says `created: true`, and the screen says so.
- **Right to manage.** The check `access::require(Manage)` comes first, before the project is even looked up, like `layers.set` and `unused.keep`. A reader is refused with `AUTH-401`.
- **Only for the local user.** Opening a program is a thing that happens on the machine where the person sits. The command is exposed on the REST API of the app (which the UI uses) and is **not an MCP tool**: an agent has no use for it and must not be able to start programs. A server that runs for a team (a later version, ADR 0033 mentions the same limit for installing tools) must switch the command off, because the program would open on the server and not at the person who clicked. This ADR states that as a rule for that version.
- **One opener.** `FileOpener` is a trait; `SystemOpener` uses the code that already starts the browser (ADR 0028): `xdg-open`, `open` or `start` with the path as one argument, never through a shell. The tests use a fake opener and never start a program.
- **What the answer holds.** Only the file name and `created`, never a path of the server's disk.
- **New error `APP-605`** (`project_file_open_failed`): the system started no program (none is registered for the type, or the start failed). It is retryable. The text says what to do: open the file by hand.

## Consequences

- The REST command is 55th in the list; the console has no counterpart because it already runs in the editor of its user (ADR 0092 lists the counterpart: the file lies in the project folder, `codeskimr rules` names it).
- On a machine without a desktop (a build server) the command fails with `APP-605`; nothing else happens, except that an empty file may have been created.
- A later team version has to disable the command (see above); a test for that belongs to that version.
