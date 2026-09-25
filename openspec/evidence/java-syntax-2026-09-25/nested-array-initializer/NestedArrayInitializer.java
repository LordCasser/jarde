public final class NestedArrayInitializer {
    static int calls;
    static int trace;

    static int element(int n) {
        calls++;
        trace = trace * 10 + n;
        return n;
    }

    static int[][] dynamic() {
        return new int[][] {
            { element(1), element(2) },
            { element(3) }
        };
    }

    static String[][] literal() {
        return new String[][] {
            { "a", "b" },
            { "c" }
        };
    }
}
