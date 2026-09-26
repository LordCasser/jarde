public final class NestedConditional {
    private NestedConditional() {}

    public static int nested(int a) {
        int t = a > 10 ? (a > 100 ? 3 : 2) : 1;
        return t;
    }

    public static int nestedEffects(int a, StringBuilder events) {
        int t = a > outerLimit(a, events)
                ? (a > innerLimit(a, events)
                    ? arm(events, "3", 3, false)
                    : arm(events, "2", 2, a == 12))
                : arm(events, "1", 1, false);
        return t;
    }

    private static int outerLimit(int a, StringBuilder events) {
        events.append('O');
        if (a == 0) {
            throw new IllegalArgumentException("outer");
        }
        return 10;
    }

    private static int innerLimit(int a, StringBuilder events) {
        events.append('I');
        if (a == 50) {
            throw new IllegalStateException("inner");
        }
        return 100;
    }

    private static int arm(StringBuilder events, String label, int value, boolean fail) {
        events.append(label);
        if (fail) {
            throw new ArithmeticException("arm-" + label);
        }
        return value;
    }
}
