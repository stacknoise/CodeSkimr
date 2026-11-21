// Fixture for the rule of no literals (ADR 0058). The expected values are in
// crates/adapters/parsing/tests/typescript_metrics.rs.

export function clean(items: number[]): number {
  let total = 0;
  for (const item of items) {
    total += 1;
  }
  return total + items[0] + item.length;
}

export function numbers(): number {
  return 42 + 0 + 1 + 2.5;
}

export function strings(name: string): string {
  const label = "x" + 'y';
  return `${name}!` + label + `plain`;
}

export function types(kind: "a" | "b", count: 3 = 3): "a" | "b" {
  return kind;
}

export function flags(value?: object): boolean {
  const table = { "key": true, other: null };
  return value === undefined || table.other === null || false;
}

function register(items: string[]): void {
  items.push("added");
  "use strict";
}

export class Holder {
  static LIMIT = 99;

  method(): number[] {
    return [1, 2, 3];
  }
}
