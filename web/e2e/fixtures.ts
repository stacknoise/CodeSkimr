import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { createInterface } from "node:readline";
import { test as base, expect, type Page } from "@playwright/test";

/** The server of the tests: the real one, over an in-memory keystore (crates/app/src/e2e.rs). */
export interface Harness {
  /** Address of the server, such as `http://127.0.0.1:41234`. */
  url: string;
  /** Asks the server for a new launch code and returns the file that carries it. */
  launchFile(): Promise<string>;
}

const REPO = path.resolve(import.meta.dirname, "../..");
const STARTUP_MS = 600_000;

function start(): Promise<{ child: ChildProcessWithoutNullStreams; harness: Harness }> {
  const child = spawn(
    "cargo",
    ["test", "-p", "codeskimr", "e2e_server", "--", "--ignored", "--nocapture", "--test-threads=1"],
    {
      cwd: REPO,
      env: { ...process.env, CODESKIMR_E2E_UI_DIR: path.resolve(REPO, "web/dist") },
    },
  );
  const waiting: ((file: string) => void)[] = [];
  return new Promise((resolve, reject) => {
    child.on("error", reject);
    child.on("exit", (code) => reject(new Error(`the test server stopped (${code})`)));
    // The test runner puts its own text in front of the first line, so the markers are searched.
    createInterface({ input: child.stdout }).on("line", (line) => {
      const url = /E2E_URL=(\S+)/.exec(line)?.[1];
      const file = /E2E_LAUNCH_FILE=(.+)$/.exec(line)?.[1];
      if (url !== undefined) {
        resolve({
          child,
          harness: {
            url,
            launchFile: () =>
              new Promise((done) => {
                waiting.push(done);
                child.stdin.write("launch\n");
              }),
          },
        });
      } else if (file !== undefined) {
        waiting.shift()?.(file);
      }
    });
  });
}

export const test = base.extend<object, { harness: Harness }>({
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

/** A scan in a debug build loads the language packs and takes a few seconds. */
export const SCAN_MS = 60_000;

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
  const add = page.getByRole("button", { name: "Add and scan" });
  await page.getByLabel("New project").fill(folder);
  await add.click();
  const open = page.getByRole("button", { name: `Open ${name}` });
  await expect(open).toBeVisible({ timeout: SCAN_MS });
  const row = page.getByRole("row", { name: new RegExp(name) });
  await expect(row.getByText("current", { exact: true })).toBeVisible({ timeout: SCAN_MS });
  return row;
}
