public class ImplicitCleanup {
    public static int trace;
    public static boolean throwTry;
    public static boolean throwCleanup;
    public static final RuntimeException TRY_FAILURE = new IllegalArgumentException("try");
    public static final RuntimeException CLEANUP_FAILURE = new IllegalStateException("cleanup");

    private static int mark(int digit) {
        trace = trace * 10 + digit;
        return digit;
    }

    private static void cleanup() {
        trace = trace * 10 + 9;
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
            return mark(2);
        } finally {
            cleanup();
        }
    }
}
