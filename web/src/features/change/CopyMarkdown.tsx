import { createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { COPY } from "../../shared/ui/consts";
import { createCopier } from "../../shared/ui/copier";
import type { MarkdownAnswer } from "./types";
import styles from "./ChangeScreen.module.css";

/** The button "Copy as Markdown" (mock erw-auswirkung): it asks `scans.impact_markdown`, the
 * counterpart of `codeskimr change --markdown`, for the text of the two scans that are shown and
 * puts it on the clipboard. The button says "Copied" or "Not copied" for a moment; the text is made
 * by the server, so the screen and the command line give the same. In the view "Metrics" the same
 * button asks `scans.metrics_markdown` (`compare --metrics --markdown`). */
export function CopyMarkdown(props: {
  command: string;
  /** What the command is asked: the project, the two scans and, for the metrics, the level. */
  input: () => Record<string, unknown>;
}) {
  const [refused, setRefused] = createSignal(false);
  const { state, copy } = createCopier();
  const ask = async () => {
    try {
      const text = await runCommand<MarkdownAnswer>(props.command, {
        ...props.input(),
        language: currentLanguage(),
      });
      setRefused(false);
      copy(text.markdown);
    } catch {
      setRefused(true);
    }
  };
  const failed = () => refused() || state() === COPY.failed;
  const label = () =>
    state() === COPY.copied && !refused()
      ? t(I18N_KEYS.CHANGE_VIEW_COPY_DONE)
      : failed()
        ? t(I18N_KEYS.CHANGE_VIEW_COPY_FAILED)
        : t(I18N_KEYS.CHANGE_VIEW_COPY);
  return (
    <span role="status">
      <button type="button" class={styles.copy} onClick={() => void ask()}>
        {label()}
      </button>
    </span>
  );
}
