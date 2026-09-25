public final class NullCatchOnly implements AutoCloseable {
    private static int catches;

    public static int catches() {
        return catches;
    }

    public static void ordinary() {
        NullCatchOnly resource = null;
        try {
            resource.toString();
        } catch (RuntimeException expected) {
            catches++;
        }
    }

    @Override
    public void close() {}
}
