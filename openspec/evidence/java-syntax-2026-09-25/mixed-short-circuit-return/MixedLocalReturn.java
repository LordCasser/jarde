public final class MixedLocalReturn {
    static boolean bValue;
    static boolean cValue;
    static int bCalls;
    static int cCalls;

    static boolean rhsB() {
        bCalls++;
        return bValue;
    }

    static boolean rhsC() {
        cCalls++;
        return cValue;
    }

    public static boolean value(boolean a) {
        return (a && rhsB()) || rhsC();
    }
}
