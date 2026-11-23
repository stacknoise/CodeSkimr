import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { POLL_PATIENCE_MS } from "./consts";
import { pollUnanswered } from "./poll";

const EVERY = 1000;

/** An ask that is answered only when its signal gives it up. */
function unanswered(signal: AbortSignal): Promise<void> {
  return new Promise((_resolve, reject) => {
    signal.addEventListener("abort", () => reject(new Error("given up")));
  });
}

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

  it("asks again after an ask that failed or threw at once", async () => {
    const ask = vi
      .fn<(signal: AbortSignal) => Promise<void>>()
      .mockRejectedValueOnce(new Error("gone"))
      .mockImplementationOnce(() => {
        throw new Error("at once");
      })
      .mockResolvedValue();
    const stop = pollUnanswered(ask, EVERY);
    await vi.advanceTimersByTimeAsync(3 * EVERY);
    expect(ask).toHaveBeenCalledTimes(3);
    stop();
  });

  it("gives up an ask that is never answered and asks again", async () => {
    const ask = vi.fn(unanswered);
    const stop = pollUnanswered(ask, EVERY);
    await vi.advanceTimersByTimeAsync(EVERY + POLL_PATIENCE_MS);
    expect(ask.mock.calls[0]?.[0].aborted).toBe(true);
    await vi.advanceTimersByTimeAsync(EVERY);
    expect(ask).toHaveBeenCalledTimes(2);
    stop();
  });

  it("stops asking when told to", async () => {
    const ask = vi.fn(() => Promise.resolve());
    const stop = pollUnanswered(ask, EVERY);
    await vi.advanceTimersByTimeAsync(EVERY);
    stop();
    await vi.advanceTimersByTimeAsync(3 * EVERY);
    expect(ask).toHaveBeenCalledTimes(1);
  });
});
