import { ApiError } from "../shared/api/client";
import { COMMAND as EXPLORER } from "../features/explorer/consts";
import type { SourceInfo } from "../features/explorer/types";
import { GRAPH_COMMAND } from "../features/graph/consts";
import { COMMAND as PROJECTS, SCAN_MODE, SOURCE_KIND } from "../features/projects/consts";
import type { ProjectRow } from "../features/projects/types";
import { SEARCH_COMMAND } from "../features/search/consts";
import { REF, USAGES_COMMAND, WAY } from "../features/usages/consts";
import type { Walk, UsagesAnswer } from "../features/usages/types";
import { NO_LINES, STATUS_NOT_FOUND } from "./consts";
import { searchSymbols } from "./search";
import type { ExportData } from "./types";

/** What the usages screen asks for (crates/app/src/ui_commands/usages.rs). */
interface UsagesInput {
  symbol: number;
  way: string;
  depth: number;
  include_tests: boolean;
}

function notFound(): ApiError {
  return new ApiError(STATUS_NOT_FOUND, null);
}

function projectRow(data: ExportData): ProjectRow {
  const { project } = data;
  return {
    id: project.id,
    name: project.name,
    kind: SOURCE_KIND.folder,
    location: "",
    branch: null,
    commit: project.commit,
    scan: {
      snapshot: data.snapshot,
      created: project.created,
      mode: project.mode === "full" ? SCAN_MODE.full : SCAN_MODE.incremental,
      files: project.files,
      symbols: project.symbols,
      references: project.references,
    },
    types: 0,
    loc: 0,
    languages: [],
    running: false,
  };
}

/** The hits that the depth and the tests switch of the toolbar leave. The count of a walk that
 * was cut stays a count of what was found. */
function narrowed(walk: Walk | null, input: UsagesInput, depth: number): Walk | null {
  if (walk === null) {
    return null;
  }
  const hits = walk.hits.filter(
    (hit) => hit.depth <= depth && (input.include_tests || hit.kind !== REF.test),
  );
  const complete = walk.hits.length === walk.total;
  return { total: complete ? hits.length : walk.total - (walk.hits.length - hits.length), hits };
}

function usages(data: ExportData, input: UsagesInput): UsagesAnswer {
  const stored = data.usages[input.symbol];
  if (stored === undefined) {
    throw notFound();
  }
  const { answer } = stored;
  const depth = input.depth === 0 ? answer.depth : Math.min(input.depth, answer.depth);
  return {
    ...answer,
    depth,
    incoming: input.way === WAY.outgoing ? null : narrowed(answer.incoming, input, depth),
    outgoing: input.way === WAY.incoming ? null : narrowed(answer.outgoing, input, depth),
    summary: input.include_tests ? answer.summary : stored.summary_without_tests,
  };
}

function source(data: ExportData, symbol: number): SourceInfo {
  const info = data.symbols[symbol];
  if (info === undefined) {
    throw notFound();
  }
  return {
    path: info.path,
    first_line: info.first_line,
    last_line: info.last_line,
    available: false,
    excluded: true,
    lines: NO_LINES,
    truncated: false,
  };
}

/** Answers a command from the data of the export: the same answers the server gave when the
 * export was made, narrowed to what the toolbar of a screen asks for. Nothing here reaches a
 * server, and no command changes anything. */
export function answer(data: ExportData, id: string, input: unknown): unknown {
  const asked = (input ?? {}) as { symbol: number; query: string };
  switch (id) {
    case PROJECTS.list:
      return [projectRow(data)];
    case EXPLORER.outline:
      return data.outline;
    case EXPLORER.symbol:
      return data.symbols[asked.symbol] ?? fail();
    case EXPLORER.source:
      return source(data, asked.symbol);
    case GRAPH_COMMAND:
      return data.graph;
    case USAGES_COMMAND:
      return usages(data, input as UsagesInput);
    case SEARCH_COMMAND:
      return { snapshot: data.snapshot, hits: searchSymbols(data, asked.query) };
    default:
      throw notFound();
  }
}

function fail(): never {
  throw notFound();
}

let current: ExportData | null = null;

/** Makes `data` the answer of every command that the screens run. */
export function installData(data: ExportData): void {
  current = data;
}

/** Stands in for the call of the server (`shared/api/commands.ts`) in the export build. */
export function runCommand<Output>(id: string, input?: unknown): Promise<Output> {
  try {
    if (current === null) {
      throw notFound();
    }
    return Promise.resolve(answer(current, id, input) as Output);
  } catch (error) {
    return Promise.reject(error);
  }
}
