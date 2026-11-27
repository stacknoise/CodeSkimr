import { createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import {
  LOG_CLEAR_COMMAND,
  LOG_COPIED_LINES,
  LOG_LINE_BREAK,
  LOG_OPEN_FOLDER_COMMAND,
  LOG_SET_LEVEL_COMMAND,
  LOG_SHOWN_LINES,
  LOG_STATUS_COMMAND,
  LOG_TAIL_COMMAND,
} from "./consts";

/** What `log.status` answers. */
export interface LogStatus {
  level: string;
  default_level: string;
  levels: string[];
  adjustable: boolean;
  file: string | null;
  size_bytes: number;
  max_file_bytes: number;
  files_kept: number;
  resets_in_seconds: number | null;
  can_open_folder: boolean;
}

interface LogLines {
  lines: string[];
}

const [state, setState] = createSignal<LogStatus | null>(null);
const [lines, setLines] = createSignal<string[]>([]);

/** The log as the last call got it; `null` before the first answer. */
export const logStatus = state;
/** The last lines of the log file. */
export const logLines = lines;

/** Reads the status and the last lines.
 *
 * @throws ApiError if the server cannot be asked.
 */
export async function loadLog(): Promise<void> {
  setState(await runCommand<LogStatus>(LOG_STATUS_COMMAND));
  const tail = await runCommand<LogLines>(LOG_TAIL_COMMAND, { lines: LOG_SHOWN_LINES });
  setLines(tail.lines);
}

/** Chooses a level; it counts at once. */
export async function chooseLogLevel(level: string): Promise<void> {
  setState(await runCommand<LogStatus>(LOG_SET_LEVEL_COMMAND, { level }));
}

/** Empties the log file and removes the old ones. */
export async function clearLog(): Promise<void> {
  setState(await runCommand<LogStatus>(LOG_CLEAR_COMMAND));
  setLines([]);
}

/** Opens the folder of the log files on the machine of the app. */
export async function openLogFolder(): Promise<void> {
  await runCommand<null>(LOG_OPEN_FOLDER_COMMAND);
}

/** The last lines of the log as one text for the clipboard. */
export async function logText(): Promise<string> {
  const tail = await runCommand<LogLines>(LOG_TAIL_COMMAND, { lines: LOG_COPIED_LINES });
  return tail.lines.join(LOG_LINE_BREAK);
}
