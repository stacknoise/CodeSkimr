# UI mocks

Binding UI reference for the web frontend (blueprint section 15). One file per screen, plus `canvas.json` (position of each screen on the design canvas).

| # | Screen | File |
|---|---|---|
| 1 | Projects | `Main.dc.html` |
| 2 | Explorer | `Explorer.dc.html` |
| 3 | Dependencies | `Graph.dc.html` |
| 4 | Quick search (Ctrl+K) | `Palette.dc.html` |
| 5 | Scan and performance | `Scan.dc.html` |
| 6 | Usages (symbol level) | `Usages.dc.html` |
| 7 | Sign-in (team server) | `Login.dc.html` |
| 8 | Users and project grants | `Users.dc.html` |
| 9 | Groups and roles | `Roles.dc.html` |
| 10 | Access tokens | `Tokens.dc.html` |
| 11 | Audit log | `Audit.dc.html` |
| 12 | Single sign-on | `Sso.dc.html` |
| 13 | Settings: themes and language | `Themes.dc.html` |

Mocks 7 to 13 have a `theme` property (`hell`, `petrol`, `dunkel`; values in blueprint 15.3, ADR 0021). Mocks 1 to 6 are drawn in Hell. Screens 7 to 12 belong to the team edition (ADR 0019), screen 13 to 1.0.

The files are written in a design-canvas format and do not render on their own in a browser. Read them for exact values (inline styles); to look at the rendered screens, open the published canvas: <https://claude.ai/artifact/FRpzF7hdgZVdHCm2mxRuNZ> (the link works for the owner of the canvas).

All names and numbers in the mocks are sample data.

The `THEMES` block at the top of each script in mocks 7 to 13 (and the miniatures in mock 13) holds the same values; when a theme value changes in blueprint 15.3, update all of them together. Petrol is the default theme of the product and is drawn as selected.
