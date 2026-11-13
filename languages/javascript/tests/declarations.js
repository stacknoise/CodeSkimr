import { Base } from "./base";

/**
 * An order of a customer.
 * @param {number} id the id
 */
export class Order extends Base {
  static MAX = 100;
  #secret = 1;
  name;
  handler = () => {
    return 1;
  };

  /** Creates an order. */
  constructor(id) {
    super();
    this.id = id;
  }

  get total() {
    return this.id > 0 ? 1 : 0;
  }

  static async create(id) {
    const order = new Order(id);
    return order;
  }

  *generate() {}
}

export const LIMIT = 10;
const hidden = "x";
export const add = (a, b) => a + b;
const sub = function (a, b) {
  return a - b;
};
let counter = 0;

/** Does things. */
export function run(items) {
  function inner() {}
  for (const item of items) {
    if (item && counter) {
      counter++;
    }
  }
  return inner();
}
