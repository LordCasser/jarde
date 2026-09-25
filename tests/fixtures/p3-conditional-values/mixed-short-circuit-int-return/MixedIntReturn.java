public final class MixedIntReturn {
    static boolean bValue;
    static boolean cValue;

    static boolean rhsB() {
        return bValue;
    }

    static boolean rhsC() {
        return cValue;
    }

    public static int value(boolean a) {
        return (a && rhsB()) || rhsC() ? 1 : 0;
    }
}
