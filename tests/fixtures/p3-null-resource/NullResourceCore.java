/** Minimal null-valued resource case; javac's handler still contains a null guard. */
public final class NullResourceCore implements AutoCloseable {
    private static int bodyCalls;
    private static int closes;

    public static void reset() {
        bodyCalls = 0;
        closes = 0;
    }

    public static int bodyCalls() {
        return bodyCalls;
    }

    public static int closes() {
        return closes;
    }

    public static void useNullResource() throws Exception {
        try (NullResourceCore resource = null) {
            bodyCalls++;
        }
    }

    @Override
    public void close() {
        closes++;
    }
}
