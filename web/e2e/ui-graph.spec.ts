// Runs after ui-explorer.spec.ts: the worker shares one server and every scan adds a project.
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { expect, keepLaunchFile, SCAN_MS, test, watchPolicy } from "./fixtures";

const LAYERS = `
[[layers]]
name = "Presentation"
namespaces = ["Shop.Ui"]

[[layers]]
name = "Domain"
namespaces = ["Shop.Domain"]
`;
const DOMAIN =
  "namespace Shop.Domain;\n\npublic class Order\n{\n    public int Total() { return 1; }\n}\n";
const UI =
  "namespace Shop.Ui;\n\nusing Shop.Domain;\n\npublic class OrderView\n{\n    public int Show(Order order) { return order.Total(); }\n}\n";

test.describe("the dependency map", () => {
  test("draws the layers of a scanned project and selects one", async ({ page, harness }) => {
    const violations = await watchPolicy(page);
    const folder = path.join(mkdtempSync(path.join(tmpdir(), "codeskimr-e2e-")), "layered");
    mkdirSync(folder);
    writeFileSync(path.join(folder, ".codeskimr.toml"), LAYERS);
    writeFileSync(path.join(folder, "Order.cs"), DOMAIN);
    writeFileSync(path.join(folder, "OrderView.cs"), UI);
    const { copy } = keepLaunchFile(await harness.launchFile());
    await page.goto(`file://${copy}`);
    await page.getByLabel("New project").fill(folder);
    await page.getByRole("button", { name: "Add and scan" }).click();
    await expect(page.getByRole("button", { name: "Open layered" })).toBeVisible({
      timeout: SCAN_MS,
    });
    const row = page.getByRole("row", { name: /layered/ });
    await expect(row.getByText("current", { exact: true })).toBeVisible({ timeout: SCAN_MS });
    await page.getByRole("button", { name: "Open layered" }).click();

    await page.getByRole("link", { name: "Dependencies" }).click();
    await expect(page.getByRole("button", { name: "Domain" })).toBeVisible({ timeout: SCAN_MS });
    await expect(page.getByRole("button", { name: "Presentation" })).toBeVisible();
    await page.getByRole("button", { name: "Domain" }).click();
    await expect(page.getByRole("heading", { name: "Domain" })).toBeVisible();
    await expect(page.getByText("2 nodes", { exact: false })).toBeVisible();
    expect(await violations()).toEqual([]);
  });
});
