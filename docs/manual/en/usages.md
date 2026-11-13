# Usages

Usages answer two questions about a symbol: **who uses it** and **what does a change to it reach**. They are the base for planning and for judging the effect of a rework, so it matters how far you can trust them. Read the section on certainty below.

## Opening the screen

In the explorer, select a member (or a type) and press `U`, click "Usages", or click the usage count in the member table. The screen shows the usages of that symbol.

## Toolbar

- **Direction:** both, who calls (incoming), what is called (outgoing).
- **Depth:** 1, 2, 3 or all. "All" is cut at the configured maximum (`mcp.max_depth`, default 5).
- **Show tests:** references from test code are left out unless you switch this on.
- **Level:** only "Symbol" exists. "Namespaces" and "Types" are shown off; they belong to the [dependency map](dependency-map.md).

Every change reads the usages again.

## The graph

One column per step of callers on the left, the symbol in focus in the middle (yellow ring), one column per step of callees on the right. A card shows type and member name, kind, layer and the number of places. Click a card to make that symbol the focus. Lines are blue (calls), orange (what the focus uses), violet (tests) and dashed grey (presumed). At most 30 cards per column are drawn; the heading says how many more exist. A reference that closes a cycle is listed but not drawn backwards.

## The hit list

One row per place: kind (call, read, write, test and so on), the calling symbol, file and line, and the certainty. `F8` and `Shift+F8` step through the hits, `Enter` (or a double click) shows the selected hit in the explorer, or the symbol in focus if none is selected. The list shows file and line, not the code itself.

The side panel shows signature, counts (places, files, layers), the **impact** of a change (symbols reached, layers, public interfaces) and the button "Open in Explorer". Constants, properties and interface members work the same way as methods.

## Certainty: verified, unchecked, presumed

CodeSkimr finds usages in two ways. **Level 1** resolves names without type information: fast, needs nothing installed, but it can miss usages and can include wrong ones. **Level 2** takes the result of the language's own tool, which knows the types ([Language indexers](indexers.md)). Every hit says which one produced it:

| Pill | Meaning |
|---|---|
| **verified** | level 2 produced it; it is as reliable as the language tool |
| **unchecked** | found by name only; **not** checked by a compiler or indexer, may be wrong |
| **presumed** | level 1 itself is unsure (several symbols fit, or the call goes through an interface); an unchecked hit of lower value |

Verified hits come first. The side panel counts the three apart ("N places verified, M unchecked, K of them presumed").

**Without an indexer for the project, every hit is unchecked**, and the screen says so: the usages and the impact are preliminary and may be incomplete or contain wrong hits. Take this seriously for decisions: a missing usage can mean you break code you did not know about. An empty list without an indexer does not rule out usages.

## Same knowledge elsewhere

MCP offers the same data to code agents with the same distinction ([MCP setup](mcp.md)). The usages walk follows these rules: a reference is listed once, a symbol reached before is not walked again (so cycles end), references end at the symbol that contains the occurrence (the usages of a type do not include those of its members; ask for the members separately), and references from tests count only when asked, except in the impact, which always includes them.
