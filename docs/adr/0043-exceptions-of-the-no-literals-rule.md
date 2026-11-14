# 0043 – Exceptions of the no_literals rule

Status: accepted

## Context

Rule R-06 (blueprint 3.2) lists four exceptions: the integers `0` and `1`, test code, the value side of a constant in `consts.rs`/`consts.ts`, and build scripts. The Dylint lint `no_literals` (`tools/lints/no_literals`) behaves in four more cases that the blueprint does not name (issue #24, found in the review of the lint):

- developer tooling below `tools/` (code generator, bench and CI tools) is not checked;
- generated code in `OUT_DIR` is not checked;
- literals in attributes and doc comments are allowed (`#[must_use = "..."]`, `#[doc = "..."]`);
- code that the Linux job does not compile (`cfg(feature)` that is off, other platforms) is not checked.

Without a decision these are silent gaps. A reader of the blueprint would think the rule is stricter than the lint.

## Options

1. Tighten the lint until it reports all four cases. Tooling and generated code are not product code, attributes cannot take a constant in several positions, and the Windows and macOS builds already compile their own code, so this would produce noise and no safety.
2. Write the four cases into blueprint 3.2 as exceptions and keep the lint as it is.

## Decision

Option 2. Blueprint 3.2 gets these exceptions in addition to the four it has:

- developer tooling in `tools/` and code generated into `OUT_DIR`, because neither ships in the product;
- literals inside attributes and doc comments;
- code behind `cfg(feature)` or another platform is checked only where CI compiles it. The `literals` job runs on Linux with the default features; the platform builds (Windows, macOS) do not run the lint.

Silencing the lint is not an exception. `allow`, `expect`, `warn` or `cfg_attr` of `no_literals` in product code, in a `[lints]` table or in a compiler flag fails the `literals` job (issue #25).

The same pull request lets `deps` check the lint workspace (about 120 packages) with the rules of the product, because it was not covered before (issue #25). That check found one advisory: `paste` is unmaintained (RUSTSEC-2024-0436, no vulnerability). It is a build-time dependency of `dylint_linting`, the newest version of which still uses it, and it never reaches the product. `deny.toml` and the audit step of the lint workspace ignore exactly this advisory, with the reason next to it; the entry goes away when Dylint drops `paste`. Nothing else is loosened.

## Consequences

- Blueprint 3.2 and `docs/ci.md` say the same as the lint does.
- A literal in platform-specific code can slip through until the lint runs on that platform. Platform code stays small and goes through review; running the lint on the other platforms is a possible later check.
- The remaining gaps of the lint itself (macros from other workspace crates, the TypeScript part) are tracked in issue #26 and are bugs, not exceptions.
