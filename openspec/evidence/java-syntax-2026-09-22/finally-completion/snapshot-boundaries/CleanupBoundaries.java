public class CleanupBoundaries {
    public static int value;
    public static int trace;
    public static boolean failAtOne;
    public static final RuntimeException TRY_FAILURE = new IllegalArgumentException("try");
    public static final RuntimeException CLEANUP_FAILURE = new IllegalStateException("cleanup");

    private static int mark(int digit) {
        trace = trace * 10 + digit;
        if (digit == 1 && failAtOne) {
            throw TRY_FAILURE;
        }
        return digit;
    }

    public static int snapshotReturn() {
        value = 41;
        try {
            mark(1);
            return value;
        } finally {
            value = 99;
            mark(2);
        }
    }

    public static int cleanupThrowsOverReturn() {
        try {
            return mark(3);
        } finally {
            mark(4);
            throw CLEANUP_FAILURE;
        }
    }

    public static int cleanupThrowsOverTryThrow() {
        try {
            mark(5);
            throw TRY_FAILURE;
        } finally {
            mark(6);
            throw CLEANUP_FAILURE;
        }
    }
}
