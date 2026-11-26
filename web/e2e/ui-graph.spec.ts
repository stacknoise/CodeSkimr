// Runs after ui-explorer.spec.ts: the worker shares one server and every scan adds a project.
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import {
  expect,
  keepLaunchFile,
  listedProject,
  submitFolder,
  SCAN_MS,
  test,
  watchPolicy,
} from "./fixtures";

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

const RING_DOMAIN =
  "namespace Shop.Domain;\n\nusing Shop.Ui;\n\npublic class Order\n{\n    public int Total() { OrderView view = new OrderView(); return view.Show(); }\n}\n";
const RING_UI =
  "namespace Shop.Ui;\n\nusing Shop.Domain;\n\npublic class OrderView\n{\n    public int Show() { Order order = new Order(); return order.Total(); }\n}\n";

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
    await submitFolder(page, folder);
    const open = await listedProject(page, "layered");
    await open.dblclick();

    await page.getByRole("link", { name: "Dependencies" }).click();
    await expect(page.getByRole("button", { name: "Domain" })).toBeVisible({ timeout: SCAN_MS });
    await expect(page.getByRole("button", { name: "Presentation" })).toBeVisible();
    await page.getByRole("button", { name: "Domain" }).click();
    await expect(page.getByRole("heading", { name: "Domain" })).toBeVisible();
    await expect(page.getByText("2 nodes", { exact: false })).toBeVisible();
    expect(await violations()).toEqual([]);
  });

  test("shows the modules with a cycle and the edge that ends it", async ({ page, harness }) => {
    const violations = await watchPolicy(page);
    const folder = path.join(mkdtempSync(path.join(tmpdir(), "codeskimr-e2e-")), "ring");
    mkdirSync(folder);
    writeFileSync(path.join(folder, ".codeskimr.toml"), LAYERS);
    writeFileSync(path.join(folder, "Order.cs"), RING_DOMAIN);
    writeFileSync(path.join(folder, "OrderView.cs"), RING_UI);
    const { copy } = keepLaunchFile(await harness.launchFile());
    await page.goto(`file://${copy}`);
    await submitFolder(page, folder);
    await (await listedProject(page, "ring")).dblclick();

    await page.getByRole("link", { name: "Dependencies" }).click();
    await page.getByRole("button", { name: "Modules", exact: true }).click();
    const panel = page.getByRole("complementary", { name: "Modules" });
    await expect(panel.getByRole("heading", { name: "Cycles" })).toBeVisible({ timeout: SCAN_MS });
    await expect(panel.getByText(/Ending the cycle takes away the weakest edge/)).toBeVisible();
    await expect(panel.getByText("in a cycle").first()).toBeVisible();
    for (const theme of ["hell", "petrol", "dunkel"]) {
      await page.evaluate(
        (name) => document.documentElement.setAttribute("data-theme", name),
        theme,
      );
      await expect(panel).toBeVisible();
      const dir = process.env["MODULES_SHOT"];
      if (dir !== undefined) {
        await page.screenshot({ path: path.join(dir, `modules-${theme}.png`) });
      }
    }
    expect(await violations()).toEqual([]);
  });
});
