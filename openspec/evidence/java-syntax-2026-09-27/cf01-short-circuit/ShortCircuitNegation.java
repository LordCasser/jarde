package probe;

public final class ShortCircuitNegation {
    private static int calls;
    private static String trace = "";

    private static boolean probe(char label, boolean value) {
        calls++;
        trace += label;
        return value;
    }

    public static boolean logicalAnd(boolean a, boolean b) {
        return probe('a', a) && probe('b', b);
    }

    public static boolean logicalOr(boolean a, boolean b) {
        return probe('a', a) || probe('b', b);
    }

    public static boolean negatedAnd(boolean a, boolean b) {
        return !(probe('a', a) && probe('b', b));
    }

    public static boolean negatedOr(boolean a, boolean b) {
        return !(probe('a', a) || probe('b', b));
    }

    public static boolean conditions(boolean a, boolean b, boolean c) {
        return (probe('a', a) && probe('b', b)) || probe('c', c);
    }

    public static boolean negatedConditions(boolean a, boolean b, boolean c) {
        return !((probe('a', a) && probe('b', b)) || probe('c', c));
    }

    public static boolean nested(boolean a, boolean b, boolean c) {
        return probe('a', a) && (!probe('b', b) || probe('c', c));
    }

    public static boolean negated(boolean a, boolean b, boolean c) {
        return !(probe('a', a) && (probe('b', b) || !probe('c', c)));
    }

    public static void reset() {
        calls = 0;
        trace = "";
    }

    public static String observation() {
        return calls + ";" + trace;
    }
}
