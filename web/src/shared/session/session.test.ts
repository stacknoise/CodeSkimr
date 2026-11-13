import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const fetchMock = vi.fn();

async function freshSession(code: string | null) {
  vi.resetModules();
  vi.doMock("./capture", () => {
    let waiting = code;
    return {
      takeLaunchCode: () => {
        const taken = waiting;
        waiting = null;
        return taken;
      },
    };
  });
  const session = await import("./session");
  const token = await import("./token");
  return { ...session, ...token };
}

function answer(status: number, body: unknown) {
  return Promise.resolve(new Response(JSON.stringify(body), { status }));
}

beforeEach(() => {
  vi.stubGlobal("fetch", fetchMock);
  sessionStorage.clear();
  localStorage.clear();
});

afterEach(() => {
  fetchMock.mockReset();
  vi.unstubAllGlobals();
  vi.doUnmock("./capture");
});

describe("startSession", () => {
  it("trades the launch code for the token and keeps it in sessionStorage only", async () => {
    fetchMock.mockImplementation(() => answer(200, { token: "csk_secret" }));
    const { startSession, sessionState } = await freshSession("c0de");
    await startSession();
    expect(sessionState()).toEqual({ kind: "connected" });
    const [path, init] = fetchMock.mock.calls[0] ?? [];
    expect(path).toBe("/api/v1/session/exchange");
    expect(init.method).toBe("POST");
    expect(JSON.parse(init.body)).toEqual({ code: "c0de" });
    expect(new Headers(init.headers).has("Authorization")).toBe(false);
    expect(sessionStorage.getItem("codeskimr.token")).toBe("csk_secret");
    expect(JSON.stringify({ ...localStorage })).not.toContain("csk_secret");
  });

  it("says that the code was used when the server refuses it", async () => {
    fetchMock.mockImplementation(() => answer(401, { code: "API-403" }));
    const { startSession, sessionState } = await freshSession("old");
    await startSession();
    expect(sessionState()).toEqual({ kind: "reopen", reason: "code-invalid" });
    expect(sessionStorage.getItem("codeskimr.token")).toBeNull();
  });

  it("reports an unreachable server", async () => {
    fetchMock.mockRejectedValue(new TypeError("network"));
    const { startSession, sessionState } = await freshSession("c0de");
    await startSession();
    expect(sessionState()).toEqual({ kind: "reopen", reason: "unreachable" });
  });

  it("checks the token of the tab on a reload", async () => {
    fetchMock.mockImplementation(() => answer(200, { pending: false }));
    const { startSession, sessionState, storeToken } = await freshSession(null);
    storeToken("csk_tab");
    await startSession();
    expect(sessionState()).toEqual({ kind: "connected" });
    const [, init] = fetchMock.mock.calls[0] ?? [];
    expect(new Headers(init.headers).get("Authorization")).toBe("Bearer csk_tab");
  });

  it("asks to open CodeSkimr again when the token is refused or missing", async () => {
    fetchMock.mockImplementation(() => answer(401, { code: "API-401" }));
    const refused = await freshSession(null);
    refused.storeToken("csk_old");
    await refused.startSession();
    expect(refused.sessionState()).toEqual({ kind: "reopen", reason: "missing" });
    expect(sessionStorage.getItem("codeskimr.token")).toBeNull();

    const missing = await freshSession(null);
    await missing.startSession();
    expect(missing.sessionState()).toEqual({ kind: "reopen", reason: "missing" });
  });
});
