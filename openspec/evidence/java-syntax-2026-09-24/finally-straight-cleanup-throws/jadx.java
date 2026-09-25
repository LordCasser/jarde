package defpackage;

/* JADX INFO: loaded from: FinallyStraightThrow.class */
public class FinallyStraightThrow {
    public static int trace;
    public static boolean failTry;
    public static boolean failCleanup;
    public static final RuntimeException TRY_FAILURE = new IllegalArgumentException("try");
    public static final RuntimeException CLEANUP_FAILURE = new IllegalStateException("cleanup");

    private static int value() {
        trace = (trace * 10) + 1;
        if (failTry) {
            throw TRY_FAILURE;
        }
        return 41;
    }

    private static void cleanup() {
        trace = (trace * 10) + 2;
        if (failCleanup) {
            throw CLEANUP_FAILURE;
        }
    }

    public static int run() {
        try {
            return value();
        } finally {
            cleanup();
        }
    }
}
