import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { currentLanguage, setLanguage } from "../../shared/i18n/i18n";
import { SESSION_PATH } from "../../shared/api/consts";
import { startSession } from "../../shared/session/session";
import { answer, serve, useServerSession } from "../../tests/commands";
import type { ProjectRow } from "../projects/types";
import { StatusBar } from "./StatusBar";

const PROJECT: ProjectRow = {
  id: 1,
  name: "shop",
  kind: "folder",
  location: "/work/shop",
  branch: null,
  commit: null,
  scan: null,
  types: 1212,
  loc: 0,
  languages: [],
  running: false,
};

useServerSession();

async function served(info: { startup_ms: number | null; memory_mb: number | null }) {
  serve({
    [SESSION_PATH]: () => answer({}),
    "status.info": () => answer(info),
    "projects.list": () => answer([PROJECT, { ...PROJECT, id: 2, types: 8 }]),
  });
  await startSession();
  return render(() => <StatusBar />);
}

describe("StatusBar", () => {
  it("shows the measured start time, the memory and the counts", async () => {
    setLanguage("en");
    await served({ startup_ms: 180, memory_mb: 41 });
    expect(await screen.findByText("Start 180 ms")).toBeInTheDocument();
    expect(screen.getByText("RAM 41 MB")).toBeInTheDocument();
    expect(await screen.findByText("2 projects · 1,220 types")).toBeInTheDocument();
    expect(screen.getByText(/Connected/)).toBeInTheDocument();
  });

  it("leaves out what the system does not tell", async () => {
    setLanguage("en");
    await served({ startup_ms: null, memory_mb: null });
    await screen.findByText("2 projects · 1,220 types");
    await waitFor(() => expect(screen.queryByText(/^Start /)).toBeNull());
    expect(screen.queryByText(/^RAM /)).toBeNull();
  });

  it("switches the language of all texts", async () => {
    setLanguage("en");
    await served({ startup_ms: 180, memory_mb: 41 });
    await screen.findByText("RAM 41 MB");
    expect(screen.getByRole("button", { name: "EN" })).toHaveAttribute("aria-pressed", "true");
    fireEvent.click(screen.getByRole("button", { name: "DE" }));
    expect(currentLanguage()).toBe("de");
    expect(await screen.findByText("2 Projekte · 1.220 Typen")).toBeInTheDocument();
    expect(screen.getByRole("group", { name: "Sprache" })).toBeInTheDocument();
    setLanguage("en");
  });
});
