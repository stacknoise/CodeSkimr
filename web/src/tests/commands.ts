import { afterEach, beforeEach, vi } from "vitest";
import { setLanguage } from "../shared/i18n/i18n";
import { storeToken } from "../shared/session/token";

/** The fetch of a test: every call of the UI to the server goes through it. */
export const fetchMock = vi.fn();

/** A JSON answer of the server. */
export function answer(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), { status });
}

/** Answers the command calls of a screen by command id; an unknown command is a 404. */
export function serve(handlers: Record<string, () => Response>) {
  fetchMock.mockImplementation((path: string) => {
    const id = path.replace("/api/v1/commands/", "");
    return Promise.resolve(handlers[id]?.() ?? answer({ code: "API-201" }, 404));
  });
}

/** Gives every test of the file a signed-in session in English and a fresh fetch. */
export function useServerSession(): void {
  beforeEach(() => {
    vi.stubGlobal("fetch", fetchMock);
    sessionStorage.clear();
    storeToken("csk_t");
    setLanguage("en");
  });
  afterEach(() => {
    fetchMock.mockReset();
    vi.unstubAllGlobals();
  });
}
