import { createSignal } from "solid-js";
import { api, ApiError } from "../api/client";
import { EXCHANGE_PATH, METHOD_POST, SESSION_PATH } from "../api/consts";
import { takeLaunchCode } from "./capture";
import { KIND, REASON } from "./consts";
import { getToken, storeToken, whenInvalidated } from "./token";

/** Where the connection to the local server stands. */
export type SessionState =
  | { kind: typeof KIND.connecting }
  | { kind: typeof KIND.connected }
  /** The UI has no valid token. `reason` says why, so that the screen can tell the user. */
  | { kind: typeof KIND.reopen; reason: (typeof REASON)[keyof typeof REASON] };

const [state, setState] = createSignal<SessionState>({ kind: KIND.connecting });

/** The state of the connection. */
export const sessionState = state;

whenInvalidated(() => setState({ kind: KIND.reopen, reason: REASON.missing }));

interface Exchanged {
  token: string;
}

/** Trades the launch code for the token and keeps the token for this tab. */
async function exchange(code: string): Promise<void> {
  try {
    const answer = await api<Exchanged>(EXCHANGE_PATH, {
      method: METHOD_POST,
      body: { code },
      token: null,
    });
    storeToken(answer.token);
    setState({ kind: KIND.connected });
  } catch (error) {
    const refused = error instanceof ApiError && error.code !== null;
    setState({ kind: KIND.reopen, reason: refused ? REASON.codeInvalid : REASON.unreachable });
  }
}

/** Asks the server whether the token of this tab is still good. */
async function verify(): Promise<void> {
  try {
    await api(SESSION_PATH);
    setState({ kind: KIND.connected });
  } catch (error) {
    // A refused token has already ended the session through `invalidateToken`.
    if (!(error instanceof ApiError)) {
      setState({ kind: KIND.reopen, reason: REASON.unreachable });
    }
  }
}

/** Starts the session: a launch code in the address is exchanged, otherwise the token of this
 * tab, if there is one, is checked. Without either the UI asks to open CodeSkimr again. */
export async function startSession(): Promise<void> {
  const code = takeLaunchCode();
  if (code !== null) {
    await exchange(code);
  } else if (getToken() !== null) {
    await verify();
  } else {
    setState({ kind: KIND.reopen, reason: REASON.missing });
  }
}
