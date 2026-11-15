// Fixture for the cognitive complexity and parameter counts (ADR 0048). The expected values are in
// crates/adapters/parsing/tests/typescript_metrics.rs.

export function straight(a: number): number {
  return a + 1;
}

export function nested(a: number, b: number): number {
  if (a > 0) {
    for (let i = 0; i < b; i++) {
      if (a > b) {
        return 1;
      }
    }
  }
  return 0;
}

export function chain(a: number): number {
  if (a === 1) {
    return 1;
  } else if (a === 2) {
    return 2;
  } else {
    return 3;
  }
}

export function runs(a: boolean, b: boolean, c: boolean, d: boolean): boolean {
  return (a && b && c) || d;
}

export function handler(x: string): number {
  try {
    return Number(x);
  } catch (error) {
    if (x) {
      return 1;
    }
  }
  return 0;
}

export function callbacks(items: number[]): number[] {
  return items.map((x, i) => (x > i ? x : 0));
}

export const arrow = (x: number): number => {
  if (x) {
    return 1;
  }
  return 0;
};

export function branches(a: number): number {
  switch (a) {
    case 1:
      return 1;
    case 2:
      return 2;
    default:
      return 0;
  }
}

export function loops(a: number): number {
  while (a) {
    a--;
  }
  do {
    a++;
  } while (a < 3);
  outer: for (const x of [1, 2]) {
    for (const y of [3]) {
      if (x === y) {
        continue outer;
      }
    }
  }
  return a;
}

export function typed(cb: (a: number, b: number) => void): void {
  cb(1, 2);
}

export const single = (x: number) => x;
export const bare = async (x) => x;

export class Shop {
  field = (p: number, q: number): number => p + q;

  method(this: Shop, a: number, b?: string, c = 3, ...rest: number[]): number {
    return a + c + rest.length + (b ? 1 : 0);
  }
}
