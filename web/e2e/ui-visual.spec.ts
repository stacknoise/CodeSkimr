// Visual check of the screens against the mocks (blueprint 17.1, 1440 x 900, all three themes).
//
// The mocks are plain HTML with inline styles, so a browser lays them out even without the design
// canvas: the test opens the mock, adds the one rule the canvas adds (border-box), measures
// landmarks and compares them with the same landmarks of the real screen. Texts and numbers of
// the mocks are sample data and are not compared. Deviations that are known and chosen are in
// docs/ui.md; they are not listed here, so a new one fails the test.
//
// `UPDATE_SCREENSHOTS=1` writes the screens to docs/screenshots/visual/ for the review by eye.
// Runs after ui-usages.spec.ts: the worker shares one server and every scan adds a project.
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import type { Page, TestInfo } from "@playwright/test";
import {
  addLayeredProject,
  addScannedProject,
  expect,
  keepLaunchFile,
  SCAN_MS,
  selectAdd,
  test,
  type Harness,
} from "./fixtures";

const MOCKS = path.resolve(import.meta.dirname, "../../docs/mocks");
const SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots/visual");
const TOKENS = path.resolve(import.meta.dirname, "../../design/tokens.toml");
const THEMES = ["hell", "petrol", "dunkel"];
const VIEWPORT = { width: 1440, height: 900 };
/** Pixels by which a landmark may differ from the mock (rounding of fractions). */
const TOLERANCE = 1;

