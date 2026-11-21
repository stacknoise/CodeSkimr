import { Base } from "./base";

/**
 * An order of a customer.
 *
 * Longer text.
 * @example new Order()
 */
export abstract class Order<T> extends Base implements Priced {
  #secret = 1;
  private static readonly MAX: number = 100;
  public name: string;
  items?: string[];
  handler = () => {
    return 1;
  };

  /** Creates an order. */
  constructor(private readonly id: number) {
    super();
  }

  get total(): number {
    return 1;
  }

  set total(value: number) {}

  public static async create(id: number): Promise<Order<string>> {
    return null as never;
  }

  protected abstract validate(): boolean;

  private helper() {}

  // not a doc comment
  plain() {}

  overload(a: string): void;
  overload(a: number): void {}

  *generate() {}
}

/** A priced thing. */
export interface Priced {
  /** The price. */
  price: number;
  readonly currency: string;
  format(locale: string): string;
}

interface Local {}

export type Id = string | number;

export enum Status {
  /** Open. */
  Open,
  Closed = 2,
}

export const LIMIT = 10;
const hidden = "x";
export const add = (a: number, b: number) => a + b;
const sub = function (a: number, b: number) {
  return a - b;
};
let counter = 0;

/** Does things. */
export function run(): void {
  function inner() {}
}

export default class Main {}

export namespace Util {
  /** The most that helps. */
  export const LIMIT = 3;
  const hidden = 'x';
  export function help() {}
}

declare function legacy(a: string): void;
export declare const VERSION: string;
declare class Ambient {
  static make(): Ambient;
  run(): void;
}

export const handlers = {
  onClick() {},
};

/**/
export class Empty {}

/** Parameter properties declare fields of the class. */
export class Account {
  constructor(
    public readonly owner: string,
    protected balance?: number,
    readonly limit = 100,
    plain: string = "",
  ) {
    const local = () => plain;
  }

  /** A method next to the constructor. */
  close(): void {}
}

export class Service extends Account {
  constructor(
    @Inject(TOKEN) private readonly store: Store,
    @Optional() private cache?: Cache,
    public override owner: string = "",
    override readonly limit = 1,
    override name: string = "",
    override extra?: string,
    @Inject(KEY) plain: string = "",
    readonly callback = () => 1,
  ) {
    super(owner);
  }
}
