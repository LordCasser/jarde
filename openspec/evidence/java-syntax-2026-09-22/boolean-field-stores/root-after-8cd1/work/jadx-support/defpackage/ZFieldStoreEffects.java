package defpackage;

public class ZFieldStoreEffects {
    public static int calls;

    public static int value(int value, boolean fail) {
        calls++;
        if (fail) {
            throw new IllegalStateException("producer");
        }
        return value;
    }

    public static void reset() {
        calls = 0;
    }
}
