// Fixture for the rule of no literals in TSX (ADR 0058). The expected values are in
// crates/adapters/parsing/tests/typescript_metrics.rs.

export function view(): JSX.Element {
  return <p title="x">{8}</p>;
}

export function cast(value: unknown): number {
  return (5 as number) + (value as number) + 0;
}
