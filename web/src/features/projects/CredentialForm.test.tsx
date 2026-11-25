import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { describe, expect, it, vi } from "vitest";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { UNCHECKED } from "../../tests/scanned";
import { SettingsScreen } from "../settings/SettingsScreen";
import { ProjectsScreen } from "./ProjectsScreen";

const SECRET = "ghp_CANARY-ui-8e1f";

useServerSession();

const SCANNED = () =>
  answer({
    project: 1,
    snapshot: 5,
    files: 3,
    symbols: 9,
    references: 4,
    seconds: 1,
    tools: UNCHECKED,
  });

/** Shows the screen and pastes `path` as the source of a new project. */
async function pasteSource(path: string): Promise<HTMLElement> {
  render(() => <ProjectsScreen />);
  const field = await screen.findByPlaceholderText(/Paste a folder path/);
  fireEvent.input(field, { target: { value: path } });
  return field;
}

function bodyOf(command: string): Record<string, unknown> {
  const call = fetchMock.mock.calls.find(([path]) => String(path).endsWith(command));
  return JSON.parse(call?.[1].body);
}

async function type(): Promise<HTMLInputElement> {
  const summary = await screen.findByText("Store a new login");
  if (!(summary.closest("details") as HTMLDetailsElement).open) {
    fireEvent.click(summary);
  }
  fireEvent.input(screen.getByLabelText("Name"), { target: { value: "work" } });
  fireEvent.input(screen.getByLabelText("User name"), { target: { value: "anton" } });
  const secret = screen.getByLabelText("Token or password") as HTMLInputElement;
  fireEvent.input(secret, { target: { value: SECRET } });
  return secret;
}

async function fill(): Promise<void> {
  await type();
  fireEvent.click(screen.getByRole("button", { name: "Store login" }));
}

/** Everything the page could have kept or printed: the secret is in none of it. */
function nowhere(spy: ReturnType<typeof vi.spyOn>): void {
  const kept = [JSON.stringify({ ...localStorage }), JSON.stringify({ ...sessionStorage })];
  expect(kept.some((text) => text.includes(SECRET))).toBe(false);
  expect(JSON.stringify(spy.mock.calls).includes(SECRET)).toBe(false);
  const shown = [...document.querySelectorAll("p")].map((node) => node.textContent ?? "");
  expect(shown.some((text) => text.includes(SECRET))).toBe(false);
}

