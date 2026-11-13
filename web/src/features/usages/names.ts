import { NAME_JOIN, NAME_PARTS, NAME_SEPARATORS } from "./consts";

/** The last parts of a qualified name: `Shop.Cart.AddItem` becomes `Cart.AddItem`, so a card shows
 * the type and the member. */
export function shortName(qualified: string): string {
  const parts = NAME_SEPARATORS.reduce(
    (names, separator) => names.flatMap((name) => name.split(separator)),
    [qualified],
  );
  return parts.slice(-NAME_PARTS).join(NAME_JOIN);
}
