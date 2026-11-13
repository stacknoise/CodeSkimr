// Runs after ui-search.spec.ts and before ui.spec.ts: the worker shares one server.
import { expect, keepLaunchFile, test, watchPolicy } from "./fixtures";

test.describe("the status bar", () => {
  test("shows the measured memory and switches the language", async ({ page, harness }) => {
    const violations = await watchPolicy(page);
    const { copy } = keepLaunchFile(await harness.launchFile());
    await page.goto(`file://${copy}`);

    const bar = page.getByRole("contentinfo");
    await expect(bar.getByText(/^RAM \d+ MB$/)).toBeVisible();
    await expect(bar.getByText("Connected")).toBeVisible();

    await bar.getByRole("button", { name: "DE" }).click();
    await expect(bar.getByText(/^RAM \d+ MB$/)).toBeVisible();
    await expect(bar.getByRole("group", { name: "Sprache" })).toBeVisible();
    await bar.getByRole("button", { name: "EN" }).click();
    await expect(bar.getByRole("group", { name: "Language" })).toBeVisible();
    expect(await violations()).toEqual([]);
  });
});
