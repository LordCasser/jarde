public class SharedFinallyExtraReturn {
    private static int cleanupCount;

    private static void cleanup() { cleanupCount++; }

    public static String handled(boolean fail, boolean early) {
        cleanupCount = 0;
        try {
            if (fail) throw new IllegalArgumentException("arg");
            if (early) return "early";
            return "normal";
        } catch (IllegalArgumentException error) {
            return "caught";
        } finally {
            cleanup();
        }
    }

    public static int count() { return cleanupCount; }
}
