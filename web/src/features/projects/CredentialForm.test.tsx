import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { describe, expect, it, vi } from "vitest";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { UNCHECKED } from "../../tests/scanned";
import { ProjectsScreen } from "./ProjectsScreen";

const SECRET = "ghp_CANARY-ui-8e1f";

useServerSession();

function bodyOf(command: string): Record<string, unknown> {
  const call = fetchMock.mock.calls.find(([path]) => String(path).endsWith(command));
  return JSON.parse(call?.[1].body);
}

async function type(): Promise<HTMLInputElement> {
  fireEvent.click(await screen.findByText("Store a new login"));
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
  it("stores a credential, forgets the secret and offers the name", async () => {
    serve({
      "projects.list": () => answer([]),
      "credentials.list": () => answer([]),
      "credentials.set": () => answer({ id: "work" }),
    });
    render(() => <ProjectsScreen />);
    await fill();
    expect(await screen.findByText("Stored work")).toBeInTheDocument();
    expect(bodyOf("credentials.set")).toEqual({ id: "work", user: "anton", secret: SECRET });
    const secret = screen.getByLabelText("Token or password") as HTMLInputElement;
    expect(secret.type).toBe("password");
    expect(secret.value).toBe("");
    expect(await screen.findByRole("status")).toHaveTextContent("Stored work");
    const choice = screen.getByLabelText("Login") as HTMLSelectElement;
    await waitFor(() => expect(choice.value).toBe("work"));
  });

  it("sends the chosen name with the new project", async () => {
    serve({
      "projects.list": () => answer([]),
      "credentials.list": () => answer([{ id: "work", stored: true }]),
      "projects.add": () =>
        answer({
          project: 1,
          snapshot: 5,
          files: 3,
          symbols: 9,
          references: 4,
          seconds: 1,
          tools: UNCHECKED,
        }),
    });
    render(() => <ProjectsScreen />);
    const field = await screen.findByPlaceholderText(/Paste a folder path/);
    fireEvent.input(field, { target: { value: "https://git.example.com/team/repo.git" } });
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
    render(() => <ProjectsScreen />);
    const field = await screen.findByPlaceholderText(/Paste a folder path/);
    fireEvent.input(field, { target: { value: "/work/shop" } });
    const secret = await type();
    fireEvent.keyDown(secret, { key: "Enter" });
    expect(await screen.findByText("Stored work")).toBeInTheDocument();
    const added = fetchMock.mock.calls.some(([path]) => String(path).endsWith("projects.add"));
    expect(added).toBe(false);
  });

  it("empties the secret when the section is closed without storing", async () => {
    serve({ "projects.list": () => answer([]), "credentials.list": () => answer([]) });
    render(() => <ProjectsScreen />);
    const secret = await type();
    const section = secret.closest("details") as HTMLDetailsElement;
    section.open = false;
    fireEvent(section, new Event("toggle"));
    expect(secret.value).toBe("");
  });

  it("sends no credential when none is chosen", async () => {
    serve({
      "projects.list": () => answer([]),
      "credentials.list": () => answer([]),
      "projects.add": () =>
        answer({
          project: 1,
          snapshot: 5,
          files: 3,
          symbols: 9,
          references: 4,
          seconds: 1,
          tools: UNCHECKED,
        }),
    });
    render(() => <ProjectsScreen />);
    const field = await screen.findByPlaceholderText(/Paste a folder path/);
    fireEvent.input(field, { target: { value: "/work/shop" } });
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
    render(() => <ProjectsScreen />);
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
    render(() => <ProjectsScreen />);
    await fill();
    expect(await screen.findByRole("alert")).toBeInTheDocument();
    expect((screen.getByLabelText("Token or password") as HTMLInputElement).value).toBe("");
    nowhere(spy);
  });
});
