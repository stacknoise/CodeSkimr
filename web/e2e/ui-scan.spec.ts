// Runs after ui-explorer.spec.ts: the worker shares one server and every scan adds a project.
import { expect, openScannedProject, showTab, test, watchPolicy } from "./fixtures";

const SOURCE =
  "namespace Scan;\n\npublic class Meter\n{\n    public int Count() { return 1; }\n}\n";

test.describe("the tab Scan and the performance targets", () => {
  test("shows the scan that just ran, the budgets from the server and the languages", async ({
    page,
    harness,
  }) => {
    const violations = await watchPolicy(page);
    await openScannedProject(page, harness, "metered", SOURCE);

    await page.getByRole("link", { name: "Projects", exact: true }).click();
    await page.getByRole("button", { name: "Open metered" }).click();
    await showTab(page, "Scan");
    // The stream was open while the scan ran, so the finished run is listed.
    await expect(page.getByRole("heading", { name: "Scan finished" })).toBeVisible();
    await expect(page.getByRole("progressbar", { name: "Progress of the scan" })).toHaveAttribute(
      "aria-valuenow",
      "100",
    );
    await expect(page.getByText("Pipeline")).toBeVisible();
    await expect(page.getByText("Parse", { exact: true })).toBeVisible();
    await expect(page.getByRole("cell", { name: "C#" })).toBeVisible();

    // The performance targets are an entry of the status bar and belong to the whole program.
    await page.getByRole("button", { name: /^Performance/ }).click();
    const budgets = page.getByRole("dialog", { name: "Performance targets" });
    await expect(budgets.getByText("Target < 300 ms")).toHaveCount(2);
    await expect(budgets.getByText("Target < 1,000 ms")).toBeVisible();
    expect(await violations()).toEqual([]);
  });
});
