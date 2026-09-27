public class SharedFinally {
    private static int cleanupCount;

    public static String handled(boolean fail) {
        cleanupCount = 0;
        try {
            if (fail) throw new IllegalArgumentException("arg");
            return "normal";
        } catch (IllegalArgumentException error) {
            return "caught";
        } finally {
            cleanupCount++;
        }
    }

    public static int count() { return cleanupCount; }
}
