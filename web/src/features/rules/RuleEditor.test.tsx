import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it } from "vitest";
import { answer, fetchMock, useServerSession } from "../../tests/commands";
import { openProject } from "../../shared/project/active";
import { setLanguage } from "../../shared/i18n/i18n";
import { closeDialog } from "./editor";
import { resetRules } from "./rules";
import { RulesScreen } from "./RulesScreen";
import type { RuleOut, RulesOut } from "./types";

useServerSession();

beforeEach(() => {
  resetRules();
  closeDialog();
  openProject(1);
  setLanguage("en");
});

const RULE: RuleOut = {
  id: "forbid:Shop.Reports>Shop.Checkout",
  kind: "forbid",
  from: "Shop.Reports",
  to: "Shop.Checkout",
  internal: [],
  reason: "reports read only",
  allowed: [],
  new: 0,
  violations: [],
};

const LAYERS: RuleOut = {
  ...RULE,
  id: "layers",
  kind: "layers",
  from: "",
  to: "",
  reason: null,
  allowed: [{ layer: "Api", may_use: ["Domain"] }],
};

const ANSWER: RulesOut = {
  scanned: true,
  holdable: true,
  cycles: false,
  held_on: null,
  resolved: 0,
  old: 0,
  new: 0,
  rules: [LAYERS, RULE],
};

/** Shows the screen and picks an entry of the menu "Add rule". */
async function pickFromMenu(item: RegExp) {
  render(() => <RulesScreen />);
  await screen.findByText("Api → Domain");
  fireEvent.click(screen.getByRole("button", { name: "Add rule" }));
  fireEvent.click(screen.getByRole("menuitem", { name: item }));
}

/** The bodies of the commands the screen sent, by command id. */
function serveAll(data: Partial<RulesOut> = {}) {
  const sent: Record<string, unknown[]> = {};
  fetchMock.mockImplementation((path: string, init?: { body?: string }) => {
    const id = path.replace("/api/v1/commands/", "");
    (sent[id] ??= []).push(init?.body === undefined ? null : JSON.parse(init.body));
    if (id === "layers.get") {
      return Promise.resolve(
        answer({
          layers: [{ name: "Api" }, { name: "Domain" }],
          namespaces: [{ name: "Shop.Reports" }, { name: "Shop.Checkout" }],
        }),
      );
    }
    if (id === "rules.set") {
      return Promise.resolve(answer({ rules: 1 }));
    }
    return Promise.resolve(answer({ ...ANSWER, ...data }));
  });
  return sent;
}

describe("the rule editor", () => {
  it("adds a ban through the menu and the dialog and writes the whole rule set", async () => {
    const sent = serveAll();
    await pickFromMenu(/Ban/);
    const dialog = await screen.findByRole("dialog");
    expect(dialog).toHaveTextContent("Add ban");
    fireEvent.input(screen.getByLabelText(/This module/), { target: { value: "Shop.Orders" } });
    fireEvent.input(screen.getByLabelText(/must not use this module/), {
      target: { value: "Shop.Api" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(sent["rules.set"]).toHaveLength(1));
    expect(sent["rules.set"]?.[0]).toMatchObject({
      project: 1,
      cycles: false,
      forbid: [
        { from: "Shop.Reports", to: "Shop.Checkout", reason: "reports read only" },
        { from: "Shop.Orders", to: "Shop.Api" },
      ],
    });
    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
  });

  it("deletes a ban only after the dialog asks", async () => {
    const sent = serveAll();
    render(() => <RulesScreen />);
    await screen.findByText("Api → Domain");
    fireEvent.click(screen.getByRole("button", { name: `Delete rule ${RULE.id}` }));
    expect(await screen.findByRole("dialog")).toHaveTextContent("Delete rule?");
    expect(sent["rules.set"]).toBeUndefined();
    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    await waitFor(() => expect(sent["rules.set"]?.[0]).toMatchObject({ forbid: [] }));
  });

  it("sets what a layer may use with the matrix", async () => {
    const sent = serveAll();
    render(() => <RulesScreen />);
    await screen.findByText("Api → Domain");
    fireEvent.click(screen.getByRole("button", { name: "Change rule layers" }));
    expect(await screen.findByRole("dialog")).toHaveTextContent("who may use whom");
    fireEvent.click(screen.getByLabelText("Domain Limit"));
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(sent["rules.set"]).toHaveLength(1));
    expect(sent["rules.set"]?.[0]).toMatchObject({
      limits: [
        { layer: "Api", may_use: ["Domain"] },
        { layer: "Domain", may_use: [] },
      ],
    });
  });

  it("switches the cycle rule on with the menu and closes the dialog with Esc", async () => {
    const sent = serveAll();
    await pickFromMenu(/Forbid cycles/);
    await waitFor(() => expect(sent["rules.set"]?.[0]).toMatchObject({ cycles: true }));
    fireEvent.click(screen.getByRole("button", { name: `Change rule ${RULE.id}` }));
    await screen.findByRole("dialog");
    fireEvent.keyDown(document, { key: "Escape" });
    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
  });

  it("offers no editing for a Git project", async () => {
    serveAll({ holdable: false });
    render(() => <RulesScreen />);
    await screen.findByText("Api → Domain");
    expect(screen.getByRole("button", { name: "Add rule" })).toBeDisabled();
    expect(screen.queryByRole("button", { name: /Change rule/ })).not.toBeInTheDocument();
  });
});

describe("the tab Layers", () => {
  it("shows the layer editor in a second tab", async () => {
    serveAll();
    render(() => <RulesScreen />);
    await screen.findByText("Api → Domain");
    fireEvent.click(screen.getByRole("tab", { name: "Layers" }));
    expect(screen.getByRole("tab", { name: "Layers", selected: true })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Add rule" })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("tab", { name: "Rules" }));
    expect(await screen.findByRole("button", { name: "Add rule" })).toBeInTheDocument();
  });
});
