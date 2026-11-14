import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { UNCHECKED } from "../../tests/scanned";
import { ProjectsScreen } from "./ProjectsScreen";

const SECRET = "ghp_CANARY-ui-8e1f";

useServerSession();

function bodyOf(command: string): Record<string, unknown> {
  const call = fetchMock.mock.calls.find(([path]) => String(path).endsWith(command));
  return JSON.parse(call?.[1].body);
}

async function fill(): Promise<void> {
  fireEvent.click(await screen.findByText("Store a new login"));
  fireEvent.input(screen.getByLabelText("Name"), { target: { value: "work" } });
  fireEvent.input(screen.getByLabelText("User name"), { target: { value: "anton" } });
  fireEvent.input(screen.getByLabelText("Token or password"), { target: { value: SECRET } });
  fireEvent.click(screen.getByRole("button", { name: "Store login" }));
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
    expect(document.body.innerHTML.includes(SECRET)).toBe(false);
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

  it("names a refusal and still empties the secret", async () => {
    serve({
      "projects.list": () => answer([]),
      "credentials.list": () => answer([]),
      "credentials.set": () => answer({ code: "API-101" }, 400),
    });
    render(() => <ProjectsScreen />);
    await fill();
    expect(await screen.findByRole("alert")).toBeInTheDocument();
    expect((screen.getByLabelText("Token or password") as HTMLInputElement).value).toBe("");
  });
});
