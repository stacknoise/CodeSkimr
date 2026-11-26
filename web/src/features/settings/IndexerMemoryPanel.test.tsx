import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import type { EffectiveSettings } from "./effective";
import { IndexerMemoryPanel, parseMegabytes } from "./IndexerMemoryPanel";

function effective(origin: "default" | "cli" = "default"): EffectiveSettings {
  const setting = (key: string, value: string, from: "default" | "cli" = "default") => ({
    key,
    value,
    origin: from,
    hidden: false,
  });
  return {
    user_file: null,
    overrides: [],
    settings: [
      setting("level2.max_memory_mb", "4096"),
      setting("level2.indexer_memory_mb.scip-java", "0"),
      setting("level2.indexer_memory_mb.scip-typescript", "12288", origin),
    ],
  };
}

function saved(limits: Record<string, number>) {
  return { indexer_memory_mb: limits };
}

function sentSets(): unknown[] {
  return fetchMock.mock.calls
    .filter(([path]) => String(path).endsWith("ui_settings.set_indexer_memory"))
    .map(([, init]) => JSON.parse((init as RequestInit).body as string));
}

useServerSession();

beforeEach(() => setLanguage("en"));

type Handlers = Record<string, () => Response>;

/** Shows the panel against a server that holds nothing saved; `extra` adds or replaces commands. */
async function showPanel(extra: Handlers = {}, origin: "default" | "cli" = "default") {
  serve({
    "settings.effective": () => answer(effective(origin)),
    "ui_settings.get": () => answer(saved({})),
    ...extra,
  });
  render(() => <IndexerMemoryPanel />);
  return (await screen.findAllByRole("textbox"))[0] as HTMLElement;
}

/** Types `text` into the field of the first indexer and presses its Save button. */
function typeAndSave(field: HTMLElement, text: string) {
  fireEvent.input(field, { target: { value: text } });
  fireEvent.click(screen.getAllByRole("button", { name: "Save" })[0] as HTMLElement);
}

describe("the memory limit per indexer", () => {
  it("shows a row for each indexer with the value in effect and says what 0 means", async () => {
    expect(await showPanel()).toHaveValue("0");
    expect(screen.getByLabelText("Memory limit of scip-typescript in MB")).toHaveValue("12288");
    expect(screen.getByText("follows the general limit of 4096 MB")).toBeInTheDocument();
  });

  it("saves a limit and forgets it again", async () => {
    const field = await showPanel({
      "ui_settings.set_indexer_memory": () => answer(saved({ "scip-java": 2048 })),
    });
    typeAndSave(field, "2048");
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Saved. It counts from the next start.",
    );
    expect(sentSets()).toEqual([{ indexer: "scip-java", megabytes: 2048 }]);
    expect(field).toHaveValue("2048");
    fireEvent.click(await screen.findByRole("button", { name: "Use the default" }));
    await waitFor(() =>
      expect(sentSets()).toContainEqual({ indexer: "scip-java", megabytes: null }),
    );
  });

  it("refuses a value that is not a whole number in range before it asks the server", async () => {
    typeAndSave(await showPanel(), "lots");
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Enter a whole number from 0 to 1,048,576.",
    );
    expect(sentSets()).toEqual([]);
  });

  it("tells when the server could not save the value", async () => {
    const field = await showPanel({
      "ui_settings.set_indexer_memory": () => answer({ code: "APP-201" }, 500),
    });
    typeAndSave(field, "1024");
    expect(await screen.findByRole("alert")).toHaveTextContent("Could not be saved.");
  });

  it("leaves a value that a command line flag sets alone and says so", async () => {
    await showPanel({}, "cli");
    const field = screen.getByLabelText("Memory limit of scip-typescript in MB");
    await waitFor(() => expect(field).toBeDisabled());
    expect(screen.getByText(/A command line flag sets this value/)).toBeInTheDocument();
  });

  it("says the same in German", async () => {
    setLanguage("de");
    await showPanel();
    expect(screen.getByLabelText("Speicherlimit von scip-java in MB")).toBeInTheDocument();
    expect(screen.getByText("folgt dem allgemeinen Limit von 4096 MB")).toBeInTheDocument();
  });
});

describe("the megabytes of a text", () => {
  it("accepts whole numbers up to the limit of the server and nothing else", () => {
    expect(parseMegabytes(" 8192 ")).toBe(8192);
    expect(parseMegabytes("0")).toBe(0);
    expect(parseMegabytes("1048576")).toBe(1_048_576);
    for (const bad of ["", "1048577", "-1", "1.5", "1e3", "ten"]) {
      expect(parseMegabytes(bad)).toBeNull();
    }
  });
});
