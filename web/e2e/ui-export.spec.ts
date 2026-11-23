// The export button of the top bar and its quick-search entry (blueprint 10.2, 15.4).
// `UPDATE_SCREENSHOTS=1` writes the screens to docs/screenshots/export/.
import { readFileSync } from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { expect, openScannedProject, shot, test } from "./fixtures";

const SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots/export");
const THEMES = ["hell", "petrol", "dunkel"];
const SERVER_API = "/api/v1/";
const SECRET_BODY = "return a + b;";
const SOURCE = `namespace Shop;\n\npublic class Basket\n{\n    public int Total(int a, int b) { ${SECRET_BODY} }\n}\n`;

for (const theme of THEMES) {
  test(`downloads the page of the open project (${theme})`, async ({ page, harness }, info) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.addInitScript((name) => localStorage.setItem("codeskimr.theme", name), theme);
    const name = `export${theme}`;
    await openScannedProject(page, harness, name, SOURCE);

    const button = page.getByRole("button", { name: "Export HTML" });
    await expect(button).toBeEnabled();
    await shot(page, SHOTS, `export-knopf-${theme}`, info);

    const download = page.waitForEvent("download");
    await button.click();
    const file = await download;
    expect(file.suggestedFilename()).toBe(`${name}.html`);
    const html = readFileSync((await file.path()) ?? "", "utf8");
    expect(html).toContain("Basket");
    expect(html).not.toContain(SECRET_BODY);
    expect(html).not.toContain(SERVER_API);
  });
}

test("the downloaded page opens from disk and asks no server", async ({ page, harness }) => {
  await openScannedProject(page, harness, "exportoffline", SOURCE);
  const download = page.waitForEvent("download");
  await page.getByRole("button", { name: "Export HTML" }).click();
  const saved = (await (await download).path()) ?? "";
  const offline = await page.context().newPage();
  const requests: string[] = [];
  offline.on("request", (request) => requests.push(request.url()));
  await offline.goto(pathToFileURL(saved).href);
  await expect(offline.getByText("Basket").first()).toBeVisible();
  expect(requests.filter((url) => !url.startsWith("file:"))).toEqual([]);
  expect(requests.filter((url) => url.includes(SERVER_API))).toEqual([]);
});

test("the quick search offers the export", async ({ page, harness }) => {
  await openScannedProject(page, harness, "exportsearch", SOURCE);
  await page.keyboard.press("Control+k");
  await page.getByRole("combobox").fill(">export");
  const download = page.waitForEvent("download");
  await page.getByRole("option", { name: /Export project as HTML/ }).click();
  expect((await download).suggestedFilename()).toBe("exportsearch.html");
});
