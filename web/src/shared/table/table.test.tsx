import { fireEvent, render, screen } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it } from "vitest";
import { createTableColumns, parseWidths } from "./columns";
import { ColumnWidths, ResizableTh, RestHead } from "./ResizableHead";

const IDS = ["one", "two", "three"] as const;

function Sample(props: { name: string }) {
  const columns = createTableColumns(props.name, IDS);
  return (
    <table data-testid="table" style={columns.style()}>
      <ColumnWidths columns={columns} />
      <thead>
        <tr>
          {IDS.map((id) => (
            <ResizableTh columns={columns} id={id} name={id}>
              {id}
            </ResizableTh>
          ))}
          <RestHead columns={columns} />
        </tr>
      </thead>
    </table>
  );
}

const grip = (name: string) =>
  screen.getByRole("separator", { name: `Change width of column ${name} (drag, or arrow keys)` });

describe("resizable table", () => {
  beforeEach(() => localStorage.clear());

  it("has a handle on every column, the last one included, and sizes itself at first", () => {
    render(() => <Sample name="first" />);
    expect(screen.getAllByRole("separator")).toHaveLength(IDS.length);
    expect(screen.getByTestId("table").style.tableLayout).toBe("");
  });

  it("fixes the widths of all columns at the first change and widens the one that moved", () => {
    render(() => <Sample name="second" />);
    fireEvent.keyDown(grip("three"), { key: "ArrowRight" });
    const table = screen.getByTestId("table");
    expect(table.style.tableLayout).toBe("fixed");
    expect(table.querySelectorAll("col")).toHaveLength(IDS.length + 1);
    const before = Number(grip("three").getAttribute("aria-valuenow"));
    fireEvent.keyDown(grip("three"), { key: "ArrowRight", shiftKey: true });
    expect(Number(grip("three").getAttribute("aria-valuenow"))).toBeGreaterThan(before);
    expect(Number(grip("one").getAttribute("aria-valuenow"))).toBeGreaterThan(0);
  });

  it("never goes below the minimum, remembers the widths and goes back on a double click", () => {
    const first = render(() => <Sample name="third" />);
    for (let press = 0; press < 20; press += 1) {
      fireEvent.keyDown(grip("two"), { key: "ArrowLeft" });
    }
    expect(Number(grip("two").getAttribute("aria-valuenow"))).toBe(48);
    first.unmount();
    render(() => <Sample name="third" />);
    expect(screen.getByTestId("table").style.tableLayout).toBe("fixed");
    fireEvent.dblClick(grip("two"));
    expect(screen.getByTestId("table").style.tableLayout).toBe("");
  });

  it("drops stored widths that are not widths", () => {
    expect(parseWidths('{"one":120,"two":"x","four":9}', IDS)).toEqual({ one: 120 });
    expect(parseWidths("not json", IDS)).toEqual({});
    expect(parseWidths(null, IDS)).toEqual({});
  });

  it("reports the width on focus while the table sizes itself, and keeps sizing itself", () => {
    const original = Object.getOwnPropertyDescriptor(HTMLElement.prototype, "offsetWidth");
    Object.defineProperty(HTMLElement.prototype, "offsetWidth", { configurable: true, value: 90 });
    try {
      render(() => <Sample name="fourth" />);
      const handle = grip("two");
      expect(handle.getAttribute("aria-valuenow")).toBeNull();
      fireEvent.focus(handle);
      expect(Number(handle.getAttribute("aria-valuenow"))).toBe(90);
      expect(screen.getByTestId("table").style.tableLayout).toBe("");
    } finally {
      if (original) {
        Object.defineProperty(HTMLElement.prototype, "offsetWidth", original);
      }
    }
  });

  it("takes all columns back to their first widths with Home", () => {
    render(() => <Sample name="fifth" />);
    fireEvent.keyDown(grip("one"), { key: "ArrowRight" });
    expect(screen.getByTestId("table").style.tableLayout).toBe("fixed");
    fireEvent.keyDown(grip("three"), { key: "Home" });
    expect(screen.getByTestId("table").style.tableLayout).toBe("");
  });

  it("starts from the width of now, not of the focus, and forgets the width after Home", () => {
    const original = Object.getOwnPropertyDescriptor(HTMLElement.prototype, "offsetWidth");
    const width = (value: number) =>
      Object.defineProperty(HTMLElement.prototype, "offsetWidth", { configurable: true, value });
    try {
      width(90);
      render(() => <Sample name="sixth" />);
      fireEvent.focus(grip("two"));
      width(140);
      fireEvent.keyDown(grip("two"), { key: "ArrowRight" });
      expect(Number(grip("two").getAttribute("aria-valuenow"))).toBe(152);
      fireEvent.keyDown(grip("two"), { key: "Home" });
      expect(grip("two").getAttribute("aria-valuenow")).toBeNull();
    } finally {
      if (original) {
        Object.defineProperty(HTMLElement.prototype, "offsetWidth", original);
      }
    }
  });
});
