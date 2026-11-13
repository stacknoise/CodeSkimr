# 0021 – UI themes (Hell, Petrol, Dunkel) and mocks for the team administration

Status: proposed

## Context

Blueprint 15.3 has one set of design tokens (the light look of mocks 1 to 6) and says dark mode is "planned" (open point O-02). ADR 0019 requires a mock update for the team edition screens before they are built (blueprint 15.1), because login, user, role, token and audit screens do not exist in the mocks.

While the product owner reviewed the new administration mock, he liked its petrol-coloured look and asked to offer it as a theme, with two or three themes at the start and a small preview.

## Options

1. One look only, dark mode later. No work now, but O-02 stays open and the petrol look is lost.
2. Themes as a closed set of token tables (`design/tokens.toml`), chosen per user, with a live preview in the settings. Adding a theme is data, not code.
3. Free colour editing by the user. Flexible, but unverifiable for contrast and out of scope.

## Decision

Option 2.

- Three themes in the first version: **Hell** (the existing values, blue accent), **Petrol** (light, petrol accent, the colours of the administration mock; the default, decided by the product owner on 2026-11-13) and **Dunkel** (dark, petrol-leaning accent). Values are in blueprint 15.3.
- Every theme defines every token. `tools/codegen` generates the CSS variables per theme from `design/tokens.toml` (`[data-theme="…"]`). Components use variables only (R-05 and the no-literals rule already require this).
- A CI check in the `codegen` job fails when a theme lacks a token, and when a listed text/background pair falls below 4.5:1 (the rule of 15.4). The check is the acceptance test for every future theme.
- The choice is stored per user (blueprint 13 already stores language per user); without a team server it lives in the local settings. First start: Petrol, or Dunkel when the operating system is set to dark, until the user picks one. The default key lives in `config/default.toml`.
- The settings page shows one preview card per theme. The preview renders the real components in a container with the theme's `data-theme`, not a screenshot.
- Adding a theme is a table in `tokens.toml` plus a name in `locales/`; no code change.
- The administration screens (login, users, groups and roles, tokens, audit log, single sign-on) get mocks 7 to 12 now, the settings page gets mock 13. The screens are built after 1.0 (milestone T2 of ADR 0019); themes and the settings page belong to the 1.0 UI.
- Mocks 7 to 13 take a `theme` property. Mocks 1 to 6 stay in Hell (layout binding, colours follow the active theme); mock 13 shows all three themes. This closes open point O-02.

## Consequences

- Blueprint 13, 15.1 to 15.3, Appendix B and O-02 change (separate sections from the ADR 0019 follow-up PR, which touches 10.1 and 18.4).
- Kind, layer and edge colours need dark variants; they are part of the token table and pass the same contrast check.
- Per-user storage of the theme needs the `user_settings` table of ADR 0019 only in team mode.
- Every new screen is checked in all three themes (visual test, blueprint 17.1).
- Risk: more themes mean more visual-test baselines. Mitigation: three themes only, more by ADR.
