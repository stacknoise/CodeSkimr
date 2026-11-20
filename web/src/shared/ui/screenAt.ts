import type { Component } from "solid-js";

/** The screens of an app by the path that shows them. */
export type ScreenTable = ReadonlyArray<readonly [path: string, screen: Component]>;

/** The screen of `path`, or `fallback` for a path that no screen answers. */
export function screenAt(table: ScreenTable, path: string, fallback: Component): Component {
  return table.find(([known]) => known === path)?.[1] ?? fallback;
}
