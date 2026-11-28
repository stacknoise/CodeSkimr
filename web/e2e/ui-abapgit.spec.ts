// An abapGit repository (ADR 0113, issues #843 and #844): the window "New project" says what the
// folder is before it is added, the tab "Scope" shows the card of the repository, and the explorer
// shows the packages as the folders of the files. The server is the real one; the folder is
// written for this test. `UPDATE_SCREENSHOTS=1` writes the screens to docs/screenshots/abapgit/.
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import {
  expect,
  keepLaunchFile,
  listedProject,
  openInExplorer,
  openNewForm,
  shot,
  showTab,
  SOURCE_LABEL,
  test,
} from "./fixtures";

const SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots/abapgit");
const VIEWPORT = { width: 1440, height: 900 };
const THEMES = ["hell", "petrol", "dunkel"];
const CONFIG =
  '<?xml version="1.0" encoding="utf-8"?>\n<abapGit version="v1.0.0">\n <DATA>\n  <STARTING_FOLDER>/src/</STARTING_FOLDER>\n  <FOLDER_LOGIC>PREFIX</FOLDER_LOGIC>\n </DATA>\n</abapGit>\n';
const ORDER =
  "CLASS zcl_order DEFINITION PUBLIC.\n  PUBLIC SECTION.\n    METHODS total RETURNING VALUE(result) TYPE i.\nENDCLASS.\n\nCLASS zcl_order IMPLEMENTATION.\n  METHOD total.\n    result = 1.\n  ENDMETHOD.\nENDCLASS.\n";
const VIEW =
  "CLASS zcl_view DEFINITION PUBLIC.\n  PUBLIC SECTION.\n    METHODS show.\nENDCLASS.\n\nCLASS zcl_view IMPLEMENTATION.\n  METHOD show.\n  ENDMETHOD.\nENDCLASS.\n";
const PACKAGE =
  '<?xml version="1.0" encoding="utf-8"?>\n<abapGit version="v1.0.0"><asx:abap xmlns:asx="http://www.sap.com/abapxml"><asx:values><DEVC><CTEXT>Order core</CTEXT></DEVC></asx:values></asx:abap></abapGit>\n';
const INTERFACE = "INTERFACE zif_order PUBLIC.\nENDINTERFACE.\n";

/** A folder in the layout of abapGit: two packages below `src/`. */
function abapGitFolder(name: string): string {
  const folder = path.join(mkdtempSync(path.join(tmpdir(), "codeskimr-e2e-")), name);
  for (const [file, text] of Object.entries({
    ".abapgit.xml": CONFIG,
    "src/zcore/package.devc.xml": PACKAGE,
    "src/zcore/zcl_order.clas.abap": ORDER,
    "src/zcore/zif_order.intf.abap": INTERFACE,
    "src/zui/zcl_view.clas.abap": VIEW,
  })) {
    mkdirSync(path.dirname(path.join(folder, file)), { recursive: true });
    writeFileSync(path.join(folder, file), text);
  }
  return folder;
}

for (const theme of THEMES) {
  test.describe(`an abapGit repository (1440 x 900, ${theme})`, () => {
    test.use({ viewport: VIEWPORT });

    test(`is named in the window, on the card and in the package tree, ${theme}`, async ({
      page,
      harness,
    }, info) => {
      await page.addInitScript((value) => localStorage.setItem("codeskimr.theme", value), theme);
      const name = `abap${theme}`;
      const folder = abapGitFolder(name);
      const { copy } = keepLaunchFile(await harness.launchFile());
      await page.goto(`file://${copy}`);

      // The window says what the folder is before the project exists.
      await openNewForm(page);
      await page.getByLabel(SOURCE_LABEL).fill(folder);
      await expect(page.getByText("This is an abapGit repository")).toBeVisible();
      await expect(page.getByText(/3 objects in 2 packages, below src/)).toBeVisible();
      await shot(page, SHOTS, `abapgit-new-project-${theme}`, info);
      await page.getByRole("button", { name: "Add and scan" }).click();

      // The card on the tab "Scope".
      const row = await listedProject(page, name);
      await row.click({ position: { x: 4, y: 4 } });
      await showTab(page, "Scope");
      const card = page.getByRole("region", { name: "abapGit repository" });
      await expect(card).toBeVisible();
      await expect(card).toContainText("CLAS");
      await expect(card).toContainText("INTF");
      await shot(page, SHOTS, `abapgit-scope-${theme}`, info);

      // The explorer: the packages are the folders of the files.
      await openInExplorer(page);
      await expect(page.getByRole("tree")).toBeVisible();
      await expect(page.getByText("zcore", { exact: true })).toBeVisible();
      await expect(page.getByText("zui", { exact: true })).toBeVisible();
      await expect(page.getByText("Order core", { exact: true })).toBeVisible();
      await page.getByText("zcore", { exact: true }).click();
      await shot(page, SHOTS, `abapgit-explorer-${theme}`, info);
    });
  });

  test.describe(`ABAP files without an abapGit layout (1440 x 900, ${theme})`, () => {
    test.use({ viewport: VIEWPORT });

    test(`the tab "Scope" says that no layout was recognised, ${theme}`, async ({
      page,
      harness,
    }, info) => {
      await page.addInitScript((value) => localStorage.setItem("codeskimr.theme", value), theme);
      const name = `abapmiss${theme}`;
      const folder = path.join(mkdtempSync(path.join(tmpdir(), "codeskimr-e2e-")), name);
      mkdirSync(path.join(folder, "lib"), { recursive: true });
      writeFileSync(path.join(folder, "lib/zcl_order.clas.abap"), ORDER);
      const { copy } = keepLaunchFile(await harness.launchFile());
      await page.goto(`file://${copy}`);
      await openNewForm(page);
      await page.getByLabel(SOURCE_LABEL).fill(folder);
      // Without `.abapgit.xml` the window has no hint.
      await page.getByRole("button", { name: "Add and scan" }).click();
      const row = await listedProject(page, name);
      await row.click({ position: { x: 4, y: 4 } });
      await showTab(page, "Scope");
      await expect(
        page.getByRole("heading", { name: "ABAP files found, no abapGit layout recognised" }),
      ).toBeVisible();
      await expect(page.getByRole("heading", { name: "abapGit repository" })).toHaveCount(0);
      await shot(page, SHOTS, `abapgit-missed-${theme}`, info);
    });
  });
}
