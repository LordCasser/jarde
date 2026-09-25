public class NarrowFieldStoreEffects {
    public static int calls;

    public static void reset() {
        calls = 0;
    }

    public static int value(int value, boolean fail) {
        calls++;
        if (fail) {
            throw new IllegalStateException("producer-failure");
        }
        return value;
    }
}
