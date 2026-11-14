import { expect } from "vitest";
import { waitFor } from "@solidjs/testing-library";
import { fetchMock } from "./commands";
import type { ToolsInfo } from "../features/projects/types";

/** What a scan reports when it did not check the tools (an incremental scan). */
export const UNCHECKED: ToolsInfo = { checked_at: null, languages: [], tools: [] };

/** The answer of `projects.rescan` for `project`. */
export function rescanned(project: number): Record<string, unknown> {
  return {
    project,
    snapshot: 6,
    files: 3,
    symbols: 9,
    references: 4,
    seconds: 0.2,
    tools: UNCHECKED,
  };
}

/** Waits until the UI has asked for a rescan. */
export async function rescanAsked(): Promise<void> {
  await waitFor(() =>
    expect(fetchMock.mock.calls.some(([path]) => String(path).endsWith("projects.rescan"))).toBe(
      true,
    ),
  );
}
