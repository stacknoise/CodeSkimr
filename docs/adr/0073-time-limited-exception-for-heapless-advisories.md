# 0073 – Time-limited exception for the `heapless` 0.8 advisories

Status: accepted

## Context

On 2026-11-18 nine advisories for the crate `heapless` were published: RUSTSEC-2026-0574, 0575, 0576, 0577, 0578, 0579, 0580, 0581 and 0582. All are of the kind "unsound" (informational, no known exploit in any real program). They affect `heapless` before 0.9.4; the patched version 0.9.4 appeared on the same day. Since then the required check `deps` fails on every pull request, because `cargo audit --deny warnings` reports them. The same nine appear in `Cargo.lock` of the product and in the lock file of `tools/accuracy/import_scip`.

`heapless` 0.8.0 reaches us through exactly one path: `gix` -> `gix-hash` 0.28.1 and `gix-packetline` 0.24.1 -> `faster-hex` 0.10.1 -> `heapless ^0.8`. `faster-hex` 0.10.x requires `heapless` 0.8 (a hard requirement, no feature switches it off), so `cargo update` cannot reach 0.9.4. `faster-hex` 1.0.0 (2026-09-23) no longer needs `heapless`, but both `gix-hash` and `gix-packetline` pin `faster-hex ^0.10.1` in every released version, including the newest `gix` 0.90.0. The `main` branch of gitoxide already uses `faster-hex 1.0.0`, but there is no release with it yet. So the fix has to come from upstream.

## Is `heapless` reachable in the release binary?

The affected functions are in `sorted_linked_list`, `spsc`, `IndexMap`, `Deque`, `HistoryBuf`, the object pools and `vec::IntoIter`. `faster-hex` 0.10.1 uses `heapless` in one place only: `heapless::String` and `heapless::Vec` in `encode.rs`, in the variant of `hex_string` for builds without the `alloc` feature. It uses none of the affected types or functions. Of the nine advisories, only 0577 names a function of `Vec` (`vec::IntoIter::drop`), and `faster-hex` never turns its `Vec` into an iterator. The pools (0580) and the concurrency problem (0575) are not used either. `heapless` is compiled in, but none of the faulty code paths can be reached from CodeSkimr.

## Options

1. **Wait for upstream and keep `deps` red.** Not possible: no pull request can be merged.
2. **Patch or fork `faster-hex` or `gix`** (`[patch]`, Git source). Breaks the rule `unknown-git = "deny"`, means maintaining a fork of Git internals, and is out of proportion for an advisory without a reachable path.
3. **Replace `gix`.** Large work package, blocks everything.
4. **Time-limited exception** for exactly these nine advisories, with the reason next to it and a review date (pattern of ADR 0043 and 0069).

## Decision

Option 4. `deny.toml` and all `cargo audit` steps of the `deps` job that see a lock file with `heapless` 0.8.0 (product and `import_scip`) ignore exactly the nine identifiers listed above, **until 2027-02-28**. Nothing else is loosened: `--deny warnings` stays, and every other advisory, yanked crate or license problem still fails the check. A new advisory for `heapless` (another identifier) or a vulnerability (not "unsound") is not covered.

## Consequences

- As soon as a `gix` release uses `faster-hex` 1.x, the dependency is updated, the nine entries are removed and this ADR is set to "superseded". Until then the exception stays in place.
- On or before 2027-02-28 the exception is reviewed: removed, or renewed by a new ADR.
- No product code, no behaviour and no dependency changes.
