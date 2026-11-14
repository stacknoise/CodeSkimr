/* A licence header is no documentation. */
package com.example.shop;

import java.util.List;

/**
 * A shopping cart.
 *
 * <p>Holds {@link Item}s.
 *
 * @author nobody
 */
public class Cart<T extends Item> extends Base implements Priced, Comparable<Cart<T>> {
    /** Upper limit of items. */
    public static final int MAX_ITEMS = 100;
    static final String NAME = "cart";
    final static long SEED = 7L;
    public static final int[] PRIMES = {2, 3, 5};
    private final List<T> items;
    protected int a, b;
    int packageLevel;

    /** Creates an empty cart. */
    public Cart() {
        this.items = null;
    }

    Cart(List<T> items) {
        this.items = items;
    }

    /**
     * Adds an item.
     *
     * @param item the item
     */
    @Override
    public void add(T item) {
        items.add(item);
    }

    private static int count(List<?> list) {
        return list.size();
    }

    abstract static class Inner {
        abstract void run();
    }

    interface Priced {
        int LIMIT = 3;

        /** The price. */
        double price();

        default String label() {
            return "";
        }

        private void hidden() {}
    }

    @interface Marker {
        String value() default "";
    }

    /** Colours. */
    enum Colour {
        /** Red. */
        RED,
        GREEN("g") {
            @Override
            void paint() {}
        };

        private final String code;

        Colour() {
            this("");
        }

        Colour(String code) {
            this.code = code;
        }

        void paint() {}
    }

    /** A point. */
    public record Point(int x, int y) {
        public Point {
            if (x < 0) {
                throw new IllegalArgumentException();
            }
        }

        double length() {
            return Math.sqrt(x * x + y * y);
        }
    }

    void local() {
        Runnable r = new Runnable() {
            @Override
            public void run() {}
        };
        class Local {}
    }
}
