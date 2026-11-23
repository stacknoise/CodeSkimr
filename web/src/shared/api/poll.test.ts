import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { pollUnanswered } from "./poll";

const EVERY = 1000;

describe("pollUnanswered", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("does not ask again while the last ask is unanswered", async () => {
    let answer: () => void = () => undefined;
    const ask = vi.fn(() => new Promise<void>((resolve) => (answer = resolve)));
    const stop = pollUnanswered(ask, EVERY);
    await vi.advanceTimersByTimeAsync(5 * EVERY);
    expect(ask).toHaveBeenCalledTimes(1);
    answer();
    await vi.advanceTimersByTimeAsync(EVERY);
    expect(ask).toHaveBeenCalledTimes(2);
    stop();
  });

  it("asks again after a failed ask and stops when told to", async () => {
    const ask = vi.fn(() => Promise.reject(new Error("gone")).catch(() => undefined));
    const stop = pollUnanswered(ask, EVERY);
    await vi.advanceTimersByTimeAsync(2 * EVERY);
    expect(ask).toHaveBeenCalledTimes(2);
    stop();
    await vi.advanceTimersByTimeAsync(2 * EVERY);
    expect(ask).toHaveBeenCalledTimes(2);
  });
});