describe("the login of a Git project", () => {
  it("stores a credential in the settings, forgets the secret and lists the name", async () => {
    serve({
      "projects.list": () => answer([]),
      "credentials.list": () => answer([]),
      "credentials.set": () => answer({ id: "work" }),
    });
    render(() => <SettingsScreen />);
    await fill();
    expect(await screen.findByText("Stored work")).toBeInTheDocument();
    expect(bodyOf("credentials.set")).toEqual({ id: "work", user: "anton", secret: SECRET });
    const secret = screen.getByLabelText("Token or password") as HTMLInputElement;
    expect(secret.type).toBe("password");
    expect(secret.value).toBe("");
    expect(await screen.findByRole("button", { name: "Remove login work" })).toBeInTheDocument();
  });

  it("removes a stored credential from the list and the keystore", async () => {
    serve({
      "credentials.list": () => answer([{ id: "work", stored: true }]),
      "credentials.delete": () => answer({ id: "work" }),
    });
    render(() => <SettingsScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "Remove login work" }));
    await waitFor(() => expect(bodyOf("credentials.delete")).toEqual({ id: "work" }));
    await waitFor(() =>
      expect(screen.queryByRole("button", { name: "Remove login work" })).toBeNull(),
    );
  });

  it("shows branch and login only for a Git source", async () => {
    serve({ "projects.list": () => answer([]), "credentials.list": () => answer([]) });
    render(() => <ProjectsScreen />);
    await screen.findByPlaceholderText(/Paste a folder path/);
    expect(screen.queryByLabelText("Login")).toBeNull();
    expect(screen.queryByLabelText("Branch")).toBeNull();
    fireEvent.click(screen.getByRole("radio", { name: "Git" }));
    expect(await screen.findByLabelText("Login")).toBeInTheDocument();
    expect(screen.getByLabelText("Branch")).toBeInTheDocument();
  });

  it("sends the chosen name with the new project", async () => {
    serve({
      "projects.list": () => answer([]),
      "credentials.list": () => answer([{ id: "work", stored: true }]),
      "projects.add": () => SCANNED(),
    });
    await pasteSource("https://git.example.com/team/repo.git");
    const choice = (await screen.findByLabelText("Login")) as HTMLSelectElement;
    await screen.findByRole("option", { name: "work" });
    fireEvent.change(choice, { target: { value: "work" } });
    fireEvent.click(screen.getByRole("button", { name: "Add and scan" }));
    await waitFor(() => expect(bodyOf("projects.add").credential).toBe("work"));
  });

  it("stores the login on Enter instead of adding the project", async () => {
    serve({
      "projects.list": () => answer([]),
      "credentials.list": () => answer([]),
      "credentials.set": () => answer({ id: "work" }),
      "projects.add": () => answer({ code: "PRJ-101" }, 400),
    });
    render(() => <SettingsScreen />);
    const secret = await type();
    fireEvent.keyDown(secret, { key: "Enter" });
    expect(await screen.findByText("Stored work")).toBeInTheDocument();
    const added = fetchMock.mock.calls.some(([path]) => String(path).endsWith("projects.add"));
    expect(added).toBe(false);
  });

  it("empties the secret when the section is closed without storing", async () => {
    serve({ "projects.list": () => answer([]), "credentials.list": () => answer([]) });
    render(() => <SettingsScreen />);
    const secret = await type();
    const section = secret.closest("details") as HTMLDetailsElement;
    section.open = false;
    fireEvent(section, new Event("toggle"));
    expect(secret.value).toBe("");
  });

  it("empties the secret when the screen is left without storing", async () => {
    serve({ "projects.list": () => answer([]), "credentials.list": () => answer([]) });
    const view = render(() => <SettingsScreen />);
    const secret = await type();
    view.unmount();
    expect(secret.value).toBe("");
  });

  it("empties the secret when the request cannot be sent at all", async () => {
    serve({
      "projects.list": () => answer([]),
      "credentials.list": () => answer([]),
      "credentials.set": () => {
        throw new TypeError("network down");
      },
      "tools.indexers": () => answer({ checked_at: null, languages: [], tools: [] }),
      "settings.effective": () => answer({ user_file: null, overrides: [], settings: [] }),
    });
    const spy = vi.spyOn(console, "error");
    render(() => <SettingsScreen />);
    const secret = await type();
    fireEvent.click(screen.getByRole("button", { name: "Store login" }));
    await waitFor(() => expect(secret.value).toBe(""));
    expect(await screen.findByRole("alert")).toBeInTheDocument();
    nowhere(spy);
  });

  it("sends no credential when none is chosen", async () => {
    serve({
      "projects.list": () => answer([]),
      "credentials.list": () => answer([]),
      "projects.add": () => SCANNED(),
    });
    await pasteSource("/work/shop");
    fireEvent.click(screen.getByRole("button", { name: "Add and scan" }));
    await waitFor(() =>
      expect(bodyOf("projects.add")).toEqual({ source: "/work/shop", branch: "main" }),
    );
  });

  it("still lets a project be added when the names cannot be read", async () => {
    serve({
      "projects.list": () => answer([]),
      "credentials.list": () => answer({ code: "API-401" }, 401),
    });
    await pasteSource("https://git.example.com/team/repo.git");
    const choice = (await screen.findByLabelText("Login")) as HTMLSelectElement;
    expect(choice.value).toBe("");
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("names a refusal and still empties the secret", async () => {
    serve({
      "projects.list": () => answer([]),
      "credentials.list": () => answer([]),
      "credentials.set": () => answer({ code: "API-101" }, 400),
    });
    const spy = vi.spyOn(console, "error");
    render(() => <SettingsScreen />);
    await fill();
    const field = screen.getByLabelText("Token or password") as HTMLInputElement;
    await waitFor(() => expect(field.value).toBe(""));
    expect((await screen.findAllByRole("alert")).length).toBeGreaterThan(0);
    nowhere(spy);
  });
});
