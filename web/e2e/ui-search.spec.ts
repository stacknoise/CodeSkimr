// Runs after ui-explorer.spec.ts and before ui.spec.ts: the worker shares one server.
import { addScannedProject, expect, test, watchPolicy } from "./fixtures";

const SOURCE =
  "namespace Shop;\n\npublic class Basket\n{\n    public int Total(int a, int b) { return a + b; }\n}\n";

test.describe("the quick search", () => {
  test("Ctrl+K finds a type and Enter opens it in the explorer", async ({ page, harness }) => {
    const violations = await watchPolicy(page);
    await addScannedProject(page, harness, "shop", SOURCE);
    await page.getByRole("button", { name: "Open shop" }).click();
    await expect(page.getByRole("tree")).toBeVisible();

    await page.keyboard.press("Control+k");
    await page.getByRole("combobox").fill("bask");
    await expect(page.getByRole("option", { name: /Shop.Basket/ })).toBeVisible();
    await page.keyboard.press("Enter");

    await expect(page.getByRole("dialog")).toBeHidden();
    await expect(page.getByRole("heading", { name: "Basket" })).toBeVisible();
    await expect(page.getByRole("tree").getByText("Total")).toBeHidden();
    expect(await violations()).toEqual([]);
  });

  test("f: lists the files and Enter opens the first type, @ searches every project", async ({
    page,
    harness,
  }) => {
    await addScannedProject(page, harness, "shopfiles", SOURCE);
    await page.getByRole("button", { name: "Open shopfiles" }).click();
    await expect(page.getByRole("tree")).toBeVisible();

    await page.keyboard.press("Control+k");
    const input = page.getByRole("combobox");
    await input.fill("f:shopf");
    await expect(page.getByRole("option")).toHaveCount(1);
    await expect(page.getByRole("option")).toContainText("lines");
    await page.keyboard.press("Enter");
    await expect(page.getByRole("heading", { name: "Basket" })).toBeVisible();

    await page.keyboard.press("Control+k");
    await page.getByRole("combobox").fill("@bask");
    await expect(page.getByRole("option", { name: /Shop.Basket/ }).first()).toBeVisible();
    await page.keyboard.press("Escape");
  });
});
