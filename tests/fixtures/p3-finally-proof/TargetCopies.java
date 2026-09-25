public class TargetCopies {
    static int trace;

    static void first() { trace = trace * 10 + 1; }
    static void second() { trace = trace * 10 + 2; }

    public static int run() {
        try {
            return trace;
        } finally {
            first();
        }
    }

    public static void anchor() { second(); }
}
