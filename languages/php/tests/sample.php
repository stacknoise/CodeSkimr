<?php
namespace Shop;

/** An order. */
class Order extends Base implements Priced {
    const MAX = 5;
    private int $id;

    public function __construct(int $id) {
        $this->id = $id;
    }

    /** Total. */
    public function total(): int {
        if ($this->id > 0 && $this->id < 5) {
            return $this->id;
        }
        return 0;
    }
}

interface Priced { public function price(): int; }
trait Loggable { public function log() {} }
enum Kind { case A; case B; }

function helper(int $x): int {
    foreach ([1, 2] as $y) { $x += $y; }
    helper(strlen("a"));
    return $x ?: 0;
}

namespace Other;

class Second {
    public function run() {}
}
