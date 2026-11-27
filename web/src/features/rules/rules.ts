import { createSignal } from "solid-js";
import { ApiError } from "../../shared/api/client";
import { runCommand } from "../../shared/api/commands";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { failed, type Failure } from "../explorer/failure";
import { COMMAND } from "./consts";
import type { HeldOut, OpenedOut, RulesOut } from "./types";

const [data, setData] = createSignal<RulesOut | null>(null);
const [failure, setFailure] = createSignal<Failure | null>(null);
/** Counts the loads, so that an answer for a project that is no longer shown is dropped. */
let latest = 0;

/** What the server said about the rules of the project. */
export const rulesData = data;
/** The failure of the last read; `null` if it worked. */
export const rulesFailure = failure;

/** Reads the rules of a project and their violations. */
export async function loadRules(project: number): Promise<void> {
  const mine = ++latest;
  setData(null);
  setFailure(null);
  try {
    const read = await runCommand<RulesOut>(COMMAND.show, { project });
    if (mine === latest) {
      setData(read);
    }
  } catch (error) {
    if (mine === latest) {
      setFailure(failed(error));
    }
  }
}

const [held, setHeld] = createSignal<HeldOut | null>(null);
const [holdFailure, setHoldFailure] = createSignal<ErrorCode | null | undefined>(undefined);

/** What the last "hold state" wrote; `null` if it was not done since the screen was opened. */
export const heldNow = held;
/** The code of the failed "hold state"; `null` for a failure without a code, `undefined` if it
 * did not fail. */
export const holdFailed = holdFailure;

/** Holds the state: every violation of now becomes an old burden. Reads the rules again, so the
 * screen shows what the CI will say. */
export async function holdState(project: number): Promise<void> {
  setHeld(null);
  setHoldFailure(undefined);
  try {
    setHeld(await runCommand<HeldOut>(COMMAND.hold, { project }));
    await loadRules(project);
  } catch (error) {
    setHoldFailure(error instanceof ApiError ? error.code : null);
  }
}

const [opened, setOpened] = createSignal<OpenedOut | null>(null);
const [openFailure, setOpenFailure] = createSignal<ErrorCode | null | undefined>(undefined);

/** What the last "open project file" did; `null` if it was not done since the screen was opened. */
export const openedNow = opened;
/** The code of the failed "open project file"; `null` for a failure without a code, `undefined`
 * if it did not fail. */
export const openFailed = openFailure;

/** Asks the server to open the project file of the project with the program of the system. */
export async function openProjectFile(project: number): Promise<void> {
  setOpened(null);
  setOpenFailure(undefined);
  try {
    setOpened(await runCommand<OpenedOut>(COMMAND.openFile, { project }));
  } catch (error) {
    setOpenFailure(error instanceof ApiError ? error.code : null);
  }
}

/** Forgets what was read, when the screen is left. */
export function resetRules(): void {
  latest += 1;
  setData(null);
  setFailure(null);
  setHeld(null);
  setHoldFailure(undefined);
  setOpened(null);
  setOpenFailure(undefined);
}
