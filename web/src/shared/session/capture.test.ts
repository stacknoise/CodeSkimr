import { describe, expect, it, vi } from "vitest";
import { captureLaunchCode } from "./capture";

function page(hash: string, pathname = "/settings", search = "?a=1") {
  const replaceState = vi.fn();
  return { location: { hash, pathname, search }, history: { replaceState }, replaceState };
}

describe("captureLaunchCode", () => {
  it("takes the code out of the fragment and removes the fragment from the address", () => {
    const { location, history, replaceState } = page("#code=0123abcd");
    expect(captureLaunchCode(location, history)).toBe("0123abcd");
    expect(replaceState).toHaveBeenCalledTimes(1);
    expect(replaceState.mock.calls[0]?.[2]).toBe("/settings?a=1");
  });

  it("removes any other fragment too, so that nothing stays in the history", () => {
    const { location, history, replaceState } = page("#other=1");
    expect(captureLaunchCode(location, history)).toBeNull();
    expect(replaceState).toHaveBeenCalledTimes(1);
  });

  it("leaves an address without a fragment alone", () => {
    const { location, history, replaceState } = page("");
    expect(captureLaunchCode(location, history)).toBeNull();
    expect(replaceState).not.toHaveBeenCalled();
  });
});
