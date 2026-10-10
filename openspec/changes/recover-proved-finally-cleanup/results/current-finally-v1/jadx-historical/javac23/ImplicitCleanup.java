package defpackage;

/* JADX INFO: loaded from: ImplicitCleanup.class */
public class ImplicitCleanup {
    public static int trace;
    public static boolean throwTry;
    public static boolean throwCleanup;
    public static final RuntimeException TRY_FAILURE = new IllegalArgumentException("try");
    public static final RuntimeException CLEANUP_FAILURE = new IllegalStateException("cleanup");

    private static int mark(int i) {
        trace = (trace * 10) + i;
        return i;
    }

    private static void cleanup() {
        trace = (trace * 10) + 9;
        if (throwCleanup) {
            throw CLEANUP_FAILURE;
        }
    }

    public static int run() {
        try {
            if (throwTry) {
                mark(1);
                throw TRY_FAILURE;
            }
            int iMark = mark(2);
            cleanup();
            return iMark;
        } catch (Throwable th) {
            cleanup();
            throw th;
        }
    }
}
