/** Null-resource body exception case with a one-call body and caller-owned primary marker. */
public final class NullResourceExceptionalCore implements AutoCloseable {
    private static int bodyCalls;
    private static int closes;

    public static void reset() {
        bodyCalls = 0;
        closes = 0;
    }

    public static int bodyCalls() { return bodyCalls; }
    public static int closes() { return closes; }

    public static void useNullResourceException() {
        try (NullResourceExceptionalCore resource = null) {
            NullResourceThrower.raise();
        }
    }

    @Override public void close() { closes++; }
}
