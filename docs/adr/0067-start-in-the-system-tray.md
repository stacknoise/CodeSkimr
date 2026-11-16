# 0067 – The bare start runs in the system tray

Status: proposed

## Context

The bare `codeskimr` runs the server in the console window that started it; closing the window ends the server (ADR 0030). The product owner wants the program to run without a permanent console window: on Windows as a tray program with a right-click menu (open the browser, show the log, quit), and a way to close the console. How it looks on Linux and macOS was open.

## Decision

1. **Windows and macOS: tray by default.** The bare `codeskimr` starts a second copy of itself and ends. The copy has no console window (Windows: process creation flag `CREATE_NO_WINDOW`; macOS: a process group of its own, so that closing the terminal does not end it), runs the server and shows a tray icon (Windows notification area, macOS menu bar). The starting program prints one line with the place of the log and returns the prompt.
2. **The menu** has three entries: *Open CodeSkimr* (a new launch code and the browser), *Show log* (the log file with the system's opener) and *Quit CodeSkimr* (ends the server and the program). The browser is also opened once at the start.
3. **The log** is `codeskimr.log` in the data folder. It starts empty with every start, so it cannot grow without bound across runs. Everything the program writes to standard error (blueprint S-01, already redacted) goes there.
4. **Console and servers stay.** `codeskimr --console` keeps the old behaviour (server, browser and messages in the window). `codeskimr serve`, `scan`, `mcp` and every other command are unchanged and need no tray or desktop. **Linux** has no common tray without system libraries (GTK, an AppIndicator extension) that a single-file download cannot bring, so the bare start stays in the console there. A Linux tray is possible later as an own decision.
5. **One program.** The release stays one file per system: no second program for the tray, and no `windows_subsystem` attribute, which would break the console commands. The copy is marked by the environment variable `CODESKIMR_TRAY_CHILD`; nothing else changes about how it runs.
6. **New error code `APP-603`** `tray_start_failed`: the copy could not be started, or the icon could not be shown. Its hint tells to start with `--console`.
7. **New dependencies (Windows and macOS builds only):** `tray-icon` (tray icon and menu) and `winit` (the event loop that the icon needs, no window is opened; chosen over `tao`, which pulls GTK crates into the lock file that `cargo audit` rejects). Both are MIT or Apache-2.0 and are pulled in only for these two systems. No `unsafe` of our own is needed (rule R-08 stays).

## Consequences

- A start that fails in the copy (for example a taken port `API-301`) shows no window; the reason is in the log. The starting program cannot see it, because it has ended already. This is the price of closing the terminal.
- A server that ends on its own after the start takes the icon with it: the tray looks once a second whether the server thread has ended, then the program ends and the cause is in the log.
- `CODESKIMR_TRAY_CHILD` is inherited by the processes of the copy. Nothing in the program starts `codeskimr` again without a command; if that ever changes, that call must remove the variable.
- The icon is a plain colour square for now; a real image is a follow-up.
- The clean-install probe of the release starts with `--console`, because a runner has no tray the probe could look at. The tray start itself is checked by hand on Windows and macOS.
- Blueprint 10.2 describes the bare start; the sentence about the tray is added there with this ADR.
