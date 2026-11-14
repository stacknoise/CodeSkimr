import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { Breakable } from "./Breakable";

describe("Breakable", () => {
  it("keeps the text and offers a break after each separator", () => {
    const { container } = render(() => <Breakable text="reverse_index.ReverseIndex" />);
    expect(container.textContent).toBe("reverse_index.ReverseIndex");
    expect(container.querySelectorAll("wbr")).toHaveLength(3);
  });
});
