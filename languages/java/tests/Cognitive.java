// Fixture for the cognitive complexity and parameter counts (ADR 0048). The expected values are in
// crates/adapters/parsing/tests/java_metrics.rs.
package demo;

import java.util.List;

public class Cognitive {
    public int straight(int a) {
        return a + 1;
    }

    public int nested(int a, int b) {
        if (a > 0) {
            for (int i = 0; i < b; i++) {
                if (a > b) {
                    return 1;
                }
            }
        }
        return 0;
    }

    public int chain(int a) {
        if (a == 1) {
            return 1;
        } else if (a == 2) {
            return 2;
        } else {
            return 3;
        }
    }

    public boolean runs(boolean a, boolean b, boolean c, boolean d) {
        return (a && b && c) || d;
    }

    public int handler(String x) {
        try {
            return Integer.parseInt(x);
        } catch (NumberFormatException error) {
            if (!x.isEmpty()) {
                return 1;
            }
        }
        return 0;
    }

    public void callbacks(List<Integer> items) {
        items.forEach(x -> {
            if (x > 1) {
                System.out.println(x);
            }
        });
    }

    public int branches(int a) {
        switch (a) {
            case 1:
                return 1;
            case 2:
                return 2;
            default:
                return 0;
        }
    }

    public int loops(int a) {
        while (a > 0) {
            a--;
        }
        do {
            a++;
        } while (a < 3);
        outer:
        for (int x : new int[] {1, 2}) {
            for (int y : new int[] {3}) {
                if (x == y) {
                    continue outer;
                }
            }
        }
        return a;
    }

    public int method(int a, String b, int c, int... rest) {
        return a + c + rest.length + (b.isEmpty() ? 0 : 1);
    }
}
