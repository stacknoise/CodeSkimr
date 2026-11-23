import { POLL_PATIENCE_MS } from "./consts";

/** Asks every `every` milliseconds, but never while the last ask is still unanswered: requests
 * that a busy server does not answer would pile up and take the few connections of the browser
 * away from every other screen. An ask that gets no answer within `POLL_PATIENCE_MS` is given up
 * through its `signal`, so that the asking goes on. Returns the function that stops the asking;
 * an ask that is open then is left to its answer. */
export function pollUnanswered(
  ask: (signal: AbortSignal) => Promise<unknown>,
  every: number,
): () => void {
  let asking: AbortController | null = null;
  const timer = setInterval(() => {
    if (asking !== null) {
      return;
    }
    const mine = new AbortController();
    asking = mine;
    const patience = setTimeout(() => mine.abort(), POLL_PATIENCE_MS);
    // Inside a promise, so that an `ask` that throws at once ends like one that fails later.
    void Promise.resolve()
      .then(() => ask(mine.signal))
      .catch(() => undefined)
      .finally(() => {
        clearTimeout(patience);
        asking = null;
      });
  }, every);
  return () => clearInterval(timer);
}
