import { For } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { LANGUAGE_SEPARATOR } from "./consts";
import { languageShares } from "./format";
import type { LanguageInfo } from "./types";
import styles from "./LanguageBar.module.css";

/** The bar of the languages of a project and their shares as text (mock 1, column "Languages"). */
export function LanguageBar(props: { languages: LanguageInfo[] }) {
  const shares = () => languageShares(props.languages);
  return (
    <>
      <div class={styles.bar} aria-hidden={true}>
        <For each={shares()}>
          {(share) => (
            <div
              class={cx(styles.segment, styles[share.tone as keyof typeof styles])}
              ref={(element) => element.style.setProperty("width", `${share.percent}%`)}
            />
          )}
        </For>
      </div>
      <div class={styles.text}>
        {shares()
          .map((share) =>
            t(I18N_KEYS.PROJECTS_LANGUAGE_MIX, { name: share.name, percent: share.percent }),
          )
          .join(LANGUAGE_SEPARATOR)}
      </div>
    </>
  );
}
