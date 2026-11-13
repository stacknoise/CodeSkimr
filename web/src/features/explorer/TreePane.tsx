import { createMemo, For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { Icon } from "../../shared/ui/Icon";
import {
  CARET,
  ICON_COLLAPSE,
  ICON_LOCATE,
  KEY,
  LAYER_TONES,
  NOT_FOUND,
  ROLE,
  ROW,
  ROW_HEIGHT_PX,
  ROW_ID_PREFIX,
  TRUE,
} from "./consts";
import { collapseAll, currentMember, currentType, setTreeFilter, treeFilter } from "./explorer";
import { KindBadge } from "./KindBadge";
import { cursorTarget } from "./keys";
import { indent, type TreeRow } from "./tree";
import { useWindow } from "./useWindow";
import styles from "./TreePane.module.css";

/** Whether a row is the selected one: a member row by member, a type row by its type. */
function selected(row: TreeRow): boolean {
  if (row.kind === ROW.member) {
    return row.member === currentMember();
  }
  return row.kind === ROW.type && row.type === currentType() && currentMember() === null;
}

/** What the tree reports to the screen. */
export interface TreeActions {
  /** A click, `Enter` or `Space` on a row. */
  onActivate: (row: TreeRow) => void;
  /** The caret, `Right` and `Left`: open or close a row, nothing else. */
  onToggle: (row: TreeRow) => void;
  onCursor: (key: string) => void;
  onReveal: () => void;
}

function Dot(props: { index: number | null }) {
  const tone = () => `layer${Math.min((props.index ?? 0) + 1, LAYER_TONES)}`;
  return <span class={cx(styles.dot, styles[tone()])} aria-hidden={true} />;
}

function Item(props: { row: TreeRow; index: number; cursor: boolean; actions: TreeActions }) {
  const row = () => props.row;
  return (
    <div
      id={`${ROW_ID_PREFIX}${props.index}`}
      role={ROLE.item}
      class={cx(
        styles.row,
        selected(row()) && styles.selected,
        props.cursor && styles.cursor,
        row().kind === ROW.layer && styles.layer,
        row().kind === ROW.group && styles.group,
      )}
      style={{
        top: `${props.index * ROW_HEIGHT_PX}px`,
        "padding-left": `${indent(row().depth)}px`,
      }}
      aria-level={row().depth + 1}
      aria-selected={selected(row()) ? TRUE : undefined}
      aria-expanded={row().expandable ? row().open : undefined}
      onClick={() => props.actions.onActivate(row())}
    >
      <span
        class={styles.caret}
        aria-hidden={true}
        onClick={(event) => {
          event.stopPropagation();
          if (row().expandable) {
            props.actions.onToggle(row());
          }
        }}
      >
        {row().expandable ? (row().open ? CARET.open : CARET.closed) : ""}
      </span>
      <Show when={row().layerIndex !== null}>
        <Dot index={row().layerIndex} />
      </Show>
      <Show when={row().symbolKind}>{(kind) => <KindBadge kind={kind()} />}</Show>
      <span class={styles.label}>{row().label}</span>
      <span class={styles.meta}>{row().meta}</span>
    </div>
  );
}

function Tools(props: { onReveal: () => void }) {
  return (
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
  );
}

/** The keys of the tree: the arrows move the cursor (the row that `aria-activedescendant` names),
 * `Right` and `Left` open and close, `Enter` and `Space` act like a click. */
function onTreeKey(event: KeyboardEvent, rows: TreeRow[], cursor: number, actions: TreeActions) {
  const row = rows[cursor];
  const target = cursorTarget(event.key, cursor, rows.length);
  if (event.ctrlKey || event.altKey || event.metaKey) {
    return;
  }
  if (target !== null && rows[target]) {
    actions.onCursor(rows[target].key);
  } else if (row && event.key === KEY.right && row.expandable && !row.open) {
    actions.onToggle(row);
  } else if (row && event.key === KEY.left && row.open) {
    actions.onToggle(row);
  } else if (row && (event.key === KEY.enter || event.key === KEY.space)) {
    actions.onActivate(row);
  } else {
    return;
  }
  event.preventDefault();
}

/** The left column: filter, the two buttons and the tree (mock 2). The tree draws only the rows
 * in view (blueprint 15.5) and is one tab stop; its cursor moves with the arrow keys. */
export function TreePane(props: {
  rows: TreeRow[];
  cursor: string | null;
  actions: TreeActions;
  note: string | null;
}) {
  const cursorIndex = createMemo(() => props.rows.findIndex((row) => row.key === props.cursor));
  const view = useWindow(() => props.rows, cursorIndex);
  return (
    <section class={styles.pane} aria-label={t(I18N_KEYS.EXPLORER_STRUCTURE_LABEL)}>
      <Tools onReveal={props.actions.onReveal} />
      <Show when={props.note}>{(note) => <p class={styles.note}>{note()}</p>}</Show>
      <div
        ref={view.attach}
        class={styles.rows}
        role={ROLE.tree}
        tabindex={0}
        aria-label={t(I18N_KEYS.EXPLORER_STRUCTURE_LABEL)}
        aria-activedescendant={
          cursorIndex() === NOT_FOUND ? undefined : `${ROW_ID_PREFIX}${cursorIndex()}`
        }
        onScroll={view.onScroll}
        onKeyDown={(event) => onTreeKey(event, props.rows, cursorIndex(), props.actions)}
      >
        <div class={styles.window} style={{ height: `${view.total()}px` }}>
          <For each={view.visible()}>
            {(place) => (
              <Item
                row={place.item}
                index={place.index}
                cursor={place.index === cursorIndex()}
                actions={props.actions}
              />
            )}
          </For>
        </div>
      </div>
    </section>
  );
}
