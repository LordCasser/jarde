public class ConditionalBoundaryCases {
    static int trace;
    static boolean fail;

    static int left() { trace = trace * 10 + 1; if (fail) throw new IllegalStateException("left"); return 1; }
    static int right() { trace = trace * 10 + 2; if (fail) throw new IllegalArgumentException("right"); return 2; }

    static int loop(boolean again, int n) {
        int value = 0;
        while (again) {
            value = again ? n : -n;
            n--;
            again = n > 0;
        }
        return value;
    }

    static int guarded(boolean condition) {
        int value;
        try {
            value = condition ? left() : right();
        } catch (RuntimeException error) {
            value = 3;
        }
        return value;
    }

    static int unknownType(boolean condition) {
        return consume(condition ? new LeftValue() : new RightValue());
    }

    static int consume(MarkerValue value) { return value.value(); }
}

