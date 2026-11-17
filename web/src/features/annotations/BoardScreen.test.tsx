import { fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { openProject } from "../../shared/project/active";
import { bodiesOf, DONE_TASK, MARK, page, serveBodies, TASK } from "../../tests/annotations";
import { answer, useServerSession } from "../../tests/commands";
import { resetBoard } from "./board";
import { BoardScreen } from "./BoardScreen";
import { forgetMe, rememberAuthor } from "./me";

useServerSession();

const OTHER = {
  ...TASK,
  id: 5,
  title: "Check the XML output",
  status: "in_progress",
  assignee: "Mara",
  due: null,
  target: { kind: "class", name: "Shop.Billing.InvoiceWriter" },
};

function serveBoard() {
  serveBodies({
    "annotations.list": (body) => {
      if (body.kind === "task") {
        return answer(page([TASK, OTHER, DONE_TASK]));
      }
      return answer(body.kind === "critical" ? page([MARK]) : page([], null));
    },
    "annotations.update": () => answer(TASK),
    "annotations.summary": () => answer(page([])),
  });
}

beforeEach(() => {
  openProject(1);
  forgetMe();
});
afterEach(resetBoard);

describe("the board of the tasks", () => {
  it("has a column for each status with its number, and reads the project with three calls", async () => {
    serveBoard();
    render(() => <BoardScreen />);
    expect(await screen.findByRole("heading", { name: "Open (1)" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "In progress (1)" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Done (1)" })).toBeTruthy();
    expect(screen.getByText("Tasks 2 open")).toBeTruthy();
    expect(bodiesOf("annotations.list")).toHaveLength(3);
  });

  it("marks the task of a critical symbol with the shield and its label", async () => {
    serveBoard();
    render(() => <BoardScreen />);
    const card = (await screen.findByText("Add a rounding test")).closest("li")!;
    expect(within(card).getByRole("img", { name: "Critical" })).toBeTruthy();
    const other = screen.getByText("Check the XML output").closest("li")!;
    expect(within(other).queryByRole("img", { name: "Critical" })).toBeNull();
  });

  it("filters with switches that say whether they are on", async () => {
    serveBoard();
    render(() => <BoardScreen />);
    await screen.findByText("Add a rounding test");
    const critical = screen.getByRole("button", { name: "Only critical" });
    fireEvent.click(critical);
    expect(critical.getAttribute("aria-pressed")).toBe("true");
    expect(screen.queryByText("Check the XML output")).toBeNull();
    fireEvent.click(critical);
    fireEvent.click(screen.getByRole("button", { name: "Due" }));
    expect(screen.queryByText("Check the XML output")).toBeNull();
    expect(screen.getByText("Add a rounding test")).toBeTruthy();
  });

  it("keeps Mine off until the name of the user is known, then shows the tasks assigned to it", async () => {
    serveBoard();
    render(() => <BoardScreen />);
    await screen.findByText("Add a rounding test");
    const mine = screen.getByRole("button", { name: "Mine" });
    expect((mine as HTMLButtonElement).disabled).toBe(true);
    rememberAuthor("Mara");
    await waitFor(() => expect((mine as HTMLButtonElement).disabled).toBe(false));
    fireEvent.click(mine);
    expect(screen.getByText("Check the XML output")).toBeTruthy();
    expect(screen.queryByText("Add a rounding test")).toBeNull();
  });

  it("moves a task to another column with its status list", async () => {
    serveBoard();
    render(() => <BoardScreen />);
    const card = (await screen.findByText("Add a rounding test")).closest("li")!;
    fireEvent.change(within(card).getByRole("combobox", { name: "Status" }), {
      target: { value: "done" },
    });
    await waitFor(() =>
      expect(bodiesOf("annotations.update")).toEqual([{ project: 1, id: 3, status: "done" }]),
    );
  });

  it("says so when the server has no tasks or cannot be read", async () => {
    serveBodies({});
    render(() => <BoardScreen />);
    expect(await screen.findByText("The tasks could not be read.")).toBeTruthy();
  });
});
