import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ApiError, api } from "./client";
import { getToken, storeToken } from "../session/token";

const fetchMock = vi.fn();

beforeEach(() => {
  vi.stubGlobal("fetch", fetchMock);
  sessionStorage.clear();
});

afterEach(() => {
  fetchMock.mockReset();
  vi.unstubAllGlobals();
});

describe("api", () => {
  it("sends the token as a bearer and nowhere else", async () => {
    fetchMock.mockResolvedValue(new Response("{}"));
    storeToken("csk_t");
    await api("/api/v1/commands");
    const [path, init] = fetchMock.mock.calls[0] ?? [];
    expect(path).toBe("/api/v1/commands");
    expect(new Headers(init.headers).get("Authorization")).toBe("Bearer csk_t");
    expect(path).not.toContain("csk_t");
  });

  it("sends no token when asked not to", async () => {
    fetchMock.mockResolvedValue(new Response("{}"));
    storeToken("csk_t");
    await api("/api/v1/ui-config", { token: null });
    const [, init] = fetchMock.mock.calls[0] ?? [];
    expect(new Headers(init.headers).has("Authorization")).toBe(false);
  });

  it("reports the registered code of a failed call", async () => {
    fetchMock.mockResolvedValue(new Response(JSON.stringify({ code: "API-402" }), { status: 403 }));
    await expect(api("/x")).rejects.toMatchObject({ status: 403, code: "API-402" });
  });

  it("does not trust a code that is not registered", async () => {
    fetchMock.mockResolvedValue(new Response(JSON.stringify({ code: "X-1" }), { status: 500 }));
    await expect(api("/x")).rejects.toMatchObject({ status: 500, code: null });
    fetchMock.mockResolvedValue(new Response("not json", { status: 502 }));
    await expect(api("/x")).rejects.toBeInstanceOf(ApiError);
  });

  it("drops a token that the server refuses", async () => {
    fetchMock.mockResolvedValue(new Response(JSON.stringify({ code: "API-401" }), { status: 401 }));
    storeToken("csk_old");
    await expect(api("/x", { method: "POST" })).rejects.toBeInstanceOf(ApiError);
    expect(getToken()).toBeNull();
  });
});
