# 0098 – The settings screen has five tabs

Status: accepted (variant B chosen by the product owner on 2026-11-24, card "Einstellungen-Seite übersichtlicher gestalten"; scheduled for 1.1; the mocks for 1.1 were approved by him on 2026-11-26)

## Context

Screen 13 (blueprint B.13) grew from two cards (theme, language) to eight blocks one below the other: theme, language, layers (a link), tools for stage 2, Git logins, error codes (a link), the editor link and the table of the effective settings with about 60 values in 17 groups. The table pushes everything else far down, two blocks are only a jump to another screen, and the things used often (theme, language) rank equal with look-up material (effective values, error codes). The product owner looked at three variants (A: list on the left, B: tabs on top, C: one page with foldable sections) and chose B because it matches the tabs of the project page (ADR 0096). The concept and the mocks are in the project folder (`entscheidungen/einstellungen-aufbau.md`, mocks `einstellungen-b1` and `einstellungen-b2`, now mock 38). Blueprint B.13 describes only theme and language, so a change is needed (CLAUDE.md rule 9).

## Options

1. A: a narrow list of the five sections on the left, the chosen one on the right.
2. B: the same five sections as tabs on top, with one search field at the end of the tab bar.
3. C: one page, each section a foldable row with a summary, a search over everything.

## Decision

Option 2.

1. **Tabs** (`role=tablist`, `tab`, `tabpanel`; the arrow keys move between them and wrap around, the same pattern as the other tab rows), in this order:
   - *Appearance:* theme cards with live preview and the switch "Follow system setting"; language.
   - *Analysis:* layers (the way to the layer editor); the tools for stage 2 (the table of ADR 0033, unchanged in content).
   - *Access:* the Git logins (ADR 0075); the editor link (read-only).
   - *Effective settings:* the table of effective values of ADR 0090 with the switch "Only values that differ from the default" and the count; read-only, unchanged in content.
   - *Info:* the configuration file that was read and the `--set` flags of the start; the way to the error codes. The version of the program stays in the header, next to the title, as in the mock.
2. **Search.** One search field at the end of the tab bar. It narrows the table of the effective settings (the only place with many values) and shows that tab as soon as something is typed. The search field that stood inside the table is gone; the table and the bar share one text. Searching the other tabs is not part of this decision: they hold a few cards each.
3. **State.** The shown tab is kept in memory like the tab of the project page: it stays while the page is open, is not part of the address and is "Appearance" again after a reload.
4. **Room for more.** Later content (the speed limit per indexer, the settings that can be changed from the UI, ADR 0092) lands in "Analysis" or in a new tab. With seven or eight tabs the bar gets tight; then variant A is the next step, and the five groups stay the same.
5. **Blueprint.** B.13 describes the five tabs; the mock is mock 38 (`docs/mocks/SettingsTabs.html`, a plain page for the tabs *Appearance* and *Analysis* in all three themes).

## Consequences

- The screen is shorter and the tables no longer push the theme choice out of view. A value in the effective table is one click (or one typed word) away.
- Tests and the manual say where a block lives now. The visual test of mock 13 keeps comparing the theme and language cards, which stay the first blocks of the first tab.
- No change in the server, the commands or the stored settings.
