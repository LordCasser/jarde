package defpackage;

/* JADX INFO: loaded from: TernaryValues.class */
public class TernaryValues {
    private static int trace;
    private static boolean failA;
    private static boolean failB;

    public static void reset(boolean z, boolean z2) {
        trace = 0;
        failA = z;
        failB = z2;
    }

    public static int trace() {
        return trace;
    }

    private static int a() {
        trace = (trace * 10) + 1;
        if (failA) {
            throw new IllegalStateException("a");
        }
        return 7;
    }

    private static int b() {
        trace = (trace * 10) + 2;
        if (failB) {
            throw new IllegalArgumentException("b");
        }
        return 11;
    }

    public static int returned(boolean z) {
        return z ? a() : b();
    }

    public static int assigned(boolean z) {
        return z ? a() : b();
    }

    public static int arithmetic(boolean z) {
        return (2 * (z ? a() : b())) + 1;
    }

    private static int add(int i, int i2) {
        trace = (trace * 10) + 3;
        return i + i2;
    }

    public static int callArgument(boolean z) {
        return add(100, z ? a() : b());
    }

    public static String reference(boolean z) {
        if (z) {
            return null;
        }
        return "value";
    }

    public static String overloadChoice(boolean z) {
        return overload(z ? null : "value");
    }

    public static String overload(String str) {
        trace = (trace * 10) + 4;
        return "string";
    }

    public static String overload(Object obj) {
        trace = (trace * 10) + 5;
        return "object";
    }

    public static int throwing(boolean z) {
        return z ? a() : b();
    }
}
