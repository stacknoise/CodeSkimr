import { createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { CREDENTIAL_COMMAND } from "./consts";
import { failed, type Failure } from "./projects";

/** One credential that a Git project names, as the server lists it. */
export interface CredentialRow {
  id: string;
  stored: boolean;
}

const [listed, setListed] = createSignal<CredentialRow[]>([]);
const [kept, setKept] = createSignal<readonly string[]>([]);

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
    setKept((previous) => [...previous, id]);
    return null;
  } catch (error) {
    return failed(error);
  }
}
