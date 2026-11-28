import { For, Show, createMemo, createSignal, onMount } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { createTableColumns } from "../../shared/table/columns";
import { ColumnWidths, HeadRow } from "../../shared/table/ResizableHead";
import { TONE } from "../../shared/ui/consts";
import { Pill } from "../../shared/ui/Pill";
import {
  EFFECTIVE_COLUMNS,
  EFFECTIVE_TABLE_KEY,
  EFFECTIVE_ONLY_CHANGED_ID,
  EDIT,
  ORIGIN,
  ORIGIN_TEXTS,
} from "./consts";
import {
  displayValue,
  effectiveSearch,
  effectiveFailed,
  effectiveSettings,
  filterSettings,
  groupSettings,
  groupTitle,
  loadEffectiveSettings,
  saveSettingValue,
  type EffectiveSetting,
  type EffectiveSettings,
} from "./effective";
import { SettingDialog } from "./SettingDialog";
import styles from "./EffectivePanel.module.css";

const columns = createTableColumns(
  EFFECTIVE_TABLE_KEY,
  EFFECTIVE_COLUMNS.map((column) => column.id),
);

/** The pill that tells where a value comes from. */
function OriginPill(props: { origin: EffectiveSetting["origin"] }) {
  const tone = () =>
    props.origin === ORIGIN.default
      ? TONE.neutral
      : props.origin === ORIGIN.cli
        ? TONE.warn
        : TONE.accent;
  return <Pill tone={tone()}>{t(ORIGIN_TEXTS[props.origin])}</Pill>;
}

/** The configuration file that was read and the flags of the start. */
function Sources(props: { info: EffectiveSettings }) {
  return (
    <div class={styles.sources}>
      <div>
        <span class={styles.label}>{t(I18N_KEYS.SETTINGS_EFFECTIVE_FILE)}</span>
        <Show
          when={props.info.user_file}
          fallback={<div class={styles.muted}>{t(I18N_KEYS.SETTINGS_EFFECTIVE_FILE_NONE)}</div>}
        >
          {(file) => (
            <div class={styles.mono}>
              {file().path}{" "}
              <Pill tone={file().found ? TONE.good : TONE.neutral}>
                {file().found
                  ? t(I18N_KEYS.SETTINGS_EFFECTIVE_FILE_FOUND)
                  : t(I18N_KEYS.SETTINGS_EFFECTIVE_FILE_MISSING)}
              </Pill>
            </div>
          )}
        </Show>
      </div>
      <div>
        <span class={styles.label}>{t(I18N_KEYS.SETTINGS_EFFECTIVE_FLAGS)}</span>
        <Show
          when={props.info.overrides.length > 0}
          fallback={<div class={styles.muted}>{t(I18N_KEYS.SETTINGS_EFFECTIVE_FLAGS_NONE)}</div>}
        >
          <For each={props.info.overrides}>
            {(flag) => <div class={styles.mono}>{`--set ${flag}`}</div>}
          </For>
        </Show>
      </div>
    </div>
  );
}

/** Why a value cannot be changed here, or `null` if it can. */
function lockReason(setting: EffectiveSetting): string | null {
  if (setting.edit === EDIT.cli) {
    return t(I18N_KEYS.SETTINGS_EFFECTIVE_BY_FLAG);
  }
  return setting.edit === EDIT.locked ? t(I18N_KEYS.SETTINGS_EFFECTIVE_LOCKED) : null;
}

/** The note under a key: why it is locked, or that a saved value waits for the next start. */
function KeyNote(props: { setting: EffectiveSetting }) {
  const note = () =>
    lockReason(props.setting) ??
    (props.setting.pending === null
      ? null
      : t(I18N_KEYS.SETTINGS_EFFECTIVE_PENDING, { value: props.setting.pending }));
  return <Show when={note()}>{(text) => <span class={styles.hint}>{text()}</span>}</Show>;
}

/** The buttons of a row: "Change" opens the dialog, and a value saved in the app can go back to
 * the default at once. A value that cannot be changed keeps its grey button with the reason. */
function RowActions(props: {
  setting: EffectiveSetting;
  onChange: (setting: EffectiveSetting) => void;
}) {
  const reason = () => lockReason(props.setting);
  return (
    <td class={styles.actions}>
      <button
        type="button"
        class={styles.rowButton}
        disabled={reason() !== null}
        title={reason() ?? undefined}
        onClick={() => props.onChange(props.setting)}
      >
        {t(I18N_KEYS.SETTINGS_EFFECTIVE_CHANGE)}
      </button>
      <Show when={props.setting.origin === ORIGIN.ui || props.setting.pending !== null}>
        <button
          type="button"
          class={styles.rowButton}
          onClick={() => void saveSettingValue(props.setting.key, null)}
        >
          {t(I18N_KEYS.SETTINGS_EFFECTIVE_RESET)}
        </button>
      </Show>
    </td>
  );
}

