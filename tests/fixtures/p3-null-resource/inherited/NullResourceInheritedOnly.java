/** TWR-compatible, but AutoCloseable is inherited rather than directly declared here. */
public final class NullResourceInheritedOnly extends NullResourceBase {
    private static int bodyCalls;

    @Override public void close() { }

    public static void useNullResource() {
        try (NullResourceInheritedOnly resource = null) {
            bodyCalls++;
        }
    }

    public static int bodyCalls() { return bodyCalls; }
}
