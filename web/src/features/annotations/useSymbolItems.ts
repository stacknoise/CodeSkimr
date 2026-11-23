import { createResource } from "solid-js";
import { ApiError } from "../../shared/api/client";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { readSymbol } from "./api";
import { KIND, TAB } from "./consts";
import { targetKey } from "./join";
import { activeTab } from "./panelState";
import { annotationChanges } from "./store";
import type { Annotation, Target } from "./types";

/** The annotations of one symbol, as they were read. */
interface Read {
  /** The symbol they belong to, so that an answer for another one is never shown. */
  key: string;
  items: Annotation[];
}

/** The annotations of the shown symbol, read when a tab other than Info is shown and again after
 * every change. The counters in the titles of the tabs come from the summary and need no call. */
export function useSymbolItems(project: () => number, target: () => Target) {
  const [read] = createResource(
    () =>
      activeTab() === TAB.info
        ? false
        : { project: project(), target: target(), changes: annotationChanges() },
    async (source): Promise<Read> => ({
      key: targetKey(source.target.kind, source.target.name, source.target.parameters),
      items: await readSymbol(source.project, source.target),
    }),
  );
  const mine = (): Annotation[] | undefined => {
    const latest = read.latest;
    const here = target();
    return latest !== undefined && latest.key === targetKey(here.kind, here.name)
      ? latest.items
      : undefined;
  };
  const note = (): string | null => {
    if (read.error) {
      const error: unknown = read.error;
      return error instanceof ApiError && error.code !== null
        ? errorText(error.code)
        : t(I18N_KEYS.ANNOTATIONS_LOAD_FAILED);
    }
    return mine() === undefined ? t(I18N_KEYS.ANNOTATIONS_LOADING) : null;
  };
  return {
    note,
    ofKind: (kind: (typeof KIND)[keyof typeof KIND]): Annotation[] =>
      (mine() ?? []).filter((one) => one.kind === kind),
  };
}
