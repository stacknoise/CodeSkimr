import { afterEach } from "vitest";

/** Lets a test put its own clipboard on `navigator` (or none, with `undefined`) and gives the
 * original one back after each test. Call it once inside a `describe`. */
export function useClipboard(): (value: unknown) => void {
  const original = Object.getOwnPropertyDescriptor(navigator, "clipboard");
  afterEach(() => {
    if (original) {
      Object.defineProperty(navigator, "clipboard", original);
    } else {
      Reflect.deleteProperty(navigator, "clipboard");
    }
  });
  return (value) => Object.defineProperty(navigator, "clipboard", { value, configurable: true });
}
