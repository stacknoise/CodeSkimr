import { describe, expect, it } from "vitest";
import { expandOutline, type WireOutline } from "./outlineWire";

const THRESHOLDS = {
  complexity_good_max: 5,
  complexity_medium_max: 10,
  maintainability_good_min: 80,
  maintainability_medium_min: 60,
  instability_stable_max: 0.3,
  instability_medium_max: 0.7,
};

function wire(name: string, types: object[], language?: string): WireOutline {
  return {
    snapshot: 1,
    thresholds: THRESHOLDS,
    layers: [{ name: "", namespaces: [{ name, types, language } as never] }],
  };
}

describe("expandOutline", () => {
  it("puts back the language and the qualified name that the answer left out", () => {
    const outline = expandOutline(
      wire("App.Domain", [{ id: 1, name: "Order", kind: "class", members: 2 }], "csharp"),
    );
    expect(outline.layers[0]?.namespaces[0]?.types[0]).toEqual({
      id: 1,
      name: "Order",
      kind: "class",
      members: 2,
      qualified_name: "App.Domain.Order",
      language: "csharp",
    });
  });

  it("keeps a qualified name and a language that the answer names", () => {
    const outline = expandOutline(
      wire(
        "app::domain",
        [
          {
            id: 2,
            name: "Order",
            kind: "struct",
            members: 0,
            qualified_name: "app::domain::Order",
            language: "rust",
          },
        ],
        "typescript",
      ),
    );
    const type = outline.layers[0]?.namespaces[0]?.types[0];
    expect(type?.qualified_name).toBe("app::domain::Order");
    expect(type?.language).toBe("rust");
  });

  it("takes the name alone for a type without a namespace", () => {
    const outline = expandOutline(
      wire("", [{ id: 3, name: "Main", kind: "class", members: 1 }], "java"),
    );
    expect(outline.layers[0]?.namespaces[0]?.types[0]?.qualified_name).toBe("Main");
  });

  it("puts back the module, the holders and the flag of a namespace", () => {
    const outline = expandOutline({
      snapshot: 1,
      thresholds: THRESHOLDS,
      layers: [
        {
          name: "",
          namespaces: [
            {
              name: "pkg.util",
              module: { id: 4, name: "util", kind: "module", members: 2 },
              language: "python",
            },
          ],
        },
      ],
    });
    const space = outline.layers[0]?.namespaces[0];
    expect(space?.types).toEqual([]);
    expect(space?.holders).toEqual([]);
    expect(space?.file).toBe(false);
    expect(space?.module?.qualified_name).toBe("pkg.util");
    expect(space?.module?.language).toBe("python");
  });
});
