// Runs after ui-explorer.spec.ts: the worker shares one server and every scan adds a project.
import { addScannedProject, expect, test, watchPolicy } from "./fixtures";

const SOURCE =
  "namespace Scan;\n\npublic class Meter\n{\n    public int Count() { return 1; }\n}\n";

test.describe("the scan and performance screen", () => {
  test("shows the scan that just ran, the budgets from the server and the languages", async ({
    page,
    harness,
  }) => {
    const violations = await watchPolicy(page);
    await addScannedProject(page, harness, "metered", SOURCE);

    await page.getByRole("link", { name: "Scans and performance" }).click();
    // The stream was open while the scan ran, so the finished run is listed.
    await expect(page.getByRole("heading", { name: "Scan finished" })).toBeVisible();
    await expect(page.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "100");
    await expect(page.getByText("Pipeline")).toBeVisible();
    await expect(page.getByText("Parse", { exact: true })).toBeVisible();

    const budgets = page.getByText("Performance targets").locator("..");
    await expect(budgets.getByText("Target < 300 ms")).toHaveCount(2);
    await expect(budgets.getByText("Target < 1,000 ms")).toBeVisible();

    await expect(page.getByRole("cell", { name: "C#" })).toBeVisible();
    await expect(page.getByText(/^metered · full/).first()).toBeVisible();
    expect(await violations()).toEqual([]);
  });
});
