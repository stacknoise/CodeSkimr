import { createMemo, For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { Icon } from "../../shared/ui/Icon";
import { CARET, ICON_COLLAPSE, ICON_LOCATE, ROLE, ROW, TRUE } from "./consts";
import {
  collapseAll,
  currentMember,
  currentType,
  expandedRows,
  setTreeFilter,
  symbolDetails,
  treeFilter,
  treeOutline,
} from "./explorer";
import { KindBadge } from "./KindBadge";
import { buildRows, indent, type TreeLabels, type TreeRow } from "./tree";
import styles from "./TreePane.module.css";

function labels(): TreeLabels {
  return {
    noLayer: t(I18N_KEYS.EXPLORER_LAYER_OTHER),
    noNamespace: t(I18N_KEYS.EXPLORER_NAMESPACE_ROOT),
    fields: t(I18N_KEYS.EXPLORER_GROUP_FIELDS),
    methods: t(I18N_KEYS.EXPLORER_GROUP_METHODS),
  };
}

/** The rows that are shown right now. */
export function useRows() {
  return createMemo<TreeRow[]>(() => {
    const outline = treeOutline();
    return outline
      ? buildRows(outline, expandedRows(), symbolDetails(), treeFilter(), labels())
      : [];
  });
}

/** Whether a row is the selected one: a member row by member, every other row by its type. */
function selected(row: TreeRow): boolean {
  if (row.kind === ROW.member) {
    return row.member === currentMember();
  }
  return row.kind === ROW.type && row.type === currentType() && currentMember() === null;
}

function Row(props: { row: TreeRow; onPick: (row: TreeRow) => void }) {
  const row = () => props.row;
  return (
    <div
      role={ROLE.item}
      class={cx(
        styles.row,
        selected(row()) && styles.selected,
        row().kind === ROW.layer && styles.layer,
        row().kind === ROW.group && styles.group,
      )}
      style={{ "padding-left": `${indent(row().depth)}px` }}
      aria-selected={selected(row()) ? TRUE : undefined}
      aria-expanded={row().expandable ? row().open : undefined}
      data-key={row().key}
      onClick={() => props.onPick(row())}
    >
      <span class={styles.caret} aria-hidden={true}>
        {row().expandable ? (row().open ? CARET.open : CARET.closed) : ""}
      </span>
      <Show when={row().symbolKind}>{(kind) => <KindBadge kind={kind()} />}</Show>
      <span class={styles.label}>{row().label}</span>
      <span class={styles.meta}>{row().meta}</span>
    </div>
  );
}

/** The left column: filter, the two buttons and the tree (mock 2). */
export function TreePane(props: {
  rows: TreeRow[];
  onPick: (row: TreeRow) => void;
  onReveal: () => void;
}) {
  return (
    <section class={styles.pane} aria-label={t(I18N_KEYS.EXPLORER_STRUCTURE_LABEL)}>
      <div class={styles.tools}>
        <input
          class={styles.filter}
          value={treeFilter()}
          placeholder={t(I18N_KEYS.EXPLORER_FILTER_TREE)}
          aria-label={t(I18N_KEYS.EXPLORER_FILTER_TREE)}
          onInput={(event) => setTreeFilter(event.currentTarget.value)}
          spellcheck={false}
          autocomplete="off"
        />
        <button
          type="button"
          class={styles.tool}
          title={t(I18N_KEYS.EXPLORER_COLLAPSE_ALL)}
          aria-label={t(I18N_KEYS.EXPLORER_COLLAPSE_ALL)}
          onClick={collapseAll}
        >
          <Icon name={ICON_COLLAPSE} />
        </button>
        <button
          type="button"
          class={styles.tool}
          title={t(I18N_KEYS.EXPLORER_REVEAL_SELECTION)}
          aria-label={t(I18N_KEYS.EXPLORER_REVEAL_SELECTION)}
          onClick={props.onReveal}
        >
          <Icon name={ICON_LOCATE} />
        </button>
      </div>
      <div class={styles.rows} role={ROLE.tree}>
        <For each={props.rows}>{(row) => <Row row={row} onPick={props.onPick} />}</For>
      </div>
    </section>
  );
}
