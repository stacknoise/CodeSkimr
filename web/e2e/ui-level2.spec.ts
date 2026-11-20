// The level 2 switch of the projects screen (ADR 0071, blueprint 15.4): the keyboard path and the
// look in the three themes. Most tests answer `level2.status`, `level2.revoke` and `confirm-window`
// from a fake that follows the ADR, because the consent is typed in a console window; everything
// else is the real server, and the two tests of "against the real server" use no fake at all.
// `UPDATE_SCREENSHOTS=1` writes the screens to docs/screenshots/level2-switch/.
import path from "node:path";
import type { Page, Route } from "@playwright/test";
import { expect, selectScannedProject, shot, test } from "./fixtures";

const SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots/level2-switch");
const VIEWPORT = { width: 1440, height: 900 };
const THEMES = ["hell", "petrol", "dunkel"];
const SOURCE =
  "namespace Lv2;\n\npublic class Ledger\n{\n    public int Add(int a, int b) { return a + b; }\n}\n";
/** The poll of the page is two seconds; the fake needs a little more to be seen. */
const POLL_WAIT_MS = 10_000;

/** What the fake server of the switch says and has seen. */
interface Fake {
  state: "on" | "off" | "not_enabled" | "locked";
  confirmed: boolean;
  launch: "window" | "copy";
  windows: number;
  revokes: number;
  /** The command the fake gave for the project that asked last. */
  command: string;
}

/** The terminal command of the page for a project; the fake answers it with the real id. */
const confirmCommandOf = (id: number) => `codeskimr level2 confirm ${id}`;

/** Answers the three calls of the switch from `fake`; nothing else is touched. */
async function serveSwitch(page: Page, fake: Fake): Promise<void> {
  const json = (route: Route, body: unknown) => route.fulfill({ json: body });
  await page.route("**/api/v1/commands/level2.status", (route) =>
    json(route, { state: fake.state, confirmed: fake.confirmed, launch: fake.launch }),
  );
  await page.route("**/api/v1/commands/level2.revoke", (route) => {
    fake.revokes += 1;
    fake.state = "off";
    fake.confirmed = false;
    return json(route, {});
  });
  await page.route("**/api/v1/level2/confirm-window", (route) => {
    fake.windows += 1;
    const { project_id: id } = route.request().postDataJSON() as { project_id: number };
    fake.command = confirmCommandOf(id);
    return json(
      route,
      fake.launch === "copy" ? { launch: "copy", command: fake.command } : { launch: "window" },
    );
  });
}

function newFake(state: Fake["state"] = "off"): Fake {
  return {
    state,
    confirmed: state === "on",
    launch: "window",
    windows: 0,
    revokes: 0,
    command: "",
  };
}

async function openProject(
  page: Page,
  harness: Parameters<typeof selectScannedProject>[1],
  name: string,
) {
  await selectScannedProject(page, harness, name, SOURCE);
  return page.getByRole("switch", { name: `Level 2 for ${name}` });
}

