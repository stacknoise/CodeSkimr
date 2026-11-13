# 0014 – Benchmarks compare against the base commit; output of tools

Status: proposed

## Context

Blueprint 16.1 asks for CI to measure "relative to a stored baseline" on a "mid-range developer laptop" (open point O-06). A baseline stored in the repository would hold numbers from one machine, while shared CI runners differ from run to run and from machine to machine; a 10 % tolerance on such numbers produces false alarms, and the `bench` check is required.

Blueprint 3.2 (R-06) forbids literals in production code. The tools under `tools/` print messages for developers and CI logs; routing these through Fluent files would serve nobody.

## Options

1. Store a baseline in the repository, measured on a developer laptop.
2. Measure the base commit and the change one after the other on the same CI runner and compare the ratio (A/B).
3. Count instructions instead of time (`iai-callgrind`), which needs a new dependency and Valgrind.

## Decision

Option 2, chosen by the product owner on 2026-11-13.

- The reference for the required check is the CI runner (`ubuntu-latest`); `tools/bench/machine.toml` records this and the class of laptop the budgets are meant for. A failed comparison is measured once more before the job fails.
- Absolute budgets of 16.1 are checked on the same runner as a coarse upper bound (`tools/bench/budgets.toml`); exact numbers for milestone acceptance are measured by hand on a laptop.
- Format strings and error messages of the developer tools under `tools/` are exempt from R-06. Values (numbers, names, paths, flags) still live in `consts.rs`. Code that ships in the product keeps R-06 without exception.
- The dependency rule file `tools/ci/architecture.toml` gets the edge `codeskimr_bench -> codeskimr_domain`; it is a dev-dependency used by the benchmarks only.

## Consequences

- Blueprint 16.1 is reworded and O-06 is closed.
- No absolute numbers are stored in the repository.
- Noisy runs remain possible; if they occur, longer measurements or option 3 need a new decision.
- The `no_literals` lint (WP-13) must skip format strings and error messages in `tools/`.
