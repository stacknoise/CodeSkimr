# 0088 – Print the launch address on request

Status: accepted (decided by the product owner on a decision card, 2026-11-23; extends ADR 0028 and ADR 0030, changes none of their decisions)

## Context

ADR 0028 gives the browser the one-time launch code through a launch file (`file://`, mode 0600), never through a printed address, so that the code does not end up in a command line, a log or the browser history. The remote browser tests of an agent session showed the cost: the browser window of the Claude app and Claude in Chrome do not open `file://` pages, so they could not sign in to the app at `127.0.0.1:7421`.

## Options

1. Keep the launch file only. Browser tests of this kind stay impossible.
2. Let `open` print the address by default. This weakens the protection of every user.
3. A switch that has to be named, `codeskimr open --no-browser --print-url`. The default stays as it is.

## Decision

Option 3.

- `--print-url` needs `--no-browser` (the parser refuses it alone, `CLI-101`). Naming both is the explicit opt-in.
- The command asks the server for a code like before, but writes no launch file, starts no browser and does not wait. It prints one line to standard output: `http://127.0.0.1:<port>/#code=<32 hex digits>`. A warning goes to standard error.
- The code is in the fragment, as before: the browser does not send it to a server, and the UI takes it out and removes the fragment from the address at once (`replaceState`, ADR 0028), so it stays out of the history and of `Referer`.
- The code is as strong as before: 128 random bits, valid `ui.launch_code_seconds` (default 60), usable once, one code at a time, a wrong code does not burn the valid one. The server holds the token text only for that time (ADR 0030).
- The code never goes into the log of CodeSkimr and never into a command line of CodeSkimr. Where the printed line ends up is the choice of the person who named the switch.

## Consequences

- Remote browser tests can sign in: read the line, open the address in the browser tool, done within the lifetime of the code.
- The new risk is limited to what the caller does with the line within the lifetime: another process that reads the same terminal or log can sign in once if it is faster than the owner. A used or expired code is useless. The threat note in `docs/server.md` says so.
- No new dependency, no new error code, no change to the server.
