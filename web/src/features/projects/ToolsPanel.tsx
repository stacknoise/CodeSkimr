import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { TONE } from "../../shared/ui/consts";
import { Pill } from "../../shared/ui/Pill";
import { TOOL_STATE } from "./consts";
import { checkTools, toolsChecking, toolsOf } from "./projects";
import { languageList, problemsOf, readyLanguages, toolTexts } from "./tools";
import type { ProjectRow, ToolInfo } from "./types";
import styles from "./ToolsPanel.module.css";

/** What is wrong with a tool, in words. */
function stateText(tool: ToolInfo): string {
  if (tool.state === TOOL_STATE.missing) {
    return t(I18N_KEYS.PROJECTS_TOOLS_STATE_MISSING);
  }
  const required = tool.required ?? "";
  return tool.state === TOOL_STATE.tooOld
    ? t(I18N_KEYS.PROJECTS_TOOLS_STATE_TOO_OLD, { found: tool.found ?? "", required })
    : t(I18N_KEYS.PROJECTS_TOOLS_STATE_UNKNOWN, { required });
}

function Problem(props: { tool: ToolInfo }) {
  const texts = () => toolTexts(props.tool);
  return (
    <li class={styles.problem}>
      <div class={styles.head}>
        <strong class={styles.tool}>{props.tool.tool}</strong>
        <Pill tone={TONE.warn}>{stateText(props.tool)}</Pill>
        <span class={styles.languages}>
          {t(I18N_KEYS.PROJECTS_TOOLS_FOR, { languages: languageList(props.tool.languages) })}
        </span>
      </div>
      <p>{t(I18N_KEYS.PROJECTS_TOOLS_PURPOSE, { text: texts().purpose })}</p>
      <p>{t(I18N_KEYS.PROJECTS_TOOLS_INSTALL, { text: texts().install })}</p>
    </li>
  );
}

/** What the check of the tools found for the project: a warning with each tool that is missing,
 * too old or of unknown version, what it is for and how to install it (ADR 0033, decision 8). */
export function ToolsPanel(props: { row: ProjectRow }) {
  const info = () => toolsOf(props.row);
  const problems = () => problemsOf(info());
  const ready = () => readyLanguages(info());
  return (
    <section
      class={styles.panel}
      aria-label={t(I18N_KEYS.PROJECTS_TOOLS_TITLE, { name: props.row.name })}
    >
      <header class={styles.header}>
        <h2>{t(I18N_KEYS.PROJECTS_TOOLS_TITLE, { name: props.row.name })}</h2>
        <button
          type="button"
          class={styles.again}
          disabled={toolsChecking().has(props.row.id)}
          onClick={() => void checkTools(props.row.id)}
        >
          {toolsChecking().has(props.row.id)
            ? t(I18N_KEYS.PROJECTS_TOOLS_CHECKING)
            : t(I18N_KEYS.PROJECTS_TOOLS_CHECK_AGAIN)}
        </button>
      </header>
      <Show
        when={info()?.checked_at != null}
        fallback={<p class={styles.muted}>{t(I18N_KEYS.PROJECTS_TOOLS_UNCHECKED)}</p>}
      >
        <Show when={problems().length > 0}>
          <div role="status" class={styles.warning}>
            <p>{t(I18N_KEYS.PROJECTS_TOOLS_WARNING)}</p>
            <ul class={styles.list}>
              <For each={problems()}>{(tool) => <Problem tool={tool} />}</For>
            </ul>
          </div>
        </Show>
        <Show when={ready().length > 0}>
          <p class={styles.ready}>
            {t(I18N_KEYS.PROJECTS_TOOLS_READY_LANGUAGES, { languages: languageList(ready()) })}
          </p>
        </Show>
        <Show when={problems().length === 0}>
          <p class={styles.muted}>
            {ready().length > 0
              ? t(I18N_KEYS.PROJECTS_TOOLS_ALL_READY)
              : t(I18N_KEYS.PROJECTS_TOOLS_NO_LEVEL2)}
          </p>
        </Show>
      </Show>
    </section>
  );
}
