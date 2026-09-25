public final class MixedBooleanField {
    static boolean result;
    static boolean bValue;
    static boolean cValue;
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

    static void andOr(boolean a) {
        result = (a && b()) || c();
    }

    static void orAnd(boolean a) {
        result = (a || b()) && c();
    }
}
