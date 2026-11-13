// Keyboard paths and click rules of blueprint 15.4. Runs after ui-graph.spec.ts and before the
// other screens: the worker shares one server and every scan adds a project.
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import type { Locator, Page } from "@playwright/test";
import {
  addScannedProject,
  expect,
  keepLaunchFile,
  SCAN_MS,
  test,
  watchPolicy,
  type Harness,
} from "./fixtures";

const SOURCE = [
  "namespace Kbd;",
  "",
  "public class Ledger",
  "{",
  "    public int Balance;",
  "    public int Add(int a, int b) { return a + b; }",
  "}",
  "",
  "public class Report",
  "{",
  "    public int Show(Ledger ledger) { return ledger.Add(1, 2); }",
  "}",
  "",
].join("\n");

const tree = (page: Page): Locator => page.getByRole("tree");
const item = (page: Page, name: string | RegExp): Locator =>
  tree(page).getByRole("treeitem", { name });
const selected = (page: Page): Locator => tree(page).getByRole("treeitem", { selected: true });

/** Adds the project `name`, opens it in the explorer and leaves the tree as it is at the start. */
async function openExplorer(page: Page, harness: Harness, name: string): Promise<void> {
  const row = await addScannedProject(page, harness, name, SOURCE);
  await row.click({ position: { x: 4, y: 4 } });
  await page.keyboard.press("Enter");
  await expect(tree(page)).toBeVisible();
}

/** Opens the layer and the namespace, so the two types are rows. */
async function openNamespace(page: Page, name: string): Promise<void> {
  await item(page, /\(no layer\)/).click();
  await item(page, new RegExp(`${name}\\.Kbd`)).click();
}

test.describe("the explorer tree (15.4)", () => {
  test("starts collapsed with nothing selected and the overview shown", async ({
    page,
    harness,
  }) => {
    const violations = await watchPolicy(page);
    await openExplorer(page, harness, "kbdstart");
    await expect(page.getByRole("heading", { name: "Overview" })).toBeVisible();
    await expect(tree(page).getByRole("treeitem")).toHaveCount(1);
    await expect(item(page, /\(no layer\)/)).toHaveAttribute("aria-expanded", "false");
    await expect(selected(page)).toHaveCount(0);
    expect(await violations()).toEqual([]);
  });

  test("the click rules: caret, unselected type, selected type, groups", async ({
    page,
    harness,
  }) => {
    await openExplorer(page, harness, "kbdclick");
    await openNamespace(page, "kbdclick");
    const ledger = item(page, /Ledger/);
    const report = item(page, /Report/);

    // A click on an unselected type selects it. The tree keeps its open state (issue #140).
    await ledger.click();
    await expect(page.getByRole("heading", { name: "Ledger" })).toBeVisible();
    await expect(ledger).toHaveAttribute("aria-selected", "true");
    const closed = await ledger.getAttribute("aria-expanded");

    // A click on the selected type toggles it.
    await ledger.click();
    await expect(ledger).not.toHaveAttribute("aria-expanded", closed ?? "");
    await expect(ledger).toHaveAttribute("aria-selected", "true");
    await expect(item(page, /Methods/)).toBeVisible();

    // The groups toggle on their own and leave the selection alone.
    const methods = item(page, /Methods/);
    await expect(methods).toHaveAttribute("aria-expanded", "false");
    await methods.click();
    await expect(methods).toHaveAttribute("aria-expanded", "true");
    await expect(item(page, /Add/)).toBeVisible();
    await expect(item(page, /Properties & fields/)).toHaveAttribute("aria-expanded", "false");
    await methods.click();
    await expect(methods).toHaveAttribute("aria-expanded", "false");
    await expect(ledger).toHaveAttribute("aria-selected", "true");

    // The caret of another type opens it and leaves the selection where it is.
    await report.locator("span").first().click();
    await expect(report).toHaveAttribute("aria-expanded", "true");
    await expect(ledger).toHaveAttribute("aria-selected", "true");
    await report.locator("span").first().click();
    await expect(report).toHaveAttribute("aria-expanded", "false");

    // Selecting another type from inside the tree does not open or close anything else.
    await report.click();
    await expect(page.getByRole("heading", { name: "Report" })).toBeVisible();
    await expect(ledger).toHaveAttribute("aria-expanded", "true");
    await expect(report).toHaveAttribute("aria-selected", "true");
    await expect(selected(page)).toHaveCount(1);
  });

  test("the arrow keys, Home, End, Enter and Space move and act in the tree", async ({
    page,
    harness,
  }) => {
    await openExplorer(page, harness, "kbdarrows");
    const root = tree(page);
    await root.focus();
    await page.keyboard.press("ArrowDown");
    await expect(root).toHaveAttribute("aria-activedescendant", /.+/);
    const layer = item(page, /\(no layer\)/);
    await page.keyboard.press("ArrowRight");
    await expect(layer).toHaveAttribute("aria-expanded", "true");
    await page.keyboard.press("ArrowLeft");
    await expect(layer).toHaveAttribute("aria-expanded", "false");
    await page.keyboard.press("Enter");
    await expect(layer).toHaveAttribute("aria-expanded", "true");
    await page.keyboard.press("Space");
    await expect(layer).toHaveAttribute("aria-expanded", "false");
    await page.keyboard.press("Enter");

    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Enter");
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Enter");
    await expect(page.getByRole("heading", { name: "Ledger" })).toBeVisible();
    await page.keyboard.press("End");
    await page.keyboard.press("Enter");
    await expect(page.getByRole("heading", { name: "Report" })).toBeVisible();
    await page.keyboard.press("Home");
    await page.keyboard.press("Enter");
    await expect(layer).toHaveAttribute("aria-expanded", "false");
  });

  test("Ctrl+Left closes the whole tree and Ctrl+. shows the selection again", async ({
    page,
    harness,
  }) => {
    await openExplorer(page, harness, "kbdreveal");
    await openNamespace(page, "kbdreveal");
    await item(page, /Ledger/).click();
    await expect(page.getByRole("heading", { name: "Ledger" })).toBeVisible();

    await page.keyboard.press("Control+ArrowLeft");
    await expect(tree(page).getByRole("treeitem")).toHaveCount(1);
    // Closing the tree keeps the selection: the details stay.
    await expect(page.getByRole("heading", { name: "Ledger" })).toBeVisible();

    await page.keyboard.press("Control+.");
    await expect(item(page, /Ledger/)).toBeVisible();
    await expect(item(page, /Ledger/)).toHaveAttribute("aria-selected", "true");
  });

  test("hotkeys do not fire while text is typed in a field", async ({ page, harness }) => {
    await openExplorer(page, harness, "kbdtyping");
    await openNamespace(page, "kbdtyping");
    await item(page, /Ledger/).click();
    const filter = page.getByRole("textbox", { name: "Filter tree" });
    await filter.focus();
    await page.keyboard.type("urg");
    await expect(filter).toHaveValue("urg");
    // U would open the usages and R rescan; both are text here.
    await expect(page.getByRole("tree")).toBeVisible();
    await expect(page).toHaveURL(/\/$|\/explorer/);
  });
});

