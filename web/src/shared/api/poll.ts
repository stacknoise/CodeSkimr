/** Asks every `every` milliseconds, but never while the last ask is still unanswered: requests
 * that a busy server does not answer would pile up and take the few connections of the browser
 * away from every other screen. Returns the function that stops the asking. */
export function pollUnanswered(ask: () => Promise<unknown>, every: number): () => void {
  let asking = false;
  const timer = setInterval(() => {
    if (asking) {
      return;
    }
    asking = true;
    void ask().finally(() => {
      asking = false;
    });
  }, every);
  return () => clearInterval(timer);
}
