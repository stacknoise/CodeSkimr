// Fixture for the rule of no literals in JavaScript (ADR 0058). The expected values are in
// crates/adapters/parsing/tests/typescript_metrics.rs.

export function defaults(n = 5, { a = 6 } = {}) {
  return n + a;
}

export function constants() {
  const MAX = 99;
  const SUM = 2 * 50;
  return MAX + SUM + 1;
}

export function tagged() {
  return sql`select 7`;
}

export function view() {
  return <p title="x">{8}</p>;
}
