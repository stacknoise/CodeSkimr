# 0075 – Folder dialog of the operating system

Status: accepted

## Context

The product owner asked for a "Browse" button next to the folder field of the new-project form (test findings of 2026-11-19) and approved the mock (`docs/mocks/FolderPicker.html`, mock 19) on 2026-11-20. A browser cannot give a web page the path of a folder (the file APIs hide it), so the dialog has to be opened by the app on the machine it runs on. That needs a dependency, and the dialog opens a window on the machine, which makes the command security-relevant.

## Decision

1. **A crate for the native dialog.** `rfd` (MIT or Apache-2.0) with its default features off, on **Windows and macOS only**. Both have a system dialog without extra system libraries; its tree adds only the Windows and macOS bindings that the tray icon (ADR 0067) already pulls in. Linux has no common dialog without GTK or a desktop portal, so, as with the tray, it is left out and keeps the typed path. A portal backend can be added later by a new ADR.
2. **Two commands in the UI command API, no MCP tool.** `projects.folder_dialog` (read) answers `{ available }`; `projects.pick_folder` (permission `register_project`, so it needs the token and an own `Host` and `Origin` like `projects.add`) shows the dialog, waits and answers `{ path }`: the chosen folder, or `null` if the user cancelled, no dialog exists on this machine or another one is open. Only one dialog is open at a time. Neither is an MCP tool: an AI agent must not open windows on the user's screen.
3. **The dialog only returns a path.** Nothing is read, listed or created in the folder. The path goes into the field of the form as if it had been typed, and the user still confirms with "Add and scan"; `projects.add` checks it like every typed path (absolute, an existing folder).
4. **The UI.** The button "Browse …" stands next to the field if `available` is true and the source is a folder. While the dialog is open the form shows "Folder dialog open" with "Stop waiting" (the window of the system stays until the user closes it; its answer is dropped). After a choice the form says "Folder taken over. Check the path …". Without a dialog the button is missing and nothing else changes. Hell, Petrol and Dunkel are covered by the mock.
5. **A server on another machine has no button.** The dialog would open on the server, not in front of the user. The app reports `available: false` for a server that is not the app on the user's machine (today: every build without the dialog; the team server of 1.0 will answer `false` by decision).

## Consequences

- New dependency `rfd` (label `new-dependency`, justification: the only way for a browser UI to get the path of a folder the user chose).
- Both commands are in `docs/server.md`, the UI text in `docs/ui.md`, the manual and the changelog. The security suite checks that `projects.pick_folder` refuses a missing, wrong or empty token (`401`) and a foreign or `null` `Host` or `Origin` (`API-402`); no test opens a real dialog.
- The dialog on macOS is shown from the thread of the call; `rfd` hands it to the main thread, which the tray (ADR 0067) already runs as the event loop. It was not run on macOS in CI (no display); the first test version on macOS has to confirm it.
- The one-at-a-time guard is in the app; two browser tabs share it.
