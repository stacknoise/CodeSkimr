import { fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { navigate } from "../../app/router";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { errorText } from "../../shared/i18n/i18n";
import { openProject } from "../../shared/project/active";
import {
  bodiesOf,
  callsOf,
  DONE_TASK,
  MARK,
  NOTE,
  page,
  serveBodies,
  SUMMARY,
  TASK,
} from "../../tests/annotations";
import { answer, serve, useServerSession } from "../../tests/commands";
import { openOrder, ORDER, OUTLINE } from "../../tests/explorer";
import { setShowAnnotationText } from "../annotations/flags";
import { selectTab } from "../annotations/panelState";
import { forgetAnnotations } from "../annotations/store";
import { forgetExplorer, setMemberSort } from "./explorer";
import { ExplorerScreen } from "./ExplorerScreen";
import { INITIAL_SORT } from "./memberSort";

useServerSession();
const PANEL_TABS = "Annotations of the symbol";
afterEach(() => {
  setMemberSort(INITIAL_SORT);
  forgetExplorer();
  forgetAnnotations();
  setShowAnnotationText(false);
  selectTab("info");
  navigate("/");
});

/** Renders the explorer for a project with annotations on the server. */
function renderAnnotated(extra: Parameters<typeof serve>[0] = {}) {
  openProject(1);
  serve({
    "explorer.outline": () => answer(OUTLINE),
    "explorer.symbol": () => answer(ORDER),
    "annotations.summary": () => answer(page(SUMMARY)),
    "annotations.list": () => answer(page([])),
    ...extra,
  });
  return render(() => <ExplorerScreen />);
}

describe("the marks", () => {
  it("show a critical type in the tree with stripe, shield and counters", async () => {
    renderAnnotated();
    await openOrder();
    const row = (await screen.findAllByRole("treeitem", { name: /Order/ }))[0]!;
    await waitFor(() => expect(within(row).getByRole("img", { name: "Critical" })).toBeTruthy());
    expect(row.className).toMatch(/critical/);
    expect(within(row).getByRole("img", { name: "2 notes" }).textContent).toBe("2");
    expect(within(row).getByRole("img", { name: "1 of 3 tasks done" }).textContent).toBe("1/3");
  });

  it("show the count of critical symbols on a collapsed namespace", async () => {
    renderAnnotated();
    fireEvent.click(await screen.findByText("Domain"));
    expect(await screen.findByRole("img", { name: "2 critical symbols below" })).toBeTruthy();
    fireEvent.click(await screen.findByText("Shop"));
    expect(screen.queryByRole("img", { name: "2 critical symbols below" })).toBeNull();
  });

  it("show a critical member with the label and no shield, and the details with both", async () => {
    renderAnnotated();
    await openOrder();
    const total = (await screen.findByText("decimal Total()")).closest("tr")!;
    await waitFor(() => expect(within(total).getByText("Critical")).toBeTruthy());
    expect(total.className).toMatch(/critical/);
    expect(within(total).queryByRole("img", { name: "Critical" })).toBeNull();
    expect(within(total).getByRole("img", { name: "1 note" })).toBeTruthy();
    const id = screen.getByText("int").closest("tr")!;
    expect(within(id).queryByText("Critical")).toBeNull();
    const heading = screen.getByRole("heading", { name: "Order" });
    expect(within(heading.parentElement!).getByText("Critical")).toBeTruthy();
    expect(heading.parentElement!.querySelector("svg")).toBeTruthy();
  });

  it("read the summary once for the snapshot, not once for each row", async () => {
    renderAnnotated();
    await openOrder();
    await screen.findByText("decimal Total()");
    expect(callsOf("annotations.summary")).toBe(1);
    expect(callsOf("annotations.list")).toBe(0);
  });

  it("leave out the tabs, the column and the switch on a server without annotations", async () => {
    openProject(1);
    serve({ "explorer.outline": () => answer(OUTLINE), "explorer.symbol": () => answer(ORDER) });
    render(() => <ExplorerScreen />);
    await openOrder();
    await screen.findByText("decimal Total()");
    await waitFor(() => expect(screen.queryByRole("tablist", { name: PANEL_TABS })).toBeNull());
    expect(screen.queryByText("Annotations")).toBeNull();
    expect(screen.queryByLabelText("Show annotations")).toBeNull();
    expect(screen.getByRole("region", { name: "Source" })).toBeTruthy();
  });
});

describe("the side panel", () => {
  it("has the tabs Info, Critical (a shield with a name), Notes and Tasks with counters", async () => {
    renderAnnotated();
    await openOrder();
    const tabs = within(await screen.findByRole("tablist", { name: PANEL_TABS })).getAllByRole(
      "tab",
    );
    expect(tabs.map((tab) => tab.getAttribute("aria-label") ?? tab.textContent)).toEqual([
      "Info",
      "Critical",
      "Notes2",
      "Tasks1/3",
    ]);
    expect(screen.getByRole("tab", { name: "Info" }).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByRole("region", { name: "Source" })).toBeTruthy();
  });

  it("moves between the tabs with the arrow keys", async () => {
    renderAnnotated();
    await openOrder();
    const info = await screen.findByRole("tab", { name: "Info" });
    info.focus();
    fireEvent.keyDown(info, { key: "ArrowRight" });
    expect(screen.getByRole("tab", { name: "Critical" }).getAttribute("aria-selected")).toBe(
      "true",
    );
    fireEvent.keyDown(screen.getByRole("tab", { name: "Critical" }), { key: "ArrowLeft" });
    expect(screen.getByRole("tab", { name: "Info" }).getAttribute("aria-selected")).toBe("true");
  });

  it("reads the annotations of the shown symbol when a tab other than Info opens", async () => {
    renderAnnotated({ "annotations.list": () => answer(page([MARK, NOTE, TASK, DONE_TASK])) });
    await openOrder();
    fireEvent.click(await screen.findByRole("tab", { name: /Notes/ }));
    expect(await screen.findByText(/Discount before/)).toBeTruthy();
    expect(bodiesOf("annotations.list")[0]).toMatchObject({
      project: 1,
      symbol: { kind: "class", name: "Shop.Domain.Order" },
    });
    expect(callsOf("annotations.list")).toBe(1);
  });

  it("draws a note as text: bold, an https link, and no link for another scheme", async () => {
    renderAnnotated({ "annotations.list": () => answer(page([NOTE])) });
    await openOrder();
    fireEvent.click(await screen.findByRole("tab", { name: /Notes/ }));
    const bold = await screen.findByText("tax");
    expect(bold.tagName).toBe("STRONG");
    const link = screen.getByRole("link", { name: "https://example.org/wiki" });
    expect(link.getAttribute("href")).toBe("https://example.org/wiki");
    expect(link.getAttribute("rel")).toContain("noopener");
    expect(screen.queryByRole("link", { name: "docs" })).toBeNull();
  });

  it("adds a note and reads the counters again", async () => {
    renderAnnotated({
      "annotations.list": () => answer(page([NOTE])),
      "annotations.add_note": () => answer(NOTE),
    });
    await openOrder();
    fireEvent.click(await screen.findByRole("tab", { name: /Notes/ }));
    fireEvent.click(await screen.findByRole("button", { name: "Add note" }));
    const field = screen.getByLabelText("Text of the note");
    fireEvent.input(field, { target: { value: "A new note" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(callsOf("annotations.add_note")).toBe(1));
    expect(bodiesOf("annotations.add_note")[0]).toEqual({
      project: 1,
      target: { kind: "class", name: "Shop.Domain.Order" },
      text: "A new note",
    });
    await waitFor(() => expect(callsOf("annotations.summary")).toBe(2));
  });

  it.each([
    ["ANN-102", 400],
    ["API-401", 401],
  ])("shows the message of the code %s when the server refuses", async (code, status) => {
    renderAnnotated({
      "annotations.list": () => answer(page([])),
      "annotations.add_note": () => answer({ code }, status),
    });
    await openOrder();
    fireEvent.click(await screen.findByRole("tab", { name: /Notes/ }));
    fireEvent.click(await screen.findByRole("button", { name: "Add note" }));
    fireEvent.input(screen.getByLabelText("Text of the note"), { target: { value: "x" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    const message = await screen.findByRole("alert");
    expect(message.textContent).toBe(errorText(code as ErrorCode));
    expect(message.textContent).not.toContain(code);
    // The form stays open with what was typed.
    expect((screen.getByLabelText("Text of the note") as HTMLTextAreaElement).value).toBe("x");
  });

  it("ticks a task done, edits a note and deletes after a second confirmation", async () => {
    renderAnnotated({
      "annotations.list": () => answer(page([TASK, NOTE])),
      "annotations.update": () => answer(TASK),
      "annotations.delete": () => answer({}),
    });
    await openOrder();
    fireEvent.click(await screen.findByRole("tab", { name: /Tasks/ }));
    fireEvent.click(await screen.findByRole("checkbox", { name: "Done: Add a rounding test" }));
    await waitFor(() => expect(callsOf("annotations.update")).toBe(1));
    expect(bodiesOf("annotations.update")[0]).toEqual({ project: 1, id: 3, status: "done" });
    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    expect(callsOf("annotations.delete")).toBe(0);
    fireEvent.click(screen.getByRole("button", { name: "Delete for good?" }));
    await waitFor(() => expect(bodiesOf("annotations.delete")).toEqual([{ project: 1, id: 3 }]));
  });

  it("adds a task with title, due date and assignee", async () => {
    renderAnnotated({
      "annotations.list": () => answer(page([])),
      "annotations.add_task": () => answer(TASK),
    });
    await openOrder();
    fireEvent.click(await screen.findByRole("tab", { name: /Tasks/ }));
    fireEvent.click(await screen.findByRole("button", { name: "Add task" }));
    fireEvent.input(screen.getByLabelText("Title"), { target: { value: "Write a test" } });
    fireEvent.input(screen.getByLabelText("Due date"), { target: { value: "2026-11-20" } });
    fireEvent.input(screen.getByLabelText("Assignee"), { target: { value: "Anna" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(callsOf("annotations.add_task")).toBe(1));
    expect(bodiesOf("annotations.add_task")[0]).toMatchObject({
      project: 1,
      target: { kind: "class", name: "Shop.Domain.Order" },
      title: "Write a test",
      assignee: "Anna",
      status: "open",
    });
    expect(bodiesOf("annotations.add_task")[0]?.["due"]).toBeGreaterThan(0);
  });

  it("marks and unmarks from the tab Critical", async () => {
    renderAnnotated({
      "annotations.list": () => answer(page([MARK])),
      "annotations.delete": () => answer({}),
    });
    await openOrder();
    fireEvent.click(await screen.findByRole("tab", { name: "Critical" }));
    expect(await screen.findByText("Money moves here.")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Remove mark" }));
    await waitFor(() => expect(bodiesOf("annotations.delete")).toEqual([{ project: 1, id: 1 }]));
  });
});

describe("the keys", () => {
  it("K marks a member that has no mark and takes it off the type that has one", async () => {
    renderAnnotated({
      "annotations.list": () => answer(page([MARK])),
      "annotations.mark_critical": () => answer(MARK),
      "annotations.delete": () => answer({}),
    });
    await openOrder();
    await screen.findByText("decimal Total()");
    fireEvent.keyDown(document.body, { key: "k" });
    await waitFor(() => expect(callsOf("annotations.delete")).toBe(1));
    expect(bodiesOf("annotations.list")[0]).toMatchObject({
      symbol: { name: "Shop.Domain.Order" },
    });
    fireEvent.click(screen.getByText("Id"));
    await screen.findByRole("heading", { name: "Id" });
    fireEvent.keyDown(document.body, { key: "k" });
    await waitFor(() => expect(callsOf("annotations.mark_critical")).toBe(1));
    expect(bodiesOf("annotations.mark_critical")[0]).toEqual({
      project: 1,
      target: { kind: "property", name: "Shop.Domain.Order.Id" },
    });
  });

  it("N and T open the form of a note and of a task for the selection", async () => {
    renderAnnotated({ "annotations.list": () => answer(page([])) });
    await openOrder();
    await screen.findByText("decimal Total()");
    fireEvent.keyDown(document.body, { key: "n" });
    const note = await screen.findByLabelText("Text of the note");
    expect(document.activeElement).toBe(note);
    fireEvent.keyDown(document.body, { key: "Escape" });
    fireEvent.keyDown(document.body, { key: "t" });
    expect(await screen.findByLabelText("Title")).toBeTruthy();
  });

  it("are not shortcuts while typing in a field", async () => {
    renderAnnotated({ "annotations.list": () => answer(page([])) });
    await openOrder();
    await screen.findByText("decimal Total()");
    const filter = screen.getByLabelText("Filter tree");
    fireEvent.keyDown(filter, { key: "k" });
    fireEvent.keyDown(filter, { key: "n" });
    expect(callsOf("annotations.mark_critical")).toBe(0);
    expect(screen.queryByLabelText("Text of the note")).toBeNull();
  });

  it("show a message and do nothing without a selection", async () => {
    renderAnnotated();
    await screen.findByText("Domain");
    fireEvent.keyDown(document.body, { key: "k" });
    expect(callsOf("annotations.mark_critical")).toBe(0);
    expect(callsOf("annotations.list")).toBe(0);
  });
});

describe("the switch Show annotations", () => {
  it("shows the text of notes and tasks under their rows, read in two calls for the project", async () => {
    openProject(1);
    const total = { kind: "method", name: "Shop.Domain.Order.Total" };
    serveBodies({
      "explorer.outline": () => answer(OUTLINE),
      "explorer.symbol": () => answer(ORDER),
      "annotations.summary": () => answer(page(SUMMARY)),
      "annotations.list": (body) =>
        answer(
          page(
            body["kind"] === "note"
              ? [{ ...NOTE, id: 20, target: total, text: "Rounds half up." }]
              : [{ ...TASK, id: 21, target: total, title: "Test the rounding" }],
          ),
        ),
    });
    render(() => <ExplorerScreen />);
    await openOrder();
    await screen.findByText("decimal Total()");
    expect(screen.queryByText("Rounds half up.")).toBeNull();
    fireEvent.click(screen.getByLabelText("Show annotations"));
    expect(await screen.findByText("Rounds half up.")).toBeTruthy();
    expect(screen.getByText("Test the rounding")).toBeTruthy();
    expect(screen.getByText("Note:")).toBeTruthy();
    expect(screen.getByText("Task:")).toBeTruthy();
    expect(bodiesOf("annotations.list").map((body) => body["kind"])).toEqual(["note", "task"]);
    fireEvent.click(screen.getByLabelText("Show annotations"));
    await waitFor(() => expect(screen.queryByText("Rounds half up.")).toBeNull());
  });
});
