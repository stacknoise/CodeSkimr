import { expect, keepLaunchFile, test, watchPolicy } from "./fixtures";

test.describe("the launch code and the token", () => {
  test("the launch file opens the UI, which exchanges the code once and keeps the token in the tab", async ({
    page,
    harness,
  }) => {
    const violations = await watchPolicy(page);
    const requested: string[] = [];
    page.on("request", (request) => requested.push(request.url()));
    const file = await harness.launchFile();
    const { copy, text } = keepLaunchFile(file);
    const code = /#code=([0-9a-f]{32})/.exec(text)?.[1] ?? "";
    expect(code).toHaveLength(32);

    await page.goto(`file://${copy}`);
    await expect(page.getByText("● Connected")).toBeVisible();

    // The fragment is gone from the address, and so is the history entry that carried it.
    expect(page.url()).toBe(`${harness.url}/`);
    expect(await page.evaluate(() => location.hash)).toBe("");
    const token = await page.evaluate(() => sessionStorage.getItem("codeskimr.token"));
    expect(token).toMatch(/^csk_[0-9a-f]{64}$/);
    // The token is in no cookie, no localStorage, and in no address that was ever requested.
    expect(await page.evaluate(() => document.cookie)).toBe("");
    expect(await page.evaluate(() => JSON.stringify({ ...localStorage }))).not.toContain("csk_");
    for (const url of requested) {
      expect(url).not.toContain(token ?? "-");
      expect(url).not.toContain(code);
    }
    expect(await violations()).toEqual([]);

    // The code works once: the same file in a new tab gets nothing.
    const second = await page.context().newPage();
    await second.goto(`file://${copy}`);
    await expect(second.getByRole("heading", { name: "Open CodeSkimr again" })).toBeVisible();
    await expect(
      second.getByText("The launch code was already used or has expired."),
    ).toBeVisible();
    await expect(second.getByText("codeskimr open", { exact: true })).toBeVisible();
    expect(await second.evaluate(() => sessionStorage.getItem("codeskimr.token"))).toBeNull();
  });

  test("a page without a token says how to get one", async ({ page, harness }) => {
    await page.goto(`${harness.url}/`);
    await expect(page.getByRole("heading", { name: "Open CodeSkimr again" })).toBeVisible();
    await expect(page.getByText("Not connected")).toBeVisible();
  });

  test("a reload keeps the session of the tab", async ({ page, harness }) => {
    const { copy } = keepLaunchFile(await harness.launchFile());
    await page.goto(`file://${copy}`);
    await expect(page.getByText("● Connected")).toBeVisible();
    await page.reload();
    await expect(page.getByText("● Connected")).toBeVisible();
  });
});

test.describe("the content security policy", () => {
  test("every answer has the policy and the page works under it", async ({ page, harness }) => {
    const violations = await watchPolicy(page);
    const answer = await page.request.get(`${harness.url}/`);
    const policy = answer.headers()["content-security-policy"] ?? "";
    expect(policy).toContain("default-src 'self'");
    expect(policy).toContain("frame-ancestors 'none'");
    expect(policy).not.toContain("unsafe-");
    expect(await answer.text()).not.toContain("<style");
    const messages: string[] = [];
    page.on("console", (message) => messages.push(message.text()));
    const { copy } = keepLaunchFile(await harness.launchFile());
    await page.goto(`file://${copy}`);
    await page.getByRole("link", { name: "Settings" }).click();
    await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible();
    await page.getByRole("button", { name: "Select" }).last().click();
    expect(await violations()).toEqual([]);
    expect(messages.filter((text) => /Content Security Policy/i.test(text))).toEqual([]);
  });

  test("inline script, inline handlers and inline styles are blocked", async ({
    page,
    harness,
  }) => {
    const violations = await watchPolicy(page);
    const { copy } = keepLaunchFile(await harness.launchFile());
    await page.goto(`file://${copy}`);
    await expect(page.getByText("● Connected")).toBeVisible();
    const outcome = await page.evaluate(() => {
      const w = window as unknown as Record<string, unknown>;
      const script = document.createElement("script");
      script.textContent = "window.__inline = 1";
      document.body.append(script);
      const image = document.createElement("img");
      image.setAttribute("onerror", "window.__handler = 1");
      image.setAttribute("src", "/missing.png");
      document.body.append(image);
      document.body.setAttribute("style", "background: rgb(255, 0, 0)");
      return {
        inline: w["__inline"],
        handler: w["__handler"],
        background: getComputedStyle(document.body).backgroundColor,
      };
    });
    expect(outcome.inline).toBeUndefined();
    expect(outcome.handler).toBeUndefined();
    expect(outcome.background).not.toBe("rgb(255, 0, 0)");
    const seen = (await violations()).join("\n");
    expect(seen).toContain("script-src");
    expect(seen).toContain("style-src");
  });

  test("the page cannot be put in a frame", async ({ page, harness }) => {
    const answer = await page.request.get(`${harness.url}/`);
    expect(answer.headers()["content-security-policy"]).toContain("frame-ancestors 'none'");
  });
});

test.describe("themes and language", () => {
  test("a theme is applied at once, remembered, and set before the first paint", async ({
    page,
    harness,
  }) => {
    const violations = await watchPolicy(page);
    const { copy } = keepLaunchFile(await harness.launchFile());
    await page.goto(`file://${copy}`);
    await page.getByRole("link", { name: "Settings" }).click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", "petrol");
    await page.getByRole("button", { name: "Select" }).last().click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", "dunkel");
    await expect(page.locator("body")).toHaveCSS("background-color", "rgb(18, 23, 29)");
    // After a reload the script of the page sets the theme before the app runs.
    await page.reload({ waitUntil: "commit" });
    await page.waitForFunction(() => document.documentElement.hasAttribute("data-theme"));
    await expect(page.locator("html")).toHaveAttribute("data-theme", "dunkel");
    expect(await violations()).toEqual([]);
  });

  test("every theme draws its preview with its own colours", async ({ page, harness }) => {
    const { copy } = keepLaunchFile(await harness.launchFile());
    await page.goto(`file://${copy}`);
    await page.getByRole("link", { name: "Settings" }).click();
    const colours = await page
      .locator("[data-theme]")
      .evaluateAll((nodes) =>
        nodes
          .filter((node) => node !== document.documentElement)
          .map(
            (node) =>
              `${node.getAttribute("data-theme")}:${getComputedStyle(node).getPropertyValue("--accent").trim()}`,
          ),
      );
    expect(new Set(colours)).toEqual(new Set(["hell:#2457D6", "petrol:#0C727B", "dunkel:#3BB6C1"]));
  });

  test("the language switches at once and the editor link is shown", async ({ page, harness }) => {
    const { copy } = keepLaunchFile(await harness.launchFile());
    await page.goto(`file://${copy}`);
    await page.getByRole("link", { name: "Settings" }).click();
    await expect(page.getByText("vscode://file/{path}:{line}")).toBeVisible();
    await page.getByRole("combobox").selectOption("de");
    await expect(page.getByRole("heading", { name: "Einstellungen" })).toBeVisible();
    await expect(page.locator("html")).toHaveAttribute("lang", "de");
  });
});
