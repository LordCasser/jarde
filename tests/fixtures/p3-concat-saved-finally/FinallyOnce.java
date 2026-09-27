public class FinallyOnce {
    private static int cleanupCount;

    public static String handled(boolean fail) {
        cleanupCount = 0;
        try {
            if (fail) throw new IllegalArgumentException("arg");
            return "normal";
        } catch (IllegalArgumentException error) {
            return "caught:" + error.getMessage();
        } finally {
            cleanupCount++;
        }
    }

    public static int count() { return cleanupCount; }
}
