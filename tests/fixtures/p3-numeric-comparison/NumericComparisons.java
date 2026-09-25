/**
 * Java 8 numeric comparison fixture.
 *
 * The first 21 methods are the direct long/float/double six-relation set from
 * the comparisons audit.  The call and boolean methods pin evaluation order,
 * exception propagation, and the refusal boundary around a converged boolean.
 */
public class NumericComparisons {
    public static int long_eq(long a, long b) { if (a == b) return 7; return 9; }
    public static int long_ne(long a, long b) { if (a != b) return 7; return 9; }
    public static int long_lt(long a, long b) { if (a < b) return 7; return 9; }
    public static int long_le(long a, long b) { if (a <= b) return 7; return 9; }
    public static int long_gt(long a, long b) { if (a > b) return 7; return 9; }
    public static int long_ge(long a, long b) { if (a >= b) return 7; return 9; }
    public static int long_not_lt(long a, long b) { if (!(a < b)) return 7; return 9; }

    public static int float_eq(float a, float b) { if (a == b) return 7; return 9; }
    public static int float_ne(float a, float b) { if (a != b) return 7; return 9; }
    public static int float_lt(float a, float b) { if (a < b) return 7; return 9; }
    public static int float_le(float a, float b) { if (a <= b) return 7; return 9; }
    public static int float_gt(float a, float b) { if (a > b) return 7; return 9; }
    public static int float_ge(float a, float b) { if (a >= b) return 7; return 9; }
    public static int float_not_lt(float a, float b) { if (!(a < b)) return 7; return 9; }

    public static int double_eq(double a, double b) { if (a == b) return 7; return 9; }
    public static int double_ne(double a, double b) { if (a != b) return 7; return 9; }
    public static int double_lt(double a, double b) { if (a < b) return 7; return 9; }
    public static int double_le(double a, double b) { if (a <= b) return 7; return 9; }
    public static int double_gt(double a, double b) { if (a > b) return 7; return 9; }
    public static int double_ge(double a, double b) { if (a >= b) return 7; return 9; }
    public static int double_not_lt(double a, double b) { if (!(a < b)) return 7; return 9; }

    public static int int_lt(int a, int b) { if (a < b) return 7; return 9; }
    public static boolean double_boolean(double a, double b) { return a < b; }

    private static int calls;

    public static void resetCalls() { calls = 0; }
    public static int calls() { return calls; }

    public static long left(long value) {
        calls = calls * 10 + 1;
        return value;
    }

    public static long right(long value) {
        calls = calls * 10 + 2;
        return value;
    }

    public static long throwingLeft(long value) {
        calls = calls * 10 + 1;
        throw new IllegalStateException("left");
    }

    public static long throwingRight(long value) {
        calls = calls * 10 + 2;
        throw new IllegalStateException("right");
    }

    public static int callOrder(long a, long b) {
        if (left(a) < right(b)) return 7;
        return 9;
    }

    public static int callThrowLeft(long a, long b) {
        if (throwingLeft(a) < right(b)) return 7;
        return 9;
    }

    public static int callThrowRight(long a, long b) {
        if (left(a) < throwingRight(b)) return 7;
        return 9;
    }

    // A direct same-block comparison with one zero-branch consumer.
    public static int sameBlock(long a, long b) {
        if (a < b) return 7;
        return 9;
    }

    // The comparison feeds a boolean result through two successors; this remains a negative shape.
    public static boolean booleanMerge(double a, double b) {
        return a < b;
    }
}
