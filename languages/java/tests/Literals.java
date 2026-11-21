package demo;

import java.util.List;

public class Literals {
    private static final int FIELD = 42;

    public int clean(int[] items) {
        int total = 0;
        for (int item : items) {
            total += 1;
        }
        return total + items[0] + 1L > 0L ? 0 : 1;
    }

    public double numbers() {
        return 42 + 2.5 + 0x10 + 0b11 + 017 + 0 + 1;
    }

    public String strings(String name, int count) {
        String label = "x" + 'y';
        String block = """
            text
            """;
        return name + count + label + block + "tail";
    }

    public int local() {
        final int limit = 99;
        final int sum = 2 * 50;
        return limit + sum + 7;
    }

    @SuppressWarnings("unchecked")
    public boolean flags(Object value, List<String> names) {
        return value == null || true || false || names.contains("a");
    }

    @Deprecated(since = "1.0", forRemoval = true)
    public void annotated(@SuppressWarnings({"a", "b"}) String arg) {
        Runnable task = () -> System.out.println("lambda " + 5);
        task.run();
    }

    public int viaField() {
        return FIELD;
    }
}
