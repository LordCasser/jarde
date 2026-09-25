public class RepeatedCopies {
    static int trace;

    static void cleanup() { trace++; }

    public static int run() {
        try {
            return trace;
        } finally {
            cleanup();
            cleanup();
        }
    }
}