function Rows(props: {
  settings: EffectiveSetting[];
  onChange: (setting: EffectiveSetting) => void;
}) {
  return (
    <For each={groupSettings(props.settings)}>
      {(group) => (
        <>
          <tr class={styles.group}>
            <th colSpan={EFFECTIVE_COLUMNS.length} scope="colgroup">
              {groupTitle(group.name)}
            </th>
          </tr>
          <For each={group.settings}>
            {(setting) => (
              <tr>
                <td class={styles.mono}>
                  {setting.key}
                  <KeyNote setting={setting} />
                </td>
                <td
                  class={`${styles.mono} ${setting.origin === ORIGIN.default ? "" : styles.changed}`}
                >
                  {displayValue(setting)}
                </td>
                <td>
                  <OriginPill origin={setting.origin} />
                </td>
                <RowActions setting={setting} onChange={props.onChange} />
              </tr>
            )}
          </For>
        </>
      )}
    </For>
  );
}

function Table(props: {
  settings: EffectiveSetting[];
  onChange: (setting: EffectiveSetting) => void;
}) {
  return (
    <div class={styles.scroll}>
      <table class={styles.table} style={columns.style()}>
        <ColumnWidths columns={columns} />
        <thead>
          <HeadRow columns={columns} heads={EFFECTIVE_COLUMNS} />
        </thead>
        <tbody>
          <Rows settings={props.settings} onChange={props.onChange} />
        </tbody>
      </table>
    </div>
  );
}

/** The switch to the values that differ from the default. */
function Tools(props: { onlyChanged: boolean; onOnlyChanged: (on: boolean) => void }) {
  return (
    <div class={styles.tools}>
      <label class={styles.check} for={EFFECTIVE_ONLY_CHANGED_ID}>
        <input
          id={EFFECTIVE_ONLY_CHANGED_ID}
          type="checkbox"
          checked={props.onlyChanged}
          onChange={(event) => props.onOnlyChanged(event.currentTarget.checked)}
        />
        <span>{t(I18N_KEYS.SETTINGS_EFFECTIVE_ONLY_CHANGED)}</span>
      </label>
    </div>
  );
}

/** The sources, the filter, the table and the count of what is shown. */
function Overview(props: { info: EffectiveSettings }) {
  const [onlyChanged, setOnlyChanged] = createSignal(false);
  const [editing, setEditing] = createSignal<string | null>(null);
  const editedSetting = () => props.info.settings.find((entry) => entry.key === editing());
  const shown = createMemo(() =>
    filterSettings(props.info.settings, effectiveSearch(), onlyChanged()),
  );
  return (
    <>
      <Tools onlyChanged={onlyChanged()} onOnlyChanged={setOnlyChanged} />
      <Show
        when={shown().length > 0}
        fallback={<p class={styles.muted}>{t(I18N_KEYS.SETTINGS_EFFECTIVE_EMPTY)}</p>}
      >
        <Table settings={shown()} onChange={(setting) => setEditing(setting.key)} />
      </Show>
      <Show when={editedSetting()}>
        {(setting) => <SettingDialog setting={setting()} onClose={() => setEditing(null)} />}
      </Show>
      <div class={styles.foot}>
        <span>
          {t(I18N_KEYS.SETTINGS_EFFECTIVE_COUNT, {
            shown: shown().length,
            total: props.info.settings.length,
          })}
        </span>
        <span class={styles.mono}>{t(I18N_KEYS.SETTINGS_EFFECTIVE_AS_OF)}</span>
      </div>
    </>
  );
}

/** The configuration file that was read and the flags of the start (tab "Info"). */
export function ConfigSourcesPanel() {
  onMount(() => void loadEffectiveSettings());
  return (
    <section class={styles.panel} aria-label={t(I18N_KEYS.SETTINGS_SOURCES_TITLE)}>
      <h2>{t(I18N_KEYS.SETTINGS_SOURCES_TITLE)}</h2>
      <Show when={effectiveFailed()}>
        <p role="alert" class={styles.error}>
          {t(I18N_KEYS.SETTINGS_EFFECTIVE_FAILED)}
        </p>
      </Show>
      <Show when={effectiveSettings()}>{(info) => <Sources info={info()} />}</Show>
    </section>
  );
}

/** The settings in effect: each value with the layer that set it (default, configuration file or
 * command line), narrowed by the search of the tab bar and by a switch to the values that differ
 * from the default. "Change" saves a number or a switch in the file of the app (mock 45, ADR 0118). */
export function EffectivePanel() {
  onMount(() => void loadEffectiveSettings());
  return (
    <section class={styles.panel} aria-label={t(I18N_KEYS.SETTINGS_EFFECTIVE_TITLE)}>
      <h2>{t(I18N_KEYS.SETTINGS_EFFECTIVE_TITLE)}</h2>
      <p class={styles.note}>{t(I18N_KEYS.SETTINGS_EFFECTIVE_NOTE)}</p>
      <Show when={effectiveFailed()}>
        <p role="alert" class={styles.error}>
          {t(I18N_KEYS.SETTINGS_EFFECTIVE_FAILED)}
        </p>
      </Show>
      <Show when={effectiveSettings()}>{(info) => <Overview info={info()} />}</Show>
    </section>
  );
}
