public final class MixedArgumentControls {
    static boolean bValue;
    static boolean cValue;
    static final MixedArgumentControls INSTANCE = new MixedArgumentControls();

    static boolean b() {
        return bValue;
    }

    static boolean c() {
        return cValue;
    }

    static void two(boolean value, int marker) {
    }

    void take(boolean value) {
    }

    public static void many(boolean a) {
        two((a && b()) || c(), 7);
    }

    public static void instance(boolean a) {
        INSTANCE.take((a && b()) || c());
    }
}
