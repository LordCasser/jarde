public final class MixedBooleanLocal {
    static boolean bValue;
    static boolean cValue;
    static boolean result;
    static int bCalls;
    static int cCalls;

    static boolean b() {
        bCalls++;
        return bValue;
    }

    static boolean c() {
        cCalls++;
        return cValue;
    }

    static boolean one(boolean a) {
        boolean value = (a && b()) || c();
        result = value;
        return value;
    }
}
