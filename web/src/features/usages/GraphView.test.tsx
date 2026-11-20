import { render, screen } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { ANSWER, HANDLER, VIEW } from "../../tests/usages";
import { GraphView } from "./GraphView";

const NO_CARD = 999;

function shown(picked: number | null, marked: number | null) {
  render(() => (
    <GraphView
      answer={ANSWER}
      picked={picked}
      marked={marked}
      onPick={() => {}}
      onOpen={() => {}}
    />
  ));
}

const card = (name: string) => screen.getByTitle(name);

describe("GraphView highlight", () => {
  it("dims nothing without a picked or marked symbol", () => {
    shown(null, null);
    expect(card(VIEW.qualified_name).className).not.toMatch(/dim/);
    expect(card(HANDLER.qualified_name).className).not.toMatch(/dim/);
  });

  it("lights the path of the marked symbol", () => {
    shown(null, HANDLER.id);
    expect(card(HANDLER.qualified_name).className).not.toMatch(/dim/);
    expect(card(VIEW.qualified_name).className).toMatch(/dim/);
  });

  it("prefers the picked card to the marked one", () => {
    shown(VIEW.id, HANDLER.id);
    expect(card(VIEW.qualified_name).className).not.toMatch(/dim/);
    expect(card(HANDLER.qualified_name).className).toMatch(/dim/);
  });

  it("dims nothing for a marked symbol that has no card", () => {
    shown(null, NO_CARD);
    expect(card(VIEW.qualified_name).className).not.toMatch(/dim/);
    expect(card(HANDLER.qualified_name).className).not.toMatch(/dim/);
  });
});
