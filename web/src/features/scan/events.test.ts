import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { listenToScans, parseEvent, resetRuns, scanRuns } from "./events";

class FakeSocket {
  static all: FakeSocket[] = [];
  onmessage: ((message: { data: unknown }) => void) | null = null;
  onclose: (() => void) | null = null;
  closed = false;
  constructor(readonly url: string) {
    FakeSocket.all.push(this);
  }
  close() {
    this.closed = true;
  }
}

beforeEach(() => {
  FakeSocket.all = [];
  vi.stubGlobal("WebSocket", FakeSocket);
  vi.stubGlobal(
    "fetch",
    vi.fn(() => Promise.resolve(new Response("[]"))),
  );
  vi.useFakeTimers();
  resetRuns();
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe("parseEvent", () => {
  it("accepts the events of the server and ignores everything else", () => {
    expect(
      parseEvent('{"type":"scan_progressed","project":1,"stage":"analyze","done":1,"total":2}'),
    ).not.toBeNull();
    expect(parseEvent('{"type":"snapshot_created","project":1,"snapshot":2}')).not.toBeNull();
    expect(
      parseEvent('{"type":"scan_progressed","project":1,"stage":"warp","done":1,"total":2}'),
    ).toBeNull();
    expect(parseEvent('{"type":"something_new","project":1}')).toBeNull();
    expect(parseEvent('{"type":"scan_stopped"}')).toBeNull();
    expect(parseEvent("not json")).toBeNull();
    expect(parseEvent("null")).toBeNull();
  });
});

describe("listenToScans", () => {
  it("opens the stream of this page and turns messages into runs", () => {
    const stop = listenToScans();
    const [socket] = FakeSocket.all;
    expect(socket?.url).toBe("ws://localhost:3000/api/v1/events");
    socket?.onmessage?.({
      data: '{"type":"scan_progressed","project":1,"stage":"analyze","done":3,"total":9}',
    });
    expect(scanRuns()[0]?.stages.analyze?.done).toBe(3);
    stop();
  });

  it("opens the stream again after it was lost, and not after the listener ended", () => {
    const stop = listenToScans();
    FakeSocket.all[0]?.onclose?.();
    vi.advanceTimersByTime(2000);
    expect(FakeSocket.all).toHaveLength(2);
    stop();
    FakeSocket.all[1]?.onclose?.();
    vi.advanceTimersByTime(10_000);
    expect(FakeSocket.all).toHaveLength(2);
    expect(FakeSocket.all[1]?.closed).toBe(true);
  });
});
