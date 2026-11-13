class M {
    int plain() { return 1; }

    int branches(int a, int b) {
        if (a > 0 && b > 0) { return 1; }
        else if (a < 0 || b < 0) { return 2; }
        return a > b ? a : b;
    }

    void loops(int[] xs) {
        for (int i = 0; i < 3; i++) { }
        for (int x : xs) { }
        while (xs.length > 0) { break; }
        do { } while (false);
    }

    String cases(int k) {
        switch (k) {
            case 1:
            case 2:
                return "a";
            default:
                return "b";
        }
    }

    String rules(int k) {
        return switch (k) {
            case 1, 2 -> "one";
            case 3 -> "three";
            default -> "other";
        };
    }

    void catches() {
        try { work(); }
        catch (java.io.IOException e) { }
        catch (RuntimeException | Error e) { }
        finally { }
    }

    void resources() {
        try (var r = open()) { work(); }
        catch (Exception e) { }
    }

    int outer() {
        java.util.function.IntUnaryOperator f = x -> x > 0 ? x : 0;
        Object o = new Object() {
            int inner(int y) { if (y > 0) { return y; } return 0; }
        };
        return f.applyAsInt(1);
    }

    abstract int noBody();
}

interface Shape {
    int area();
}
