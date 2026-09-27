public class SharedFinallyOtherField {
    private static int cleanupCount;
    private static int otherCount;

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

    public static int otherCount() { return otherCount; }

    public static int count() { return cleanupCount; }
}
