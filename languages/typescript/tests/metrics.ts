export function rules(a: number, b: string[]): number {
  let total = 0;
  if (a > 1) {
    total += 1;
  } else if (a < 0) {
    total -= 1;
  } else {
    total = 0;
  }
  const label = a ? "yes" : "no";
  for (let i = 0; i < 3; i++) {
    total++;
  }
  for (const item of b) {
    total += item.length;
  }
  for (const key in b) {
    total += key.length;
  }
  while (total > 10) {
    total--;
  }
  do {
    total++;
  } while (total < 5);
  switch (a) {
    case 1:
    case 2:
      total += 2;
      break;
    case 3:
      total += 3;
      break;
    default:
      total = 0;
  }
  try {
    total = JSON.parse(label);
  } catch (error) {
    total = 0;
  } finally {
    total += 1;
  }
  const both = a > 1 && total > 2 || label === "x";
  const fallback = b[0] ?? "none";
  let maybe: string | undefined;
  maybe ??= "x";
  maybe ||= "y";
  maybe &&= "z";
  const optional = b?.length;
  const lambda = (x: number) => (x > 1 ? x : 0);
  return total + lambda(1);
}

export const arrow = (x: number): number => {
  const nested = () => x || 1;
  return nested();
};

class Counter {
  tick = () => {
    if (this.n) {
      return 1;
    }
    return 0;
  };
  n = 0;
}
