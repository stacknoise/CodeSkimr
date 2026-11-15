# 0057 – Sandbox for level 2 indexers

Status: proposed (draft for Anton, issue #290; nothing is built yet)

## Context

Level 2 (ADR 0033; this ADR relies on its decision 11, the consent) starts the build tools of a scanned project: `scip-dotnet`, `scip-typescript`, `scip-python`, `scip-java` and others. These tools run code of the project, for example build scripts and package hooks, with the rights of the user. Today the runner (`docs/level2-runner.md`) restricts them only in these ways: an empty home folder, an allow list for the environment, own process group or job object, limits for time, memory and output, a consent of the local owner per source, location and branch, and a confirmation text that says honestly that the indexers are **not sandboxed**.

What an indexer can still do: read every file the user can read (by absolute path), use the network, write outside its output folder. The network cannot simply be switched off, because a build restores its dependencies.

The first evaluation (issue #222, `docs/level2-runner.md`, "Sandbox, evaluated") found that every route has a cost, and that a sandbox that is off on half of the systems gives a false feeling of safety. A sandbox is new technology and may need a new dependency or `unsafe`, so blueprint rule 9 asks for an ADR. This is the ADR. Only the product owner decides; nothing here is implemented.

## What a sandbox has to achieve

1. Files: the indexer may read the project folder, the toolchain and the system folders it needs, and write its output folder and the usual build folders of the project. It must not read the home folder of the user (keys, browser profiles, other projects) and must not write anywhere else.
2. Credentials over other channels than files: no access to the keystore and agent channels where CodeSkimr's own installation token and the repository credentials live (D-Bus Secret Service, macOS keychain services, Windows Credential Manager, `ssh-agent` and `gpg-agent` sockets).
3. Network: the mechanisms below can only switch the network on or off for the whole indexer. Restoring packages needs it on, so a restriction to "package restore only" is not reachable without a filtering proxy (not decided here, see the end). Loopback (the CodeSkimr server itself), link-local addresses and the local network are part of "on", so "on" is a real exposure; Landlock cannot express UDP or host names, and a restricted token (B1) does not restrict the network at all. Where a mechanism cannot even do on or off, say so.
4. Processes: no change of the rights (no `sudo`, no setuid, `no_new_privs`), no sight of or access to other processes of the user. Landlock alone does not hide processes; `bwrap` needs its own process namespace, `--new-session` and `--die-with-parent`.
5. What the indexer writes into the project folder runs later with the full rights of the user (`.git/hooks`, CI configuration, `.vscode` tasks, package scripts). Writes to such places are denied, or the indexer works on a copy of the project.
6. **Fails closed.** A mechanism that is not available on a system may degrade visibly (a warning in the confirmation text and in the scan report). A mechanism that is available but fails to apply stops the start of that indexer; it never runs unsandboxed after an error. A setting `level2.require_sandbox` makes the missing mechanism an error as well, for CI and the team server. The consent records whether it was given with or without a sandbox. A consent given with a sandbox does not carry over silently to a later run without one (for example after a kernel or tool update): the user is asked again, or the run stops when `level2.require_sandbox` is set.
7. **Reports per goal**, not "sandboxed" or not: for files, credentials, network and processes each is "enforced", "partial" or "not restricted", so the user sees what the active mechanism does and does not do.
8. The output of an indexer stays untrusted input: the SCIP importer keeps its checks (ADR 0033), whatever the sandbox does.

## Options per system

### Linux

| Option | Files | Network | Cost |
| --- | --- | --- | --- |
| A1. Landlock (crate `landlock`) | yes, from kernel 5.13 | only TCP ports, from ABI 4 (kernel 6.7); no host names | new dependency; the rules must be set between `fork` and `exec`, which in Rust is `CommandExt::pre_exec` and therefore `unsafe` in our own code, or a small helper program that sets the rules and then starts the indexer; kernel too old on many LTS systems |
| A2. `bwrap` (bubblewrap) as a program | yes (mount namespace) | on or off only (own network namespace, or none) | the user must install it; needs unprivileged user namespaces, which many distributions and containers disable |
| A3. `unshare` and a seccomp filter written by us | partly | on or off only | much own code, easy to get wrong, `unsafe` |

### Windows

| Option | Files | Network | Cost |
| --- | --- | --- | --- |
| B1. Job object with a restricted token (`CreateRestrictedToken`) | yes, by removing rights and SIDs | no | Win32 calls, `unsafe` outside `codeskimr_secrets::memory` |
| B2. AppContainer | yes (capability based) | on or off only (capability `internetClient`) | Win32 calls, `unsafe`; many build tools break inside an AppContainer (they expect the user profile and the registry) |
| B3. Windows Sandbox / a container | complete | on or off only | not on Home editions, heavy, needs virtualisation |

The job object we already use (crate `process-wrap`) restricts only limits and process lifetime, not rights.

### macOS

| Option | Files | Network | Cost |
| --- | --- | --- | --- |
| C1. `sandbox-exec` with a profile | yes | on or off only (profile rule) | marked deprecated by Apple but still working and used by many tools; a profile file to write and test per toolchain |
| C2. App Sandbox entitlements | only for the whole app | on or off only | does not fit a command line tool that starts other tools |

### Considered and not preferred

Windows low-integrity tokens (B1 is the stricter form of the same idea) and Linux `firejail` (a setuid program, itself a source of privilege bugs) are not recommended.

### Cross-platform alternative

D. A container or virtual machine (rootless Docker or Podman only, never a daemon that runs as root) as the single mechanism. One set of rules, but a large prerequisite, slow starts, and the project folder has to be mounted in, with the same trust problem for that mount.

## Recommendation

1. **1.0 stays as it is** (empty home, environment allow list, limits, consent, honest text). The risk is stated to the user at the moment of the consent, and level 2 is off by default. No sandbox is built before 1.0.
2. **After 1.0, build the sandbox per system in this order**, each as its own work package with the label `security-sensitive`:
   - Linux: A2 (`bwrap`) when it is installed and works, otherwise A1 (Landlock) for files only. The report says which one was active and that the network was not restricted when it is only Landlock below ABI 4.
   - macOS: C1 with one profile that allows the project folder, the output folder and the toolchain, denies the home folder otherwise and denies `mach-lookup` for the keychain services; a plan for the day Apple removes the deprecated `sandbox-exec` is part of that package.
   - Windows: B1 first (restricted token, its Win32 calls in `codeskimr_sandbox`), B2 only if tests with the real indexers show that they still work.
3. **Visible degradation, fail closed.** If no mechanism is available, the scan does not stop (unless `level2.require_sandbox` is set), but the confirmation text and the scan report say per goal what is not restricted, as today. A mechanism that is available but fails to apply stops that indexer (goal 6).
4. **One new place for platform code, reached through a port.** The sandbox calls live in one new adapter crate (`codeskimr_sandbox`) so that `unsafe` (the Win32 calls, and the helper program of point 5) and the new dependencies are confined and reviewed in one place. Blueprint 4.2 allows an adapter no other adapter, so the indexer runner does not call the crate: `application` gets a port (for example `ProcessSandbox`), `codeskimr_sandbox` implements it and the composition root hands it to the runner. The dependency table of 4.2 gets one new row: `sandbox` may use `application`, `domain` and `codes` only. The rule "no `unsafe` outside `codeskimr_secrets::memory`" (R-08) is changed by an amendment of blueprint 3 that names this crate as the second place.
5. **Apply the rules through a small helper program** that sets them and then starts the indexer, instead of `pre_exec` in a multithreaded process (async-signal-safety); the helper is part of `codeskimr_sandbox`. The helper is a trusted component: it is never found through a search of `PATH`, but sits next to the executable or is embedded, in the same way as `icacls.exe` is taken from the system folder.
6. **Tests.** An indexer stand-in that tries to read a file in the home folder, to ask the keystore, to connect to loopback, to write `.git/hooks`, to see other processes, to write outside its folder and to open a connection must fail on each system where the sandbox is active; the same stand-in must succeed without the sandbox, so the test shows the sandbox is what stops it. A test that is skipped because the mechanism is missing on the CI machine is reported as skipped, not as passed.

## Consequences if accepted

- Blueprint 12 gets a new section "Level 2 sandbox" after 12.5 with the goals and the rule on visible degradation; blueprint 3 names the new crate as the second place for `unsafe` (R-08); blueprint 4.2 gets the port in `application` and the row for `sandbox`; the confirmation text gets a line that names the active mechanism.
- New dependencies, at most one per system and only in `codeskimr_sandbox`: possibly `landlock` (Linux). `bwrap` and `sandbox-exec` are programs, not dependencies. Windows needs Win32 calls (`CreateProcessAsUser`, which `std::process::Command` does not offer, plus restricted tokens); `windows-sys` is already a workspace dependency (used by `fsguard` and `secrets`), so it is a new use in a new crate, not a new dependency.
- Level 2 stays optional and off by default; users on systems where no mechanism works keep today's protection and an honest text.
- The memory limit, the output limit and the process group stay; the sandbox adds to them and does not replace them.

## Not decided here

- The exact profiles and which folders of a toolchain are allowed (found per indexer during the packages).
- Whether to ship `bwrap` with the installer on Linux (licence and size to be checked).
- Whether the network is switched off by default. A build needs it for the restore, so this only works for indexers that need no restore or after a restore step outside the sandbox. A restriction to certain hosts needs a filtering proxy and is a separate decision; the recommendation is not to build one for the first version.
