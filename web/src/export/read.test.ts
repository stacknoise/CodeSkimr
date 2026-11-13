import { describe, expect, it } from "vitest";
import { DATA_ELEMENT_ID, DATA_FORMAT } from "./consts";
import { readData } from "./read";

function pageWith(text: string | null): Document {
  const page = document.implementation.createHTMLDocument();
  if (text !== null) {
    const block = page.createElement("script");
    block.id = DATA_ELEMENT_ID;
    block.textContent = text;
    page.body.append(block);
  }
  return page;
}

describe("the data block", () => {
  it("is read when it is JSON of the known format", () => {
    const data = readData(pageWith(JSON.stringify({ format: DATA_FORMAT })));
    expect(data?.format).toBe(DATA_FORMAT);
  });

  it("reads the escaped angle bracket back as the character", () => {
    const data = readData(pageWith('{"format":1,"name":"\\u003c/script>"}'));
    expect((data as unknown as { name: string }).name).toBe("</script>");
  });

  it("is refused when it is missing, damaged or of another format", () => {
    expect(readData(pageWith(null))).toBeNull();
    expect(readData(pageWith("{nope"))).toBeNull();
    expect(readData(pageWith(JSON.stringify({ format: DATA_FORMAT + 1 })))).toBeNull();
    expect(readData(pageWith("__CODESKIMR_EXPORT_DATA__"))).toBeNull();
  });
});
