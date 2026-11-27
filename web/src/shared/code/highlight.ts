import { type Accessor, createEffect, createSignal, onCleanup } from "solid-js";
import { runCommand } from "../api/commands";
import {
  CACHE_LIMIT,
  EXTENSION_SEPARATOR,
  HIGHLIGHT_BATCH,
  HIGHLIGHT_COMMAND,
  KEY_SEPARATOR,
} from "./consts";
import type { CodeToken, HighlightAnswer } from "./types";

/** The runs of every piece that was coloured before; a piece that could not be coloured is in it
 * with no runs, so it is not asked for again. */
const cache = new Map<string, CodeToken[]>();

/** Keeps an answer; the oldest ones go when the cache is full. A failed call is never kept, so the
 * next time the piece is shown it is asked for again. */
function remember(key: string, tokens: CodeToken[]): void {
  cache.delete(key);
  cache.set(key, tokens);
  if (cache.size > CACHE_LIMIT) {
    const oldest = cache.keys().next();
    if (oldest.done !== true) {
      cache.delete(oldest.value);
    }
  }
}

interface Waiting {
  path: string;
  text: string;
  listeners: ((tokens: CodeToken[]) => void)[];
}

let waiting = new Map<string, Waiting>();
let scheduled = false;

/** The extension of a path, which tells the language; the key of a piece does not need more. */
function extensionOf(path: string): string {
  return path.slice(path.lastIndexOf(EXTENSION_SEPARATOR) + 1);
}

function keyOf(path: string, text: string): string {
  return `${extensionOf(path)}${KEY_SEPARATOR}${text}`;
}

/** Asks for the waiting pieces in batches. A failure (no server, a static export) leaves the code
 * plain: the colours are never a reason for an error on the screen. */
async function flush(): Promise<void> {
  scheduled = false;
  const batch = [...waiting.entries()];
  waiting = new Map();
  for (let from = 0; from < batch.length; from += HIGHLIGHT_BATCH) {
    const chunk = batch.slice(from, from + HIGHLIGHT_BATCH);
    let answer: HighlightAnswer | null = null;
    try {
      answer = await runCommand<HighlightAnswer>(HIGHLIGHT_COMMAND, {
        pieces: chunk.map(([, piece]) => ({ path: piece.path, text: piece.text })),
      });
    } catch {
      answer = null;
    }
    const pieces = Array.isArray(answer?.pieces) ? answer.pieces : null;
    chunk.forEach(([key, piece], index) => {
      const tokens = pieces?.[index]?.tokens ?? [];
      if (pieces !== null) {
        remember(key, tokens);
      }
      piece.listeners.forEach((listener) => listener(tokens));
    });
  }
}

function request(path: string, text: string, listener: (tokens: CodeToken[]) => void): void {
  const key = keyOf(path, text);
  const known = cache.get(key);
  if (known !== undefined) {
    listener(known);
    return;
  }
  const entry = waiting.get(key) ?? { path, text, listeners: [] };
  entry.listeners.push(listener);
  waiting.set(key, entry);
  if (!scheduled) {
    scheduled = true;
    queueMicrotask(() => void flush());
  }
}

/** The coloured runs of a piece of code; empty until the server has answered, and for a language
 * without colours. All pieces asked for in the same moment travel in one call. */
export function createHighlight(
  path: Accessor<string>,
  text: Accessor<string>,
): Accessor<CodeToken[]> {
  const [tokens, setTokens] = createSignal<CodeToken[]>([]);
  let shown: string | null = null;
  onCleanup(() => {
    shown = null;
  });
  createEffect(() => {
    const [now, code] = [path(), text()];
    const key = keyOf(now, code);
    shown = key;
    setTokens(cache.get(key) ?? []);
    // An answer that comes after the code changed or went away is dropped; it never reads the
    // props of a component that is gone.
    request(now, code, (found) => {
      if (shown === key) {
        setTokens(found);
      }
    });
  });
  return tokens;
}

/** How many pieces the cache holds. */
export const cacheSize = (): number => cache.size;

/** Forgets what was coloured (for tests). */
export function resetHighlightCache(): void {
  cache.clear();
  waiting = new Map();
}
