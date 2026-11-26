import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { PREVIEW } from "../../tests/rename";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { closeRename, openRename, renameTarget } from "./rename";
import { RenameDialog } from "./RenameDialog";

useServerSession();
afterEach(closeRename);

const TARGET = {
  project: 1,
  symbol: 7,
  name: "PlaceOrder",
  qualified_name: "Shop.Orders.PlaceOrder",
  kind: "method",
};

function renderDialog() {
  openRename(TARGET);
  return render(() => <RenameDialog target={TARGET} />);
}

async function check(name: string) {
  fireEvent.input(screen.getByLabelText("New name"), { target: { value: name } });
  fireEvent.keyDown(screen.getByLabelText("New name"), { key: "Enter" });
}

describe("RenameDialog", () => {
  it("asks for a name first and changes nothing before it", () => {
    renderDialog();
    expect(screen.getByText("Type the new name and press Enter.")).toBeTruthy();
    expect(fetchMock).not.toHaveBeenCalled();
    expect((screen.getByText("Copy list") as HTMLButtonElement).disabled).toBe(true);
  });

  it("sends the symbol and the new name and shows places, banners and text hits", async () => {
    serve({ "rename.preview": () => answer(PREVIEW) });
    renderDialog();
    await check("SubmitOrder");
    expect(await screen.findByText("Api/Orders.cs")).toBeTruthy();
    const body = JSON.parse(fetchMock.mock.calls[0]?.[1].body as string);
    expect(body).toEqual({ project: 1, symbol: 7, new_name: "SubmitOrder" });
    expect(screen.getByText("docs/api.md")).toBeTruthy();
    expect(screen.getAllByText("await orders.PlaceOrder(cart);").length).toBeGreaterThan(0);
    expect(screen.getByText("Level 1: 1 place is preliminary")).toBeTruthy();
    expect(screen.getByText(/CodeSkimr does not see callers outside this project/)).toBeTruthy();
    expect(screen.getByText(/does not say whether the code compiles/)).toBeTruthy();
  });

  it("names a conflicting symbol with its signature", async () => {
    const conflict = {
      id: 9,
      qualified_name: "Shop.Orders.SubmitOrder",
      kind: "method",
      path: "Orders/IOrders.cs",
      signature: "Task SubmitOrder(Order order)",
      same_signature: true,
    };
    serve({ "rename.preview": () => answer({ ...PREVIEW, conflicts: [conflict] }) });
    renderDialog();
    await check("SubmitOrder");
    expect(
      await screen.findByText(/Shop.Orders.SubmitOrder \(Task SubmitOrder\(Order order\)\)/),
    ).toBeTruthy();
  });

  it("says that a refused name is no usable name", async () => {
    serve({ "rename.preview": () => answer({ code: "API-101" }, 400) });
    renderDialog();
    await check("two words");
    expect(await screen.findByText(/not a usable new name/)).toBeTruthy();
  });

  it("closes with Esc", async () => {
    renderDialog();
    fireEvent.keyDown(document, { key: "Escape" });
    await waitFor(() => expect(renameTarget()).toBeNull());
  });
});
