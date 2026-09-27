public class SharedFinallyOtherTarget {
    private static int cleanupCount;

    private static void cleanup() { cleanupCount++; }
    private static void other() { cleanupCount += 2; }
    private static void retainOther() { other(); }

    public static String handled(boolean fail) {
        cleanupCount = 0;
        try {
            if (fail) throw new IllegalArgumentException("arg");
            return "normal";
        } catch (IllegalArgumentException error) {
            return "caught";
        } finally {
            cleanup();
        }
    }

    public static int count() { return cleanupCount; }
}
