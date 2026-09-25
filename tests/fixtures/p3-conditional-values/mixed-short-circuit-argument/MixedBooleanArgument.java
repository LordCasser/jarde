public final class MixedBooleanArgument {
    static boolean bValue;
    static boolean cValue;
    static boolean result;
    static int bCalls;
    static int cCalls;
    static int sinkCalls;

    static boolean b() {
        bCalls++;
        return bValue;
    }

    static boolean c() {
        cCalls++;
        return cValue;
    }

    static void sink(boolean value) {
        sinkCalls++;
        result = value;
    }

    public static void call(boolean a) {
        sink((a && b()) || c());
    }
}
