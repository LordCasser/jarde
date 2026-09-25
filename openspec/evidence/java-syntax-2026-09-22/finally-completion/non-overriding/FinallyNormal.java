public class FinallyNormal {
    public static int trace;
    public static boolean failAtOne;
    public static final RuntimeException FAILURE = new IllegalArgumentException("mark-1");

    private static int mark(int digit) {
        trace = trace * 10 + digit;
        if (digit == 1 && failAtOne) {
            throw FAILURE;
        }
        return digit;
    }

    public static int run() {
        try {
            return mark(1);
        } finally {
            mark(2);
        }
    }
}
