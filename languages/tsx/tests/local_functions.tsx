// A local function component is a symbol: its uses are references. A local variable is not.
export function Panel(props: { count: number }) {
  const shut = () => props.count === 0;
  const total = props.count + 1;
  return shut() ? null : <b>{total}</b>;
}
