// A local function is a symbol: its uses are references. A local variable and a parameter are not.
export function outer(count: number, callback: () => void) {
  const double = (n: number) => n * 2;
  const total = count + 1;
  function twice(n: number) {
    return double(n) + double(n);
  }
  callback();
  return twice(total);
}
