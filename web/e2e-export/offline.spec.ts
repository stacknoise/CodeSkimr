// Opens an export page that `codeskimr export` wrote on this machine, the way a user does: from
// disk, with no server (issue #670). CODESKIMR_EXPORT_FILE names the page, CODESKIMR_EXPORT_TYPE
// a type of the exported project. The build of the page swaps modules by their paths, and paths
// differ between the systems: on Windows the page once called the server (#669) and once held its
// data twice, so the tree stayed empty though the head line was right (#676).
import { pathToFileURL } from "node:url";
import { expect, test } from "@playwright/test";

const FILE = process.env["CODESKIMR_EXPORT_FILE"] ?? "";
const TYPE = process.env["CODESKIMR_EXPORT_TYPE"] ?? "";
const LOCAL = "file:";

test("the exported page shows the tree and finds a type without a server", async ({ page }) => {
  expect(FILE, "CODESKIMR_EXPORT_FILE names the exported page").not.toBe("");
  expect(TYPE, "CODESKIMR_EXPORT_TYPE names a type of the project").not.toBe("");
  const requests: string[] = [];
  const errors: string[] = [];
  page.on("request", (request) => requests.push(request.url()));
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") {
      errors.push(message.text());
    }
  });

  await page.goto(pathToFileURL(FILE).href);
  const tree = page.getByRole("tree");
  await expect(tree.getByRole("treeitem").first()).toBeVisible();
  await page.keyboard.press("Control+k");
  await page.getByRole("combobox").fill(TYPE);
  await page.getByRole("option", { name: TYPE }).first().click();
  const picked = tree.getByRole("treeitem", { name: TYPE, selected: true });
  await expect(picked.first()).toBeVisible();

  expect(errors).toEqual([]);
  expect(requests.filter((url) => !url.startsWith(LOCAL))).toEqual([]);
});