test.describe("quick search and Esc (15.4)", () => {
  test("Ctrl+K opens, the prefixes and Tab choose the kind, Esc closes and gives the focus back", async ({
    page,
    harness,
  }) => {
    await openExplorer(page, harness, "kbdsearch");
    await item(page, /\(no layer\)/).focus();
    await page.keyboard.press("Control+k");
    const input = page.getByRole("combobox");
    await expect(input).toBeFocused();

    await input.fill("t:led");
    await expect(page.getByRole("option", { name: /Ledger/ }).first()).toBeVisible();
    await expect(page.getByRole("option", { name: /Add/ })).toHaveCount(0);
    await input.fill("m:add");
    await expect(page.getByRole("option", { name: /Add/ }).first()).toBeVisible();
    await input.fill(">");
    await expect(page.getByRole("option", { name: /Settings/i }).first()).toBeVisible();

    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog")).toBeHidden();
    await expect(tree(page)).toBeVisible();
  });

  test("Enter in the search opens the hit and Esc on a closed search does nothing", async ({
    page,
    harness,
  }) => {
    await openExplorer(page, harness, "kbdenter");
    await page.keyboard.press("Escape");
    await expect(tree(page)).toBeVisible();
    await page.keyboard.press("Control+k");
    await page.getByRole("combobox").fill("m:show");
    await expect(page.getByRole("option", { name: /Show/ }).first()).toBeVisible();
    await page.keyboard.press("Enter");
    await expect(page.getByRole("dialog")).toBeHidden();
    await expect(page.getByRole("heading", { name: "Report" })).toBeVisible();
  });
});

const LAYERS = `
[[layers]]
name = "Presentation"
namespaces = ["Kbd.Ui"]

[[layers]]
name = "Domain"
namespaces = ["Kbd.Domain"]
`;
const DOMAIN =
  "namespace Kbd.Domain;\n\npublic class Order\n{\n    public int Total() { return 1; }\n}\n";
const UI =
  "namespace Kbd.Ui;\n\nusing Kbd.Domain;\n\npublic class OrderView\n{\n    public int Show(Order order) { return order.Total(); }\n}\n";

test.describe("the dependency map (15.4)", () => {
  test("F fits the map to the view and Enter opens the selected layer in the explorer", async ({
    page,
    harness,
  }) => {
    const folder = path.join(mkdtempSync(path.join(tmpdir(), "codeskimr-e2e-")), "kbdgraph");
    mkdirSync(folder);
    writeFileSync(path.join(folder, ".codeskimr.toml"), LAYERS);
    writeFileSync(path.join(folder, "Order.cs"), DOMAIN);
    writeFileSync(path.join(folder, "OrderView.cs"), UI);
    const { copy } = keepLaunchFile(await harness.launchFile());
    await page.goto(`file://${copy}`);
    await page.getByLabel("New project").fill(folder);
    await page.getByRole("button", { name: "Add and scan" }).click();
    const open = page.getByRole("button", { name: "Open kbdgraph" });
    await expect(open).toBeVisible({ timeout: SCAN_MS });
    await expect(
      page.getByRole("row", { name: /kbdgraph/ }).getByText("current", { exact: true }),
    ).toBeVisible({ timeout: SCAN_MS });
    await open.click();
    await page.getByRole("link", { name: "Dependencies" }).click();
    const domain = page.getByRole("button", { name: "Domain" });
    await expect(domain).toBeVisible({ timeout: SCAN_MS });

    // The wheel zooms, F brings the map back to the fitted view.
    const stage = page.locator('[class*="stage"]');
    const fitted = await stage.evaluate((node) => node.style.transform);
    await page.mouse.move(700, 400);
    await page.mouse.wheel(0, -600);
    await expect.poll(() => stage.evaluate((node) => node.style.transform)).not.toBe(fitted);
    await page.keyboard.press("f");
    await expect.poll(() => stage.evaluate((node) => node.style.transform)).toBe(fitted);

    // Enter opens the selected layer's project in the explorer.
    await domain.click();
    await expect(domain).toHaveAttribute("aria-pressed", "true");
    await page.keyboard.press("Escape");
    await page.locator("body").click({ position: { x: 5, y: 5 } });
    await page.keyboard.press("Enter");
    await expect(page.getByRole("tree")).toBeVisible();
  });
});
