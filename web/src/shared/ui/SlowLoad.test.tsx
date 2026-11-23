import { render } from "@solidjs/testing-library";
import { createSignal } from "solid-js";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { SLOW_LOAD_DELAY_MS } from "./consts";
import { SlowLoad } from "./SlowLoad";

const NOTE = "p";

describe("SlowLoad", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("says nothing while the answer is quick", () => {
    const view = render(() => <SlowLoad waiting={() => true} />);
    vi.advanceTimersByTime(SLOW_LOAD_DELAY_MS - 1);
    expect(view.container.querySelector(NOTE)).toBeNull();
  });

  it("says that it is still loading after the delay and stops when the answer is there", () => {
    const [waiting, setWaiting] = createSignal(true);
    const view = render(() => <SlowLoad waiting={waiting} />);
    vi.advanceTimersByTime(SLOW_LOAD_DELAY_MS);
    expect(view.container.querySelector(NOTE)).not.toBeNull();
    setWaiting(false);
    expect(view.container.querySelector(NOTE)).toBeNull();
  });
});
