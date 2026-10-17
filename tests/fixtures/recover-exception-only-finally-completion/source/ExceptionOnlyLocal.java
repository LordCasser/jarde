public class ExceptionOnlyLocal {
    public static int trace;

    public ExceptionOnlyLocal() {
    }

    private static void mark(int digit) {
        trace = trace * 10 + digit;
    }

    public static int run(RuntimeException pending) {
        try {
            mark(7);
            throw pending;
        } finally {
            mark(8);
        }
    }
}
