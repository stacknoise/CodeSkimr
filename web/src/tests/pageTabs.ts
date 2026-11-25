import { fireEvent, screen } from "@solidjs/testing-library";

/** Chooses a project in the list of the projects screen. */
export async function chooseProject(name = "shop"): Promise<void> {
  fireEvent.click(
    await screen.findByRole("button", { name: new RegExp(`^(Open ${name}|${name} öffnen)$`) }),
  );
}

/** Shows a tab of the project page. */
export async function chooseTab(name: RegExp): Promise<void> {
  fireEvent.click(await screen.findByRole("tab", { name }));
}

export const TAB_LEVEL2 = /Exact check|Genaue Prüfung/;
export const TAB_SCAN = /^(Scan)$/;
export const TAB_TOOLS = /^(Tools|Werkzeuge)$/;

/** The label of the source field of the window "New project". */
export const SOURCE_LABEL = /^(Folder|Repository address)$/;
