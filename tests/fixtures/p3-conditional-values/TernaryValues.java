public class TernaryValues {
    private static int trace;
    private static boolean failA;
    private static boolean failB;

    public static void reset(boolean a, boolean b) {
        trace = 0;
        failA = a;
        failB = b;
    }

    public static int trace() {
        return trace;
    }

    private static int a() {
        trace = trace * 10 + 1;
        if (failA) {
            throw new IllegalStateException("a");
        }
        return 7;
    }

    private static int b() {
        trace = trace * 10 + 2;
        if (failB) {
            throw new IllegalArgumentException("b");
        }
        return 11;
    }

    public static int returned(boolean condition) {
        return condition ? a() : b();
    }

    public static int assigned(boolean condition) {
        int selected = condition ? a() : b();
        return selected;
    }

    public static int arithmetic(boolean condition) {
        return 2 * (condition ? a() : b()) + 1;
    }

    private static int add(int left, int right) {
        trace = trace * 10 + 3;
        return left + right;
    }

    public static int callArgument(boolean condition) {
        return add(100, condition ? a() : b());
    }

    public static String reference(boolean condition) {
        return condition ? null : "value";
    }

    public static String overloadChoice(boolean condition) {
        return overload(condition ? null : "value");
    }

    public static String overload(String value) {
        trace = trace * 10 + 4;
        return "string";
    }

    public static String overload(Object value) {
        trace = trace * 10 + 5;
        return "object";
    }

    public static int throwing(boolean condition) {
        return condition ? a() : b();
    }
}
