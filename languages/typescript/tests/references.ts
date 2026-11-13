import Default, { Named as Alias } from "./mod";
import * as ns from "./ns";

export class Cart extends Base<number> implements Iterable<Item>, Priced {
  private readonly items: Item[] = [];
  private store = new Store();
  constructor(private readonly repo: Repo) {
    super();
  }

  override run(): void {}

  find(key: string, fallback?: Item): Item {
    helper(key);
    this.count = this.items.length + offset;
    this.count++;
    const copy = fallback;
    const created: Item = new Item();
    const total = compute<number>(this.count);
    this.repo.save(created);
    ns.log(copy);
    return total > 1 ? created : copy;
  }
}

export interface Named extends Base {}

function helper(a: Named, b: Cart[]): Promise<Cart> {
  let local = a;
  for (const item of b) {
    local = item;
  }
  try {
  } catch (error) {
    throw error;
  }
  return null as unknown as Promise<Cart>;
}

const mapper = (value: number) => value * LIMIT;
