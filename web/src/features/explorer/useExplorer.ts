import { createMemo, createSignal } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject } from "../../shared/project/active";
import { ROW, TYPE_KINDS } from "./consts";
import {
  currentMember,
  currentType,
  expandedRows,
  openRows,
  revealNested,
  selectType,
  symbolDetails,
  toggleRow,
  toggleType,
  treeFilter,
  treeOutline,
} from "./explorer";
import { buildRows, revealKeys, typeKey, type TreeLabels, type TreeRow } from "./tree";
import type { MemberRow } from "./types";

function labels(): TreeLabels {
  return {
    noLayer: t(I18N_KEYS.EXPLORER_LAYER_OTHER),
    noNamespace: t(I18N_KEYS.EXPLORER_NAMESPACE_ROOT),
    fields: t(I18N_KEYS.EXPLORER_GROUP_FIELDS),
    methods: t(I18N_KEYS.EXPLORER_GROUP_METHODS),
    nested: t(I18N_KEYS.EXPLORER_GROUP_TYPES),
  };
}

/** A click on a row. A type is shown first and folded on the second click; a layer, a namespace
 * and a group fold; a member is shown with its type. Nothing but a click on an already shown
 * type changes which rows are open (blueprint 15.4). */
export function activate(project: number, row: TreeRow): void {
  const shown = row.kind === ROW.type && row.type === currentType() && currentMember() === null;
  if (row.type !== null && (row.kind === ROW.type || row.kind === ROW.member)) {
    if (shown) {
      void toggleType(project, row.type);
    } else {
      void selectType(project, row.type, row.member);
    }
  } else {
    toggleRow(row.key);
  }
}

/** The caret, `Right` and `Left`: opens or closes the row and leaves the selection. */
export function fold(project: number, row: TreeRow): void {
  if (row.kind === ROW.type && row.type !== null) {
    void toggleType(project, row.type);
  } else {
    toggleRow(row.key);
  }
}

/** Everything the explorer screen does with the tree and the table. */
export function useExplorer() {
  const [cursor, setCursor] = createSignal<string | null>(null);
  const rows = createMemo<TreeRow[]>(() => {
    const outline = treeOutline();
    return outline
      ? buildRows(outline, expandedRows(), symbolDetails(), treeFilter(), labels())
      : [];
  });
  const withProject = (run: (project: number) => void) => () => {
    const project = activeProject();
    if (project !== null) {
      run(project);
    }
  };
  const reveal = () => {
    const outline = treeOutline();
    const type = currentType();
    if (outline && type !== null) {
      openRows(revealKeys(outline, type).filter((key) => key !== typeKey(type)));
      setCursor(typeKey(type));
    }
  };
  return {
    rows,
    cursor,
    reveal,
    actions: {
      onCursor: setCursor,
      onReveal: reveal,
      onActivate: (row: TreeRow) => {
        setCursor(row.key);
        withProject((project) => activate(project, row))();
      },
      onToggle: (row: TreeRow) => withProject((project) => fold(project, row))(),
    },
    pickMember: (owner: number, member: MemberRow) =>
      withProject((project) => {
        if (TYPE_KINDS.includes(member.kind)) {
          void revealNested(project, owner, member.id);
        } else {
          void selectType(project, owner, member.id);
        }
      })(),
  };
}
