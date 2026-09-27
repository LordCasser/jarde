public class FinallyOnce {
    private static int cleanupCount;

    public static void escaping() {
        cleanupCount = 0;
        try {
            throw new IllegalStateException("state");
        } finally {
            cleanupCount++;
        }
    }

    public static int count() { return cleanupCount; }
}
