public class ExceptionOnlyLocalChanged {
    public static int trace;
    private static RuntimeException replacement = new IllegalStateException("replacement");
    private static void mark(int digit) { trace = trace * 10 + digit; }
    public static int run(RuntimeException pending) {
        try {
            mark(7);
            pending = replacement;
            throw pending;
        } finally {
            mark(8);
        }
    }
}