test.describe("the level 2 switch (keyboard, ADR 0071)", () => {
  test("Escape ends the waiting for the console window, and so does the button", async ({
    page,
    harness,
  }) => {
    const fake = newFake();
    await serveSwitch(page, fake);
    const toggle = await openProject(page, harness, "lvwait");
    await toggle.focus();
    await page.keyboard.press("Space");
    await expect(page.getByText("Finish the confirmation in the console window")).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(page.getByText(/Waiting ended\. Nothing was confirmed/)).toBeVisible();
    await expect(toggle).toBeFocused();
    expect(fake.revokes).toBe(0);

    await page.keyboard.press("Space");
    const stop = page.getByRole("button", { name: "Cancel waiting" });
    await stop.focus();
    await page.keyboard.press("Enter");
    await expect(page.getByText(/Waiting ended\. Nothing was confirmed/)).toBeVisible();
    await expect(toggle).toBeFocused();
    expect(fake.windows).toBe(2);
  });

  test("Space switches on and waits, Space then asks before switching off, Escape and Enter answer", async ({
    page,
    harness,
  }) => {
    const fake = newFake();
    await serveSwitch(page, fake);
    const toggle = await openProject(page, harness, "lvkeys");
    await toggle.focus();
    await expect(toggle).toHaveAttribute("aria-checked", "false");

    await page.keyboard.press("Space");
    await expect(page.getByText("Finish the confirmation in the console window")).toBeVisible();
    expect(fake.windows).toBe(1);

    // The owner types the name in the console window; the page only sees the state change.
    fake.state = "on";
    fake.confirmed = true;
    await expect(toggle).toHaveAttribute("aria-checked", "true", { timeout: POLL_WAIT_MS });

    await toggle.focus();
    await page.keyboard.press("Enter");
    const yes = page.getByRole("button", { name: "Yes, switch off" });
    await expect(yes).toBeFocused();
    await page.keyboard.press("Escape");
    await expect(toggle).toBeFocused();
    expect(fake.revokes).toBe(0);

    await page.keyboard.press("Space");
    await expect(yes).toBeFocused();
    await page.keyboard.press("Enter");
    await expect(page.getByText(/Checked results stay until the next scan/).first()).toBeVisible();
    await expect(toggle).toHaveAttribute("aria-checked", "false");
    expect(fake.revokes).toBe(1);
  });

  test("without a terminal the command is shown and can be copied by keyboard", async ({
    page,
    harness,
    context,
  }) => {
    await context.grantPermissions(["clipboard-read", "clipboard-write"]);
    const fake = newFake();
    fake.launch = "copy";
    await serveSwitch(page, fake);
    const toggle = await openProject(page, harness, "lvcopy");
    await toggle.focus();
    await page.keyboard.press("Space");
    await expect.poll(() => fake.command).toMatch(/^codeskimr level2 confirm \d+$/);
    const command = fake.command;
    await expect(page.getByText(command, { exact: true })).toBeVisible();
    await page.getByRole("button", { name: "Copy", exact: true }).focus();
    await page.keyboard.press("Enter");
    await expect(page.getByRole("button", { name: "Copied" })).toBeVisible();
    expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(command);
  });

  test("not enabled offers no click and shows the two config lines", async ({ page, harness }) => {
    const fake = newFake("not_enabled");
    await serveSwitch(page, fake);
    const toggle = await openProject(page, harness, "lvhint");
    await expect(toggle).toHaveAttribute("aria-disabled", "true");
    await expect(page.getByText(/enabled = true/)).toBeVisible();
    await toggle.click({ force: true });
    expect(fake.windows).toBe(0);
  });
});

test.describe("the level 2 switch against the real server (ADR 0071)", () => {
  test("a fresh installation says not enabled, shows the two config lines and offers no click", async ({
    page,
    harness,
  }) => {
    const toggle = await openProject(page, harness, "lvreal");
    await expect(toggle).toHaveAttribute("aria-disabled", "true");
    await expect(page.getByText(/enabled = true/)).toBeVisible();
    await expect(page.getByText("File: in your config.toml in the data folder")).toBeVisible();
    await toggle.click({ force: true });
    await expect(page.getByText("Finish the confirmation in the console window")).toHaveCount(0);
  });

  test("the server tells the poll values of the page", async ({ page, harness }) => {
    await openProject(page, harness, "lvconfig");
    const config = await page.evaluate(async () => {
      const response = await fetch("/api/v1/ui-config");
      return response.json();
    });
    expect(config).toMatchObject({ level2_poll_seconds: 2, level2_poll_max_seconds: 300 });
  });
});

for (const theme of THEMES) {
  test.describe(`the level 2 switch (1440 x 900, ${theme})`, () => {
    test.use({ viewport: VIEWPORT });

    test(`waiting for the console window, ${theme}`, async ({ page, harness }, info) => {
      await page.addInitScript((name) => localStorage.setItem("codeskimr.theme", name), theme);
      await serveSwitch(page, newFake());
      const toggle = await openProject(page, harness, `lvshot${theme}`);
      await toggle.click();
      await expect(page.getByText("Finish the confirmation in the console window")).toBeVisible();
      await page
        .getByRole("region", { name: /Level 2 for/ })
        .evaluate((el) => el.scrollIntoView({ block: "center" }));
      await shot(page, SHOTS, `level2-switch-${theme}`, info);
    });

    test(`not enabled, ${theme}`, async ({ page, harness }, info) => {
      await page.addInitScript((name) => localStorage.setItem("codeskimr.theme", name), theme);
      await serveSwitch(page, newFake("not_enabled"));
      const toggle = await openProject(page, harness, `lvhint${theme}`);
      await expect(toggle).toHaveAttribute("aria-disabled", "true");
      await expect(page.getByText("File: in your config.toml in the data folder")).toBeVisible();
      await page
        .getByRole("region", { name: /Level 2 for/ })
        .evaluate((el) => el.scrollIntoView({ block: "center" }));
      await shot(page, SHOTS, `level2-switch-not-enabled-${theme}`, info);
    });
  });
}
