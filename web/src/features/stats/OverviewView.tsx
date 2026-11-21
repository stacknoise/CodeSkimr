import { For, Show } from "solid-js";
import type { JSX } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import type { MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { KindBadge } from "../explorer/KindBadge";
import { count } from "../projects/format";
import { OPEN_ARROW, TAB } from "./consts";
import { shortPath } from "./geometry";
import { AssessmentPill, usageText } from "./RankingTable";
import type { Overview, RankingName, Row } from "./types";
import styles from "./OverviewView.module.css";

/** A number of the project with its title and a line below; `tone` is a class for the number. */
export function Tile(props: {
  title: MessageKey;
  value: number;
  detail: string;
  warn?: boolean;
  tone?: string | undefined;
}) {
  return (
    <div class={styles.tile}>
      <div class={styles.tileTitle}>{t(props.title)}</div>
      <div class={cx(styles.tileValue, props.tone)}>{count(props.value, currentLanguage())}</div>
      <div class={props.warn ? styles.tileWarn : styles.tileDetail}>{props.detail}</div>
    </div>
  );
}

function Tiles(props: { data: Overview }) {
  const number = (value: number) => count(value, currentLanguage());
  const unused = () =>
    props.data.unused.likely + props.data.unused.public + props.data.unused.approximate;
  return (
    <div class={styles.tiles}>
      <Tile
        title={I18N_KEYS.STATS_TILE_SYMBOLS}
        value={props.data.symbols}
        detail={t(I18N_KEYS.STATS_TILE_SYMBOLS_DETAIL, { count: number(props.data.ranked) })}
      />
      <Tile
        title={I18N_KEYS.STATS_TILE_USAGES}
        value={props.data.references.certain + props.data.references.probable}
        detail={t(I18N_KEYS.STATS_TILE_USAGES_DETAIL, {
          certain: number(props.data.references.certain),
          probable: number(props.data.references.probable),
        })}
      />
      <Tile
        title={I18N_KEYS.STATS_TILE_UNUSED}
        value={unused()}
        detail={t(I18N_KEYS.STATS_TILE_UNUSED_DETAIL)}
        warn={true}
      />
      <Tile
        title={I18N_KEYS.STATS_TILE_COMPLEXITY}
        value={props.data.complexity.over_limit}
        detail={t(I18N_KEYS.STATS_TILE_COMPLEXITY_DETAIL, { limit: props.data.complexity.limit })}
      />
    </div>
  );
}

function CardRow(props: {
  row: Row;
  value: (row: Row) => JSX.Element;
  onOpen: (row: Row) => void;
}) {
  return (
    <li>
      <button
        type="button"
        class={styles.row}
        aria-label={t(I18N_KEYS.STATS_OPEN_EXPLORER_ROW, { name: props.row.name })}
        onClick={() => props.onOpen(props.row)}
      >
        <KindBadge kind={props.row.kind} />
        <span class={styles.name} title={props.row.qualified_name}>
          {props.row.name}
        </span>
        <span class={styles.path}>{shortPath(props.row.path)}</span>
        <span class={styles.value}>{props.value(props.row)}</span>
        <span class={styles.arrow} aria-hidden={true}>
          {OPEN_ARROW}
        </span>
      </button>
    </li>
  );
}

/** One card of the overview: the first rows of a ranking and the way to all of them. */
function Card(props: {
  title: MessageKey;
  detail: MessageKey;
  rows: Row[];
  tab: RankingName;
  value: (row: Row) => JSX.Element;
  onShowAll: (tab: RankingName) => void;
  onOpen: (row: Row) => void;
}) {
  return (
    <section class={styles.card}>
      <div class={styles.cardHead}>
        <h2>{t(props.title)}</h2>
        <span class={styles.cardDetail}>{t(props.detail)}</span>
        <button type="button" class={styles.more} onClick={() => props.onShowAll(props.tab)}>
          {t(I18N_KEYS.STATS_SHOW_ALL)}
        </button>
      </div>
      <Show
        when={props.rows.length > 0}
        fallback={<p class={styles.empty}>{t(I18N_KEYS.STATS_EMPTY)}</p>}
      >
        <ol class={styles.rows}>
          <For each={props.rows}>
            {(row) => <CardRow row={row} value={props.value} onOpen={props.onOpen} />}
          </For>
        </ol>
      </Show>
    </section>
  );
}

const number = (value: number | null) => count(value ?? 0, currentLanguage());

/** The four cards of the overview: their texts, their tab and the value they show per row. */
const CARDS: readonly {
  title: MessageKey;
  detail: MessageKey;
  tab: RankingName;
  rows: (data: Overview) => Row[];
  value: (row: Row) => JSX.Element;
}[] = [
  {
    title: I18N_KEYS.STATS_CARD_MOST_USED,
    detail: I18N_KEYS.STATS_CARD_MOST_USED_DETAIL,
    tab: TAB.mostUsed,
    rows: (data) => data.top.most_used,
    value: usageText,
  },
  {
    title: I18N_KEYS.STATS_CARD_UNUSED,
    detail: I18N_KEYS.STATS_CARD_UNUSED_DETAIL,
    tab: TAB.unused,
    rows: (data) => data.top.unused,
    value: (row) => <AssessmentPill row={row} />,
  },
  {
    title: I18N_KEYS.STATS_CARD_COMPLEXITY,
    detail: I18N_KEYS.STATS_CARD_COMPLEXITY_DETAIL,
    tab: TAB.complexity,
    rows: (data) => data.top.complexity,
    value: (row) => number(row.complexity),
  },
  {
    title: I18N_KEYS.STATS_CARD_COUPLING,
    detail: I18N_KEYS.STATS_CARD_COUPLING_DETAIL,
    tab: TAB.coupling,
    rows: (data) => data.top.coupling,
    value: (row) => number(row.efferent),
  },
];

/** The start of the screen: the numbers of the project and the first rows of four rankings. Every
 * row opens its symbol in the explorer. */
export function OverviewView(props: {
  data: Overview;
  onShowAll: (tab: RankingName) => void;
  onOpen: (row: Row) => void;
}) {
  return (
    <div class={styles.overview}>
      <Tiles data={props.data} />
      <div class={styles.cards}>
        <For each={CARDS}>
          {(card) => (
            <Card
              title={card.title}
              detail={card.detail}
              tab={card.tab}
              rows={card.rows(props.data)}
              value={card.value}
              onShowAll={props.onShowAll}
              onOpen={props.onOpen}
            />
          )}
        </For>
      </div>
    </div>
  );
}
