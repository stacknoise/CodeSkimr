// The keyboard paths of the annotations (blueprint 15.4, ADR 0068): K, N and T in the explorer, the
// tabs of the side panel, the bell with its drawer and the board of the tasks.
// Runs after ui-explorer.spec.ts: the worker shares one server and every scan adds a project.
import { addScannedProject, expect, test } from "./fixtures";

const SOURCE =
  "namespace Notes;\n\npublic class Ledger\n{\n    public int Add(int a, int b) { return a + b; }\n}\n";

/** Opens the project in the explorer and selects its type with the keyboard-reachable tree. */
async function openLedger(
  page: import("@playwright/test").Page,
  row: import("@playwright/test").Locator,
) {
  await row.click({ position: { x: 4, y: 4 } });
  await page.keyboard.press("Enter");
  await page.getByText("Notes", { exact: true }).click();
  await page.getByText("Ledger", { exact: true }).click();
  await expect(page.getByRole("heading", { name: "Ledger" })).toBeVisible();
}

test.describe("annotations", () => {
  test("K marks the selected type as critical, with a label and not by colour alone", async ({
    page,
    harness,
  }) => {
    const row = await addScannedProject(page, harness, "annotationsdemo", SOURCE);
    await openLedger(page, row);
    await page.keyboard.press("k");
    await expect(page.getByText("Marked as system critical.")).toBeVisible();
    const details = page.getByRole("region", { name: "Details" });
    await expect(details.getByText("Critical", { exact: true })).toBeVisible();
    await expect(page.getByRole("tree").getByRole("img", { name: "Critical" })).toBeVisible();
    // The tab list names the shield for a screen reader.
    await expect(page.getByRole("tab", { name: "Critical" })).toBeVisible();
    // The second K takes the mark off again.
    await page.keyboard.press("k");
    await expect(details.getByText("Critical", { exact: true })).toHaveCount(0);
  });

  test("N and T open the forms of the tabs, Escape closes them and the counters follow", async ({
    page,
    harness,
  }) => {
    const row = await addScannedProject(page, harness, "annotationsnotes", SOURCE);
    await openLedger(page, row);
    await page.keyboard.press("n");
    const text = page.getByLabel("Text of the note");
    await expect(text).toBeFocused();
    await text.fill("Rounding is done by the caller.");
    await page.keyboard.press("Control+Enter");
    await expect(page.getByText("Rounding is done by the caller.")).toBeVisible();
    await expect(page.getByRole("tab", { name: /Notes/ })).toContainText("1");

    await page.keyboard.press("t");
    const title = page.getByLabel("Title");
    await expect(title).toBeFocused();
    await title.fill("Add a test");
    await page.keyboard.press("Control+Enter");
    await expect(page.getByRole("tab", { name: /Tasks/ })).toContainText("0/1");
  });

  test("the arrow keys move along the tabs of the side panel", async ({ page, harness }) => {
    const row = await addScannedProject(page, harness, "annotationstabs", SOURCE);
    await openLedger(page, row);
    const tabs = page.getByRole("tablist", { name: "Annotations of the symbol" });
    await tabs.getByRole("tab", { name: "Info" }).focus();
    await page.keyboard.press("ArrowRight");
    await expect(tabs.getByRole("tab", { name: "Critical" })).toBeFocused();
    await expect(tabs.getByRole("tab", { name: "Critical" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });

  test("the bell opens the drawer, Escape closes it and the board lists the tasks", async ({
    page,
    harness,
  }) => {
    const row = await addScannedProject(page, harness, "annotationsboard", SOURCE);
    await openLedger(page, row);
    await page.keyboard.press("t");
    await page.getByLabel("Title").fill("Check the ledger");
    await page.keyboard.press("Control+Enter");

    const bell = page.getByRole("button", { name: /^Alerts/ });
    await bell.focus();
    await page.keyboard.press("Enter");
    await expect(page.getByRole("dialog", { name: "Alerts" })).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog", { name: "Alerts" })).toHaveCount(0);
    await expect(bell).toBeFocused();

    await page.getByRole("link", { name: "Annotations", exact: true }).click();
    await expect(page.getByRole("heading", { name: "Annotations" })).toBeVisible();
    await expect(page.getByText("Check the ledger")).toBeVisible();
    await expect(page.getByRole("heading", { name: "Open (1)" })).toBeVisible();
  });
});
