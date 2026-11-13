import { CLASS_SEPARATOR } from "./consts";

/** Joins class names; names that are missing or `false` are left out. CSS modules give
 * `undefined` for a name they do not know, which then draws nothing instead of failing. */
export function cx(...names: (string | false | undefined)[]): string {
  return names.filter(Boolean).join(CLASS_SEPARATOR);
}
