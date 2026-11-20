import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { answer, serve, useServerSession } from "../../tests/commands";
import { ProjectsScreen } from "./ProjectsScreen";

useServerSession();

/** A server that has the dialog or not, and answers `pick` with what the test gives. */
function dialog(available: boolean, pick: () => ReturnType<typeof answer>) {
  serve({
    "projects.list": () => answer([]),
    "projects.folder_dialog": () => answer({ available }),
    "projects.pick_folder": pick,
  });
}

describe("the Browse button of the new-project form", () => {
  it("is not there without a folder dialog on the machine of the app", async () => {
    dialog(false, () => answer({ path: null }));
    render(() => <ProjectsScreen />);
    await screen.findByLabelText("New project");
    expect(screen.queryByRole("button", { name: "Browse …" })).toBeNull();
  });

  it("puts the chosen folder into the field and asks to check it", async () => {
    dialog(true, () => answer({ path: "/work/shop" }));
    render(() => <ProjectsScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "Browse …" }));
    await waitFor(() =>
      expect(screen.getByPlaceholderText(/Paste a folder path/)).toHaveValue("/work/shop"),
    );
    expect(screen.getByText(/Folder taken over/)).toBeInTheDocument();
  });

  it("leaves the field as it is when the dialog is cancelled", async () => {
    dialog(true, () => answer({ path: null }));
    render(() => <ProjectsScreen />);
    const field = await screen.findByPlaceholderText(/Paste a folder path/);
    fireEvent.input(field, { target: { value: "/typed" } });
    fireEvent.click(await screen.findByRole("button", { name: "Browse …" }));
    await waitFor(() => expect(screen.queryByRole("status")).toBeNull());
    expect(field).toHaveValue("/typed");
    expect(screen.queryByText(/Folder taken over/)).toBeNull();
  });

  it("says that it waits, and stops waiting on request", async () => {
    dialog(true, () => new Promise(() => {}) as never);
    render(() => <ProjectsScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "Browse …" }));
    expect(await screen.findByText("Folder dialog open")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Stop waiting" }));
    await waitFor(() => expect(screen.queryByText("Folder dialog open")).toBeNull());
  });
});
