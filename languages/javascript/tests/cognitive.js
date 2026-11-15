// Fixture for the cognitive complexity and parameter counts of the JavaScript pack (ADR 0048); the
// queries are the TypeScript ones. The expected values are in
// crates/adapters/parsing/tests/typescript_metrics.rs.

export function nested(a, b) {
  if (a > 0) {
    for (let i = 0; i < b; i++) {
      if (a > b) {
        return 1;
      }
    }
  }
  return 0;
}

export function callbacks(items) {
  return items.map((x, i) => (x > i ? x : 0));
}

export const single = x => x;

export class Shop {
  field = (p, q) => p + q;
}
