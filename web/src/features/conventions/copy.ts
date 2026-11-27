import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { MARKDOWN, PLACE_SEPARATOR } from "./consts";
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