interface Box {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** The box of the first element that matches `selector`, or `null`. */
async function boxOf(page: Page, selector: string): Promise<Box | null> {
  return page.evaluate((query) => {
    const found = document.querySelector(query);
    if (!found) {
      return null;
    }
    const { x, y, width, height } = found.getBoundingClientRect();
    return {
      x: Math.round(x),
      y: Math.round(y),
      width: Math.round(width),
      height: Math.round(height),
    };
  }, selector);
}

/** A page with the mock `name` laid out as the design canvas does. */
async function openMock(page: Page, name: string): Promise<Page> {
  const mock = await page.context().newPage();
  await mock.setViewportSize(VIEWPORT);
  await mock.goto(`file://${MOCKS}/${name}.dc.html`);
  await mock.addStyleTag({ content: "*,*::before,*::after{box-sizing:border-box}" });
  return mock;
}

/** One thing to compare: a selector in the mock, the one in the real screen and what to compare. */
interface Landmark {
  name: string;
  mock: string;
  real: string;
  /** Which of x, y, width and height must match; all of them by default. */
  only?: (keyof Box)[];
}

const SHELL: Landmark[] = [
  { name: "top bar", mock: "header", real: "header" },
  { name: "rail", mock: "nav", real: "nav" },
  { name: "status bar", mock: "footer", real: "footer" },
];

async function compare(page: Page, mock: Page, landmarks: Landmark[]): Promise<void> {
  const problems: string[] = [];
  for (const landmark of landmarks) {
    const wanted = await boxOf(mock, landmark.mock);
    const actual = await boxOf(page, landmark.real);
    if (!wanted || !actual) {
      problems.push(
        `${landmark.name}: ${wanted ? "missing in the screen" : "missing in the mock"}`,
      );
      continue;
    }
    for (const key of landmark.only ?? (["x", "y", "width", "height"] as const)) {
      if (Math.abs(wanted[key] - actual[key]) > TOLERANCE) {
        problems.push(`${landmark.name} ${key}: mock ${wanted[key]}, screen ${actual[key]}`);
      }
    }
  }
  expect(problems).toEqual([]);
}

/** Colours and type of the frame, read from the elements themselves. */
async function frameStyle(page: Page): Promise<Record<string, string>> {
  return page.evaluate(() => {
    const read = (query: string, props: string[]) => {
      const found = document.querySelector(query);
      const style = found ? getComputedStyle(found) : null;
      return props.map((prop) => `${query}.${prop}=${style?.getPropertyValue(prop) ?? "-"}`);
    };
    return Object.fromEntries(
      [
        ...read("body", ["background-color"]),
        ...read("header", ["background-color", "border-bottom-color", "height"]),
        ...read("nav", ["background-color", "border-right-color", "width"]),
        ...read("footer", ["background-color", "border-top-color", "color", "height"]),
      ].map((entry) => entry.split(/=(.*)/s).slice(0, 2) as [string, string]),
    );
  });
}

/** The size and theme of the mocks: 1440 x 900, and light, which is how mocks 1 to 6 are drawn. */
async function prepare(page: Page, theme = "hell"): Promise<void> {
  await page.setViewportSize(VIEWPORT);
  await page.addInitScript((name) => localStorage.setItem("codeskimr.theme", name), theme);
}

async function openUi(page: Page, harness: Harness, theme = "hell"): Promise<void> {
  await prepare(page, theme);
  const { copy } = keepLaunchFile(await harness.launchFile());
  await page.goto(`file://${copy}`);
}

/** Saves the screen for the review by eye when asked to, and always attaches it to the report. */
async function shot(page: Page, name: string, info: TestInfo): Promise<void> {
  const body = await page.screenshot();
  await info.attach(name, { body, contentType: "image/png" });
  if (process.env["UPDATE_SCREENSHOTS"]) {
    mkdirSync(SHOTS, { recursive: true });
    writeFileSync(path.join(SHOTS, `${name}.png`), body);
  }
}

const SOURCE = [
  "namespace Vis;",
  "",
  "public class Calculator",
  "{",
  "    public int Add(int a, int b) { return a + b; }",
  "}",
  "",
  "public class Checkout",
  "{",
  "    public int Total(Calculator calculator) { return calculator.Add(1, 2); }",
  "}",
  "",
].join("\n");

/** One type with 40 members, so a search for "Item" fills the list of the quick search. */
const MANY_MEMBERS = [
  "namespace Many;",
  "",
  "public class Items",
  "{",
  ...Array.from({ length: 40 }, (_, n) => `    public int Item${n}() { return ${n}; }`),
  "}",
  "",
].join("\n");

// The landmarks do not move with the theme, so every screen is measured against the same mock in
// all three themes (blueprint 19, M3). The colours of the frame per theme are checked below.
for (const theme of THEMES) {
  test.describe(`the screens against the mocks (1440 x 900, ${theme})`, () => {
    test.use({ viewport: VIEWPORT });

    // The colours of the mocks 1 to 6 are those of Hell; Petrol and Dunkel are checked against the
    // token file below.
    if (theme === "hell") {
      test("the frame has the colours, sizes and type of the mocks", async ({ page, harness }) => {
        await openUi(page, harness, theme);
        await expect(page.getByText("● Connected")).toBeVisible();
        const mock = await openMock(page, "Main");
        const wanted = await frameStyle(mock);
        const actual = await frameStyle(page);
        expect(actual).toEqual(wanted);
        await compare(page, mock, SHELL);
      });
    }

    test("Projects (1)", async ({ page, harness }, info) => {
      await prepare(page, theme);
      const row = await addScannedProject(page, harness, `visprojects${theme}`, SOURCE);
      await expect(row).toBeVisible();
      const mock = await openMock(page, "Main");
      await compare(page, mock, [
        ...SHELL,
        { name: "screen", mock: "nav ~ main", real: "main" },
        { name: "form", mock: "nav ~ main form", real: "main form", only: ["x", "width"] },
        { name: "list", mock: "nav ~ main table", real: "main table", only: ["x", "width"] },
      ]);
      await shot(page, `projects-${theme}`, info);
    });

    test("Explorer (2)", async ({ page, harness }, info) => {
      await prepare(page, theme);
      const row = await addScannedProject(page, harness, `visexplorer${theme}`, SOURCE);
      await row.click({ position: { x: 4, y: 4 } });
      await page.keyboard.press("Enter");
      await page.getByText("Vis", { exact: true }).click();
      await page.getByText("Calculator", { exact: true }).click();
      await expect(page.getByRole("heading", { name: "Calculator" })).toBeVisible();
      const mock = await openMock(page, "Explorer");
      await compare(page, mock, [
        ...SHELL,
        {
          name: "tree column",
          mock: "nav ~ section:nth-of-type(1)",
          real: "main section[aria-label='Structure']",
        },
        {
          name: "source column",
          mock: "nav ~ section:nth-of-type(3)",
          real: "main section[aria-label='Source']",
        },
      ]);
      await shot(page, `explorer-${theme}`, info);
    });

    test("Quick search (4)", async ({ page, harness }, info) => {
      await prepare(page, theme);
      const row = await addScannedProject(page, harness, `vissearch${theme}`, MANY_MEMBERS);
      await row.click({ position: { x: 4, y: 4 } });
      await page.keyboard.press("Enter");
      await expect(page.getByRole("tree")).toBeVisible();
      await page.keyboard.press("Control+k");
      // With many hits the dialog is cut to 70 vh: the rows above the list keep their size.
      await page.getByRole("combobox").fill("Item");
      await expect(page.getByRole("option").first()).toBeVisible();
      const mock = await openMock(page, "Palette");
      await compare(page, mock, [
        { name: "dialog", mock: "[role=dialog]", real: "[role=dialog]", only: ["x", "y", "width"] },
        {
          name: "input row",
          mock: "[role=dialog] > div:nth-child(1)",
          real: "[role=dialog] > div:nth-child(1)",
          only: ["height"],
        },
        {
          name: "preview",
          mock: "[role=dialog] > div:nth-child(3) > div:last-child",
          real: "[role=dialog] aside",
          only: ["x", "width"],
        },
      ]);
      await shot(page, `search-${theme}`, info);
    });

    test("Settings (13)", async ({ page, harness }, info) => {
      await openUi(page, harness, theme);
      await expect(page.getByText("● Connected")).toBeVisible();
      await page.getByRole("link", { name: "Settings" }).click();
      await expect(page.getByRole("heading", { name: "Language" })).toBeVisible();
      const mock = await openMock(page, "Themes");
      await compare(page, mock, [
        ...SHELL,
        {
          name: "theme card",
          mock: "main .panel",
          real: "main section:nth-of-type(1)",
          only: ["x", "width"],
        },
        {
          name: "language card",
          mock: "main .panel[style*='max-width:560px']",
          real: "main section:has(select)",
          only: ["x", "width"],
        },
      ]);
      await shot(page, `settings-${theme}`, info);
    });

    test("Scan and performance (5)", async ({ page, harness }, info) => {
      await prepare(page, theme);
      await addScannedProject(page, harness, `visscan${theme}`, SOURCE);
      await page.getByRole("link", { name: "Scans and performance" }).click();
      await expect(page.getByRole("heading", { name: "Scan finished" })).toBeVisible();
      const mock = await openMock(page, "Scan");
      await compare(page, mock, [
        ...SHELL,
        {
          name: "progress column",
          mock: "nav ~ main > div",
          real: "main section > div",
          only: ["x", "width"],
        },
        {
          name: "side column",
          mock: "nav ~ main > aside",
          real: "main section > aside",
          only: ["x", "width"],
        },
      ]);
      await shot(page, `scan-${theme}`, info);
    });

    test("Usages (6)", async ({ page, harness }, info) => {
      await prepare(page, theme);
      const row = await addScannedProject(page, harness, `visusages${theme}`, SOURCE);
      await selectAdd(page, row, "Vis");
      await page.keyboard.press("u");
      await expect(page.getByRole("region", { name: "Usage graph" })).toBeVisible();
      await expect(page.getByText("Hits 1")).toBeVisible();
      const mock = await openMock(page, "Usages");
      await compare(page, mock, [
        ...SHELL,
        { name: "graph column", mock: "nav ~ section", real: "main section", only: ["x", "width"] },
        {
          name: "toolbar",
          mock: "nav ~ section > div:nth-child(1)",
          real: "main section > div:nth-child(1)",
          only: ["x", "width"],
        },
        { name: "side panel", mock: "nav ~ aside", real: "main aside" },
      ]);
      await shot(page, `usages-${theme}`, info);
    });

    test("Dependencies (3)", async ({ page, harness }, info) => {
      await openUi(page, harness, theme);
      await expect(page.getByText("● Connected")).toBeVisible();
      const open = await addLayeredProject(page, harness, `visgraph${theme}`);
      await open.click();
      await page.getByRole("link", { name: "Dependencies" }).click();
      await page.getByRole("button", { name: "Domain" }).click();
      await expect(page.getByRole("heading", { name: "Domain" })).toBeVisible({ timeout: SCAN_MS });
      const mock = await openMock(page, "Graph");
      await compare(page, mock, [
        ...SHELL,
        {
          name: "map",
          mock: "nav ~ section",
          real: "main section > div:first-of-type",
          only: ["x", "width"],
        },
        { name: "side panel", mock: "nav ~ aside", real: "main aside" },
      ]);
      await shot(page, `graph-${theme}`, info);
    });
  });
}

/** The colour tokens of one theme from design/tokens.toml (blueprint 15.3), as `{ token: "#RRGGBB" }`. */
function themeTokens(theme: string): Record<string, string> {
  const text = readFileSync(TOKENS, "utf8");
  const start = text.indexOf(`[themes.${theme}]`);
  if (start === -1) {
    throw new Error(`design/tokens.toml has no theme ${theme}`);
  }
  const rest = text.slice(start + 1);
  const end = rest.indexOf("\n[");
  const body = rest.slice(0, end === -1 ? undefined : end);
  return Object.fromEntries(
    [...body.matchAll(/^([\w-]+) = "(#[0-9A-Fa-f]{6})"/gm)].map((found) => [found[1], found[2]]),
  );
}

/** `#RRGGBB` as the browser prints a computed colour. */
function rgb(hex: string): string {
  const [r, g, b] = [1, 3, 5].map((at) => parseInt(hex.slice(at, at + 2), 16));
  return `rgb(${r}, ${g}, ${b})`;
}

// Every theme is drawn with its own tokens (the mocks of 1 to 6 are Hell only, so Petrol and Dunkel
// are checked against the token file of 15.3), and the landmarks do not move with the theme.
for (const theme of THEMES) {
  test.describe(`the frame in the theme ${theme} (1440 x 900)`, () => {
    test.use({ viewport: VIEWPORT });

    test("has the colours of the theme and the sizes of the mocks", async ({
      page,
      harness,
    }, info) => {
      await openUi(page, harness, theme);
      await expect(page.getByText("● Connected")).toBeVisible();
      const tokens = themeTokens(theme);
      const actual = await frameStyle(page);
      expect(actual["body.background-color"]).toBe(rgb(tokens["bg"] ?? ""));
      expect(actual["header.background-color"]).toBe(rgb(tokens["surface"] ?? ""));
      expect(actual["header.border-bottom-color"]).toBe(rgb(tokens["line"] ?? ""));
      expect(actual["nav.background-color"]).toBe(rgb(tokens["surface"] ?? ""));
      expect(actual["nav.border-right-color"]).toBe(rgb(tokens["line"] ?? ""));
      expect(actual["footer.background-color"]).toBe(rgb(tokens["surface"] ?? ""));
      expect(actual["footer.border-top-color"]).toBe(rgb(tokens["line"] ?? ""));
      await compare(page, await openMock(page, "Main"), SHELL);
      await shot(page, `frame-${theme}`, info);
    });
  });
}
