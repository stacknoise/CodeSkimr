import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { createInterface } from "node:readline";
import { test as base, expect, type Locator, type Page, type TestInfo } from "@playwright/test";

/** The server of the tests: the real one, over an in-memory keystore (crates/app/src/e2e.rs). */
export interface Harness {
  /** Address of the server, such as `http://127.0.0.1:41234`. */
  url: string;
  /** Asks the server for a new launch code and returns the file that carries it. */
  launchFile(): Promise<string>;
  /** Forgets the theme and language the server stored, so that a test starts as a new installation. */
  resetPreferences(): Promise<void>;
}

const REPO = path.resolve(import.meta.dirname, "../..");
const STARTUP_MS = 600_000;

/** Starts the test server of the Rust side: the shell that prints the markers of the harness. */
function spawnServer(): ChildProcessWithoutNullStreams {
  return spawn(
    "cargo",
    ["test", "-p", "codeskimr", "e2e_server", "--", "--ignored", "--nocapture", "--test-threads=1"],
    {
      cwd: REPO,
      env: { ...process.env, CODESKIMR_E2E_UI_DIR: path.resolve(REPO, "web/dist") },
    },
  );
}

/** The calls of the harness: each writes a command to the server and waits for its answer. */
function harnessOf(
  child: ChildProcessWithoutNullStreams,
  url: string,
  waiting: ((file: string) => void)[],
  resets: (() => void)[],
): Harness {
  return {
    url,
    launchFile: () =>
      new Promise((done) => {
        waiting.push(done);
        child.stdin.write("launch\n");
      }),
    resetPreferences: () =>
      new Promise((done) => {
        resets.push(done);
        child.stdin.write("reset\n");
      }),
  };
}

function start(): Promise<{ child: ChildProcessWithoutNullStreams; harness: Harness }> {
  const child = spawnServer();
  const waiting: ((file: string) => void)[] = [];
  const resets: (() => void)[] = [];
  return new Promise((resolve, reject) => {
    child.on("error", reject);
    child.on("exit", (code) => reject(new Error(`the test server stopped (${code})`)));
    // The test runner puts its own text in front of the first line, so the markers are searched.
    createInterface({ input: child.stdout }).on("line", (line) => {
      const url = /E2E_URL=(\S+)/.exec(line)?.[1];
      const file = /E2E_LAUNCH_FILE=(.+)$/.exec(line)?.[1];
      if (line.includes("E2E_RESET=done")) {
        resets.shift()?.();
      } else if (url !== undefined) {
        resolve({ child, harness: harnessOf(child, url, waiting, resets) });
      } else if (file !== undefined) {
        waiting.shift()?.(file);
      }
    });
  });
}

export const test = base.extend<{ freshInstallation: void }, { harness: Harness }>({
  // Every test starts without stored preferences, as the first start of a new installation.
  freshInstallation: [
    async ({ harness }, use) => {
      await harness.resetPreferences();
      await use();
    },
    { auto: true },
  ],
  harness: [
    // eslint-disable-next-line no-empty-pattern
    async ({}, use) => {
      const { child, harness } = await start();
      await use(harness);
      child.removeAllListeners("exit");
      child.stdin.end();
    },
    { scope: "worker", timeout: STARTUP_MS },
  ],
});

export { expect };

/** The text of a launch file, and a copy of it that stays when the server deletes the original. */
export function keepLaunchFile(file: string): { copy: string; text: string } {
  const copy = path.join(mkdtempSync(path.join(tmpdir(), "codeskimr-e2e-")), "launch-copy.html");
  copyFileSync(file, copy);
  return { copy, text: readFileSync(copy, "utf8") };
}

/** Records every violation of the content security policy of the page. */
export async function watchPolicy(page: Page): Promise<() => Promise<string[]>> {
  await page.addInitScript(() => {
    const seen: string[] = [];
    (window as unknown as { __violations: string[] }).__violations = seen;
    document.addEventListener("securitypolicyviolation", (event) => {
      seen.push(`${event.violatedDirective} ${event.blockedURI}`);
    });
  });
  return () => page.evaluate(() => (window as unknown as { __violations: string[] }).__violations);
}

/** The states a project has once its scan is over: current, or a tool is missing (the machine of
 * a test has no indexer), or stage 2 failed. */
export const SCANNED_STATE = /^(current|\d+ tools? missing|Exact check failed)$/;

/** A scan in a debug build loads the language packs and takes a few seconds. */
export const SCAN_MS = 60_000;

/** Adding a project shows the tab "Scan" of its page once the stream reports the scan; the list
 * names the project from then on. */
export async function leaveScanScreen(page: Page, name: string): Promise<void> {
  await expect(page.getByRole("button", { name: `Open ${name}` })).toBeVisible({
    timeout: SCAN_MS,
  });
}

/** The label of the source field of the window "New project". */
export const SOURCE_LABEL = /^(Folder|Repository address)$/;

/** Opens the form "New project": it is open on its own only while the list is empty. */
export async function openNewForm(page: Page): Promise<void> {
  await page.getByRole("button", { name: "New", exact: true }).click();
}

/** Shows the explorer for the project whose page is shown. */
export async function openInExplorer(page: Page): Promise<void> {
  await page.getByRole("button", { name: "Open in explorer" }).click();
}

