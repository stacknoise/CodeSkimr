import { NAME_JOIN, NAME_PARTS, NAME_SEPARATORS } from "./consts";

/** The parts of a qualified name, split at `.` and `::`. */
function partsOf(qualified: string): string[] {
  return NAME_SEPARATORS.reduce(
    (names, separator) => names.flatMap((name) => name.split(separator)),
    [qualified],
  );
}

/** What a card writes: the name of the symbol itself (`AddItem`), and the one it belongs to
 * (`Cart`), empty if the name has only one part. */
export function cardNames(qualified: string): { member: string; owner: string } {
  const parts = partsOf(qualified);
  return {
    member: parts[parts.length - 1] ?? qualified,
    owner: parts[parts.length - NAME_PARTS] ?? "",
  };
}

/** The last parts of a qualified name: `Shop.Cart.AddItem` becomes `Cart.AddItem`, so a card shows
 * the type and the member. */
export function shortName(qualified: string): string {
  return partsOf(qualified).slice(-NAME_PARTS).join(NAME_JOIN);
}
