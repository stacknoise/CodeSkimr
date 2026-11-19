import { createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { readSetting, writeSetting } from "../../shared/settings/storage";
import { CREDENTIAL_COMMAND, CREDENTIAL_NAMES_KEY } from "./consts";
import { failed, type Failure } from "./projects";

/** One credential that a Git project names, as the server lists it. */
export interface CredentialRow {
  id: string;
  stored: boolean;
}

const [listed, setListed] = createSignal<CredentialRow[]>([]);
/** The names this browser stored: the keystore cannot be listed, so the names are kept here. */
function storedNames(): string[] {
  try {
    const parsed: unknown = JSON.parse(readSetting(CREDENTIAL_NAMES_KEY) ?? "[]");
    return Array.isArray(parsed) ? parsed.filter((name) => typeof name === "string") : [];
  } catch {
    return [];
  }
}

const [kept, setKept] = createSignal<readonly string[]>(storedNames());

function keep(names: readonly string[]): void {
  setKept(names);
  writeSetting(CREDENTIAL_NAMES_KEY, JSON.stringify(names));
}

/** The names of the credentials that can be chosen: those a project names and that exist, and
 * those stored in this tab (a new credential is only listed once a project names it). */
export function credentialNames(): string[] {
  const names = listed()
    .filter((row) => row.stored)
    .map((row) => row.id);
  return [...new Set([...names, ...kept()])].sort();
}

/** Reads the names again. A failure keeps what the form has: the choice stays optional. */
export async function refreshCredentials(): Promise<void> {
  try {
    setListed(await runCommand<CredentialRow[]>(CREDENTIAL_COMMAND.list));
  } catch {
    // The login of the system's Git still works without a name.
  }
}

/** Stores a credential in the keystore of the server's machine. The secret travels in this one
 * request and is not kept here. Returns `null` if it worked, else the failure. */
export async function storeCredential(
  id: string,
  user: string,
  secret: string,
): Promise<Failure | null> {
  try {
    await runCommand(CREDENTIAL_COMMAND.set, { id, user, secret });
    keep([...new Set([...kept(), id])]);
    return null;
  } catch (error) {
    return failed(error);
  }
}

/** Removes a credential from the keystore of the server's machine and from the names offered.
 * Returns `null` if it worked, else the failure. */
export async function removeCredential(id: string): Promise<Failure | null> {
  try {
    await runCommand(CREDENTIAL_COMMAND.remove, { id });
    keep(kept().filter((name) => name !== id));
    setListed((rows) => rows.filter((row) => row.id !== id));
    return null;
  } catch (error) {
    return failed(error);
  }
}