/** Shows a tab of the project page. */
export async function showTab(page: Page, name: string | RegExp): Promise<void> {
  await page.getByRole("tab", { name }).click();
}

/** Fills the form "New project" with the folder and sends it. */
export async function submitFolder(page: Page, folder: string): Promise<void> {
  await openNewForm(page);
  await page.getByLabel(SOURCE_LABEL).fill(folder);
  await page.getByRole("button", { name: "Add and scan" }).click();
}

/** Waits until the list names the project `name` with its finished scan; returns its row. */
export async function listedProject(page: Page, name: string) {
  await leaveScanScreen(page, name);
  const open = page.getByRole("button", { name: `Open ${name}` });
  await expect(open.getByRole("img", { name: SCANNED_STATE })).toBeVisible({ timeout: SCAN_MS });
  return open;
}

/** Opens the exact check of a project that is added and selected; the tab is shown afterwards. */
export async function selectExactCheck(
  page: Page,
  harness: Harness,
  name: string,
  source: string,
): Promise<void> {
  await selectScannedProject(page, harness, name, source);
  await showTab(page, /Exact check/);
}

/** Opens the UI, adds a folder with one C# file as the project `name` and waits until its scan
 * is listed. Returns the row of the project. */
export async function addScannedProject(
  page: Page,
  harness: Harness,
  name: string,
  source: string,
) {
  const folder = path.join(mkdtempSync(path.join(tmpdir(), "codeskimr-e2e-")), name);
  mkdirSync(folder);
  writeFileSync(path.join(folder, `${name}.cs`), source);
  const { copy } = keepLaunchFile(await harness.launchFile());
  await page.goto(`file://${copy}`);
  await submitFolder(page, folder);
  return listedProject(page, name);
}

/** Adds a scanned project like `addScannedProject` and selects it in the project list, so that its
 * page with the tabs (overview, scan, exact check, tools) is shown. */
export async function selectScannedProject(
  page: Page,
  harness: Harness,
  name: string,
  source: string,
): Promise<void> {
  const row = await addScannedProject(page, harness, name, source);
  await row.click({ position: { x: 4, y: 4 } });
}

/** Adds a scanned project like `addScannedProject` and opens it (the screens of a project stay shut
 * until one is open); the explorer is shown afterwards. */
export async function openScannedProject(
  page: Page,
  harness: Harness,
  name: string,
  source: string,
) {
  const row = await addScannedProject(page, harness, name, source);
  await row.click({ position: { x: 4, y: 4 } });
  await openInExplorer(page);
  await expect(page.getByRole("tree")).toBeVisible();
}

/** The layer file and the two sources of a small project with a Domain and a Presentation layer. */
function layeredFiles(prefix: string): Record<string, string> {
  return {
    ".codeskimr.toml": [
      `[[layers]]\nname = "Presentation"\nnamespaces = ["${prefix}.Ui"]`,
      `[[layers]]\nname = "Domain"\nnamespaces = ["${prefix}.Domain"]`,
    ].join("\n\n"),
    "Order.cs": `namespace ${prefix}.Domain;\n\npublic class Order\n{\n    public int Total() { return 1; }\n}\n`,
    "OrderView.cs": `namespace ${prefix}.Ui;\n\nusing ${prefix}.Domain;\n\npublic class OrderView\n{\n    public int Show(Order order) { return order.Total(); }\n}\n`,
  };
}

/** Opens the UI (unless `page` shows it already), adds a project with the layers Domain and
 * Presentation as `name` (namespaces `<Name>.Domain` and `<Name>.Ui`), waits for its scan and
 * returns the button that opens it. */
export async function addLayeredProject(page: Page, harness: Harness, name: string) {
  const folder = path.join(mkdtempSync(path.join(tmpdir(), "codeskimr-e2e-")), name);
  mkdirSync(folder);
  for (const [file, text] of Object.entries(layeredFiles(name))) {
    writeFileSync(path.join(folder, file), text);
  }
  if (!page.url().startsWith(harness.url)) {
    const { copy } = keepLaunchFile(await harness.launchFile());
    await page.goto(`file://${copy}`);
  }
  await submitFolder(page, folder);
  return listedProject(page, name);
}

/** From a project row: opens it in the explorer and selects the member `Add` of the type
 * `Calculator` in `namespace` (the project has no layers). */
export async function selectAdd(page: Page, row: Locator, namespace: string): Promise<void> {
  await row.click({ position: { x: 4, y: 4 } });
  await openInExplorer(page);
  await page.getByText(namespace, { exact: true }).click();
  await page.getByText("Calculator", { exact: true }).click();
  await page.getByRole("button", { name: /Add/ }).first().click();
}

/** Attaches the screen to the report and, when `UPDATE_SCREENSHOTS` is set, writes it to `dir` for
 * the review by eye. */
export async function shot(page: Page, dir: string, name: string, info: TestInfo): Promise<void> {
  const body = await page.screenshot();
  await info.attach(name, { body, contentType: "image/png" });
  if (process.env["UPDATE_SCREENSHOTS"]) {
    mkdirSync(dir, { recursive: true });
    writeFileSync(path.join(dir, `${name}.png`), body);
  }
}
