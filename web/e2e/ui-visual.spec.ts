// Visual check of the screens against the mocks (blueprint 17.1, 1440 x 900, light theme).
//
// The mocks are plain HTML with inline styles, so a browser lays them out even without the design
// canvas: the test opens the mock, adds the one rule the canvas adds (border-box), measures
// landmarks and compares them with the same landmarks of the real screen. Texts and numbers of
// the mocks are sample data and are not compared. Deviations that are known and chosen are in
// docs/ui.md; they are not listed here, so a new one fails the test.
//
// `UPDATE_SCREENSHOTS=1` writes the screens to docs/screenshots/visual/ for the review by eye.
// Runs after ui-usages.spec.ts: the worker shares one server and every scan adds a project.
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import type { Page } from "@playwright/test";
import { addScannedProject, expect, keepLaunchFile, SCAN_MS, test, type Harness } from "./fixtures";

const MOCKS = path.resolve(import.meta.dirname, "../../docs/mocks");
const SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots/visual");
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
async function prepare(page: Page): Promise<void> {
  await page.setViewportSize(VIEWPORT);
  await page.addInitScript(() => localStorage.setItem("codeskimr.theme", "hell"));
}

async function openUi(page: Page, harness: Harness): Promise<void> {
  await prepare(page);
  const { copy } = keepLaunchFile(await harness.launchFile());
  await page.goto(`file://${copy}`);
}

/** Saves the screen for the review by eye when asked to, and always attaches it to the report. */
async function shot(page: Page, name: string, info: { attach: Function }): Promise<void> {
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

const LAYERS = `
[[layers]]
name = "Presentation"
namespaces = ["Vis.Ui"]

[[layers]]
name = "Domain"
namespaces = ["Vis.Domain"]
`;

test.describe("the screens against the mocks (1440 x 900, light)", () => {
  test.use({ viewport: VIEWPORT });

  test("the frame has the colours, sizes and type of the mocks", async ({ page, harness }) => {
    await openUi(page, harness);
    await expect(page.getByText("● Connected")).toBeVisible();
    const mock = await openMock(page, "Main");
    const wanted = await frameStyle(mock);
    const actual = await frameStyle(page);
    // The mock has no border on the top bar's own height: compare what is drawn.
    expect(actual).toEqual(wanted);
    await compare(page, mock, SHELL);
  });

  test("Projects (1)", async ({ page, harness }, info) => {
    await prepare(page);
    const row = await addScannedProject(page, harness, "visprojects", SOURCE);
    await expect(row).toBeVisible();
    const mock = await openMock(page, "Main");
    await compare(page, mock, [
      ...SHELL,
      { name: "screen", mock: "nav ~ main", real: "main" },
      { name: "form", mock: "nav ~ main form", real: "main form", only: ["x", "width"] },
      { name: "list", mock: "nav ~ main table", real: "main table", only: ["x", "width"] },
    ]);
    await shot(page, "projects-hell", info);
  });

  test("Explorer (2)", async ({ page, harness }, info) => {
    await prepare(page);
    const row = await addScannedProject(page, harness, "visexplorer", SOURCE);
    await row.click({ position: { x: 4, y: 4 } });
    await page.keyboard.press("Enter");
    await page.getByText("(no layer)").click();
    await page.getByText("visexplorer.Vis", { exact: true }).click();
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
    await shot(page, "explorer-hell", info);
  });

  test("Quick search (4)", async ({ page, harness }, info) => {
    await openUi(page, harness);
    await expect(page.getByText("● Connected")).toBeVisible();
    await page.keyboard.press("Control+k");
    await expect(page.getByRole("dialog")).toBeVisible();
    const mock = await openMock(page, "Palette");
    await compare(page, mock, [
      { name: "dialog", mock: "[role=dialog]", real: "[role=dialog]", only: ["x", "y", "width"] },
      {
        name: "preview",
        mock: "[role=dialog] > div:nth-child(3) > div:last-child",
        real: "[role=dialog] aside",
        only: ["x", "width"],
      },
      {
        name: "input row",
        mock: "[role=dialog] > div:nth-child(1)",
        real: "[role=dialog] > div:nth-child(1)",
        only: ["height"],
      },
    ]);
    await shot(page, "search-hell", info);
  });

  test("Scan and performance (5)", async ({ page, harness }, info) => {
    await prepare(page);
    await addScannedProject(page, harness, "visscan", SOURCE);
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
    await shot(page, "scan-hell", info);
  });

  test("Usages (6)", async ({ page, harness }, info) => {
    await prepare(page);
    const row = await addScannedProject(page, harness, "visusages", SOURCE);
    await row.click({ position: { x: 4, y: 4 } });
    await page.keyboard.press("Enter");
    await page.getByText("(no layer)").click();
    await page.getByText("visusages.Vis", { exact: true }).click();
    await page.getByText("Calculator", { exact: true }).click();
    await page.getByRole("button", { name: /Add/ }).first().click();
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
    await shot(page, "usages-hell", info);
  });

  test("Dependencies (3)", async ({ page, harness }, info) => {
    await openUi(page, harness);
    await expect(page.getByText("● Connected")).toBeVisible();
    const folder = path.join(mkdtempSync(path.join(tmpdir(), "codeskimr-e2e-")), "visgraph");
    mkdirSync(folder);
    writeFileSync(path.join(folder, ".codeskimr.toml"), LAYERS);
    writeFileSync(
      path.join(folder, "Order.cs"),
      "namespace Vis.Domain;\n\npublic class Order\n{\n    public int Total() { return 1; }\n}\n",
    );
    writeFileSync(
      path.join(folder, "OrderView.cs"),
      "namespace Vis.Ui;\n\nusing Vis.Domain;\n\npublic class OrderView\n{\n    public int Show(Order order) { return order.Total(); }\n}\n",
    );
    await page.getByLabel("New project").fill(folder);
    await page.getByRole("button", { name: "Add and scan" }).click();
    const open = page.getByRole("button", { name: "Open visgraph" });
    await expect(open).toBeVisible({ timeout: SCAN_MS });
    await expect(
      page.getByRole("row", { name: /visgraph/ }).getByText("current", { exact: true }),
    ).toBeVisible({ timeout: SCAN_MS });
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
    await shot(page, "graph-hell", info);
  });
});
