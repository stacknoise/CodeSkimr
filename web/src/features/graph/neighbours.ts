import type { LayerLink } from "./types";

/** What depends on a layer and what it depends on. */
export interface Neighbours {
  uses: LayerLink[];
  usedBy: LayerLink[];
  references: number;
}

/** The links that leave and enter `layer`. Links of a layer to itself are left out. */
export function neighboursOf(layer: string, links: readonly LayerLink[]): Neighbours {
  const own = links.filter((link) => link.from !== link.to);
  const uses = own.filter((link) => link.from === layer);
  const usedBy = own.filter((link) => link.to === layer);
  const references = [...uses, ...usedBy].reduce((sum, link) => sum + link.count, 0);
  return { uses, usedBy, references };
}

/** The links that break the layering, most references first. */
export function violations(links: readonly LayerLink[]): LayerLink[] {
  return links.filter((link) => link.violates_layering).sort((a, b) => b.count - a.count);
}
