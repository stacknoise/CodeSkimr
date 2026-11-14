import { For } from "solid-js";
import { cx } from "../../shared/ui/cx";
import type { Tile } from "./metrics";
import { ratingClass } from "./ratingClass";

/** The CSS classes a grid of tiles is drawn with; `good`, `medium` and `poor` colour the value. */
export type TileClasses = Readonly<Record<string, string | undefined>>;

/** The grid of metric tiles of the explorer and of the preview of the quick search: the value,
 * coloured by its rating, above its label, with the tip as the title. */
export function TileGrid(props: { tiles: Tile[]; classes: TileClasses }) {
  return (
    <div class={props.classes.grid}>
      <For each={props.tiles}>
        {(tile) => (
          <div class={props.classes.tile} title={tile.tip}>
            <div class={cx(props.classes.value, ratingClass(tile.rating, props.classes))}>
              {tile.value}
            </div>
            <div class={props.classes.label}>{tile.label}</div>
          </div>
        )}
      </For>
    </div>
  );
}
