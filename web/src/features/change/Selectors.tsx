import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { changePicked, changeReading, changeScans, loadChange, pickScans } from "./change";
import { BASE_FIELD_ID, NOT_FOUND, SCAN_ARROW, TARGET_FIELD_ID } from "./consts";
import { optionText } from "./scans";
import parts from "./Parts.module.css";
import styles from "./Selectors.module.css";

/** The choice of one scan: a label and a list of the scans that can be that end of the comparison. */
function ScanField(props: {
  id: string;
  label: string;
  value: number | null;
  /** Indexes into the list of the scans, newest first, that the field offers. */
  indexes: readonly number[];
  onPick: (scan: number) => void;
}) {
  return (
    <div class={styles.field}>
      <label class={parts.figureLabel} for={props.id}>
        {props.label}
      </label>
      <select
        id={props.id}
        class={styles.select}
        disabled={changeReading()}
        onChange={(event) => props.onPick(Number(event.currentTarget.value))}
      >
        <For each={props.indexes}>
          {(index) => {
            const scan = () => changeScans()?.scans[index];
            return (
              <Show when={scan()}>
                {(entry) => (
                  <option value={entry().scan} selected={entry().scan === props.value}>
                    {optionText(entry(), index === 0)}
                  </option>
                )}
              </Show>
            );
          }}
        </For>
      </select>
    </div>
  );
}

/** The indexes of the scans that can be the base (older than the target) and the target (newer
 * than the base). */
function offers(): { base: number[]; target: number[] } {
  const scans = changeScans()?.scans ?? [];
  const at = (id: number | null, fallback: number) => {
    const found = scans.findIndex((scan) => scan.scan === id);
    return found === NOT_FOUND ? fallback : found;
  };
  const target = at(changePicked().target, 0);
  const base = at(changePicked().base, scans.length - 1);
  const all = scans.map((_, index) => index);
  return {
    base: all.filter((index) => index > target),
    target: all.filter((index) => index < base),
  };
}

/** The two selectors of the scans and the button that compares them (mock 29). */
export function Selectors(props: { project: number }) {
  return (
    <div class={styles.selectors}>
      <ScanField
        id={BASE_FIELD_ID}
        label={t(I18N_KEYS.CHANGE_VIEW_BASE)}
        value={changePicked().base}
        indexes={offers().base}
        onPick={(base) => pickScans({ ...changePicked(), base })}
      />
      <span class={styles.arrow} aria-hidden={true}>
        {SCAN_ARROW}
      </span>
      <ScanField
        id={TARGET_FIELD_ID}
        label={t(I18N_KEYS.CHANGE_VIEW_TARGET)}
        value={changePicked().target}
        indexes={offers().target}
        onPick={(target) => pickScans({ ...changePicked(), target })}
      />
      <button
        type="button"
        class={styles.run}
        disabled={changeReading()}
        onClick={() => void loadChange(props.project, changePicked())}
      >
        {t(I18N_KEYS.CHANGE_VIEW_RUN)}
      </button>
    </div>
  );
}
