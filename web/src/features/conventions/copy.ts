import { createSignal } from "solid-js";
import { t, type MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { conventionsAnswer } from "./conventions";
import { COPY_NOTE, COPY_NOTE_MS, MARKDOWN, PLACE_SEPARATOR } from "./consts";
import { measureText, placeText } from "./measure";
import type { ConventionsOut, Finding } from "./types";

/** One finding as a line of a Markdown list: the code, the place, the function and the measure. */
function bullet(finding: Finding): string {
  const code = `${MARKDOWN.code}${finding.code}${MARKDOWN.code}`;
  const place = `${MARKDOWN.code}${placeText(finding, PLACE_SEPARATOR)}${MARKDOWN.code}`;
  return (
    MARKDOWN.bullet +
    [code, place, finding.symbol, measureText(finding, PLACE_SEPARATOR)]
      .filter((part) => part !== "")
      .join(MARKDOWN.join)
  );
}

/** The result as Markdown, for an issue or a comment: a head line and one line per finding. */
export function markdownOf(answer: ConventionsOut, project: string): string {
  const head = t(I18N_KEYS.CONVENTIONS_MARKDOWN_HEAD, {
    project,
    findings: answer.total,
    files: answer.files_checked,
  });
  const lines =
    answer.findings.length === 0
      ? [t(I18N_KEYS.CONVENTIONS_MARKDOWN_NONE)]
      : answer.findings.map(bullet);
  return [head, ...lines].join(MARKDOWN.lineBreak);
}

/** The state of the copy button: its label, which shows for a moment whether the copy worked, and
 * the copy itself. */
export function copyNote() {
  const [note, setNote] = createSignal<(typeof COPY_NOTE)[keyof typeof COPY_NOTE] | null>(null);
  const hold = (next: (typeof COPY_NOTE)[keyof typeof COPY_NOTE]) => {
    setNote(next);
    setTimeout(() => setNote(null), COPY_NOTE_MS);
  };
  const label = (): MessageKey =>
    note() === COPY_NOTE.done
      ? I18N_KEYS.CONVENTIONS_COPIED
      : note() === COPY_NOTE.failed
        ? I18N_KEYS.CONVENTIONS_COPY_FAILED
        : I18N_KEYS.CONVENTIONS_COPY;
  const copy = async (project: string) => {
    const answer = conventionsAnswer();
    if (answer === null) {
      return;
    }
    try {
      await navigator.clipboard.writeText(markdownOf(answer, project));
      hold(COPY_NOTE.done);
    } catch {
      hold(COPY_NOTE.failed);
    }
  };
  return { label, copy };
}
