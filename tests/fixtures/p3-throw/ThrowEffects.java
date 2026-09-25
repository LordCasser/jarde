public final class ThrowEffects {
    public static int calls;
    public static RuntimeException expected;
    public static RuntimeException failure;

    public static RuntimeException problem() {
        calls++;
        if (failure != null) {
            throw failure;
        }
        return expected;
    }

    public static void mark() {
        calls++;
    }
}
