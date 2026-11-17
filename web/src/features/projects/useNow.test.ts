import { createRoot } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import { NOW_TICK_MS } from "./consts";
import { useNow } from "./useNow";

describe("useNow", () => {
  afterEach(() => vi.useRealTimers());

  it("moves on every tick and stops after the owner is gone", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date(0));
    const { dispose, now } = createRoot((stop) => ({ dispose: stop, now: useNow() }));
    expect(now()).toBe(0);
    vi.advanceTimersByTime(NOW_TICK_MS);
    expect(now()).toBe(NOW_TICK_MS);
    dispose();
    vi.advanceTimersByTime(NOW_TICK_MS);
    expect(now()).toBe(NOW_TICK_MS);
  });
});
