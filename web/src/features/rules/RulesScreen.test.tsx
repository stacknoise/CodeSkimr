import { fireEvent, render, screen } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import { openProject } from "../../shared/project/active";
import { answer, fetchMock, useServerSession } from "../../tests/commands";
import { resetRules } from "./rules";
import { RulesScreen } from "./RulesScreen";
import type { RulesOut, ViolationOut } from "./types";

useServerSession();

beforeEach(() => {
  resetRules();
  openProject(1);
  setLanguage("en");
});

function violation(index: number, old: boolean): ViolationOut {
  return {
    from: `Shop.Reports.Job${index}.Run`,
    to: "Shop.Checkout.Cart.Total",
    old,
    places: [
      { path: `Reports/Job${index}.cs`, line: 18, column: 5 },
      { path: `Reports/Job${index}.cs`, line: 31, column: 9 },
      { path: `Reports/Job${index}.cs`, line: 40, column: 9 },
    ],
  };
}

const ANSWER: RulesOut = {
  scanned: true,
  holdable: true,
  cycles: false,
  held_on: "2026-11-26",
  resolved: 1,
  old: 4,
  new: 1,
  rules: [
    {
      id: "layers",
      kind: "layers",
      from: "",
      to: "",
      internal: [],
      reason: null,
      allowed: [
        { layer: "Api", may_use: ["Domain"] },
        { layer: "Data", may_use: [] },
      ],
      new: 0,
      violations: [],
    },
    {
      id: "forbid:Shop.Reports>Shop.Checkout",
      kind: "forbid",
      from: "Shop.Reports",
      to: "Shop.Checkout",
      internal: [],
      reason: null,
      allowed: [],
      new: 1,
      violations: [violation(1, false), violation(2, true), violation(3, true), violation(4, true)],
    },
    {
      id: "public_api:Shop.Data",
      kind: "public_api",
      from: "Shop.Data",
      to: "",
      internal: ["Shop.Data.Internal"],
      reason: null,
      allowed: [],
      new: 0,
      violations: [],
    },
  ],
};

function serve(data: Partial<RulesOut> = {}) {
  fetchMock.mockImplementation(() => Promise.resolve(answer({ ...ANSWER, ...data })));
}

describe("RulesScreen", () => {
  it("shows the numbers, the broken rule with its first violations and the kept rule", async () => {
    serve();
    render(() => <RulesScreen />);
    expect(
      await screen.findByText("Ban: Shop.Reports must not use Shop.Checkout"),
    ).toBeInTheDocument();
    expect(screen.getByText("1 fail the CI")).toBeInTheDocument();
    expect(screen.getByText("4 held on 2026-11-26")).toBeInTheDocument();
    expect(screen.getByText("1 resolved")).toBeInTheDocument();
    expect(screen.getByText("Shop.Reports.Job1.Run")).toBeInTheDocument();
    expect(screen.queryByText("Shop.Reports.Job4.Run")).not.toBeInTheDocument();
    expect(screen.getAllByText("Reports/Job1.cs:18, Reports/Job1.cs:31, …")).toHaveLength(1);
    expect(screen.getByText("1 new")).toBeInTheDocument();
    expect(screen.getByText("3 old burdens")).toBeInTheDocument();
    expect(screen.getByText("Only through the public interface")).toBeInTheDocument();
    expect(screen.getAllByText("kept")).toHaveLength(2);
    expect(screen.getByText("Shop.Data.Internal.* only within Shop.Data")).toBeInTheDocument();
  });

  it("shows the layer rule first, says what each layer may use and gives it no new-kind badge", async () => {
    serve();
    render(() => <RulesScreen />);
    expect(await screen.findByText("Layers")).toBeInTheDocument();
    expect(screen.getByText("Api → Domain; Data → nothing")).toBeInTheDocument();
    expect(screen.getAllByText("new rule kind")).toHaveLength(2);
  });

  it("holds the state with the button, shows what was written and reads the rules again", async () => {
    let held = false;
    fetchMock.mockImplementation((path: string) => {
      if (path.endsWith("rules.hold")) {
        held = true;
        return Promise.resolve(answer({ held_on: "2026-11-26", entries: 5 }));
      }
      return Promise.resolve(answer(held ? { ...ANSWER, new: 0, old: 5 } : ANSWER));
    });
    render(() => <RulesScreen />);
    expect(await screen.findByText("1 fail the CI")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Hold state" }));
    expect(await screen.findByText(/State held on 2026-11-26: 5 violations/)).toBeInTheDocument();
    expect(await screen.findByText("0 fail the CI")).toBeInTheDocument();
  });

  it("switches the button off for a Git project and for a project without a scan", async () => {
    serve({ holdable: false });
    const first = render(() => <RulesScreen />);
    await screen.findByText("Layers");
    const button = screen.getByRole("button", { name: "Hold state" });
    expect(button).toBeDisabled();
    expect(button.getAttribute("title")).toMatch(/Folder projects only/);
    first.unmount();
    resetRules();
    serve({ scanned: false, rules: [] });
    render(() => <RulesScreen />);
    await screen.findByText(/has not been scanned yet/);
    expect(screen.getByRole("button", { name: "Hold state" })).toBeDisabled();
  });

  it("shows the failure of holding the state", async () => {
    fetchMock.mockImplementation((path: string) =>
      Promise.resolve(
        path.endsWith("rules.hold") ? answer({ code: "CFG-104" }, 500) : answer(ANSWER),
      ),
    );
    render(() => <RulesScreen />);
    await screen.findByText("Layers");
    fireEvent.click(screen.getByRole("button", { name: "Hold state" }));
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });

  it("folds the rest of the violations away and shows them on request", async () => {
    serve();
    render(() => <RulesScreen />);
    fireEvent.click(await screen.findByRole("button", { name: "… 1 more · Show all" }));
    expect(screen.getByText("Shop.Reports.Job4.Run")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Show fewer" }));
    expect(screen.queryByText("Shop.Reports.Job4.Run")).not.toBeInTheDocument();
  });

  it("says so when no state was held and when nothing was scanned or written", async () => {
    serve({ held_on: null, old: 0, resolved: 0 });
    render(() => <RulesScreen />);
    expect(await screen.findByText("No state held")).toBeInTheDocument();
  });

  it("explains an unscanned project and a project file without rules", async () => {
    serve({ scanned: false, rules: [] });
    const first = render(() => <RulesScreen />);
    expect(await screen.findByText(/has not been scanned yet/)).toBeInTheDocument();
    first.unmount();
    resetRules();
    serve({ rules: [] });
    render(() => <RulesScreen />);
    expect(await screen.findByText(/has no rules yet/)).toBeInTheDocument();
  });

  it("tells that the cycle rule is on but not checked yet", async () => {
    serve({ cycles: true });
    render(() => <RulesScreen />);
    expect(await screen.findByText(/Cycles between modules are switched on/)).toBeInTheDocument();
  });

  it("reads in German too", async () => {
    setLanguage("de");
    serve();
    render(() => <RulesScreen />);
    expect(
      await screen.findByText("Verbot: Shop.Reports darf Shop.Checkout nicht nutzen"),
    ).toBeInTheDocument();
    expect(screen.getByText("3 Altlasten")).toBeInTheDocument();
  });
});
