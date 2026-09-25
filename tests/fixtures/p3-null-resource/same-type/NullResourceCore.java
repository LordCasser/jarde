/** Ordinary null/catch and a damaged TWR-like protocol beside the frozen positive class. */
public final class NullResourceCore implements AutoCloseable {
    private static int bodyCalls;
    private static int closes;

    public static void reset() {
        bodyCalls = 0;
        closes = 0;
    }

    public static int bodyCalls() { return bodyCalls; }
    public static int closes() { return closes; }

    public static void useNullResource() throws Exception {
        try (NullResourceCore resource = null) {
            bodyCalls++;
        }
    }

    @Override public void close() { closes++; }

    public static void ordinaryNullThenCatch() {
        NullResourceCore resource = null;
        try {
            resource.toString();
        } catch (RuntimeException expected) {
            bodyCalls++;
        }
    }

    public static void closeWithoutSuppression() throws Throwable {
        NullResourceCore resource = null;
        try {
            bodyCalls++;
        } catch (Throwable primary) {
            if (resource != null) {
                try {
                    resource.close();
                } catch (Throwable ignoredCloseFailure) {
                    // Deliberately do not suppress the close failure onto primary.
                }
            }
            throw primary;
        }
        if (resource != null) {
            resource.close();
        }
    }
}
