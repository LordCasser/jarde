public final class ChainOrFieldDuplicatePhi {
    static boolean result;
    static boolean mirror;
    static boolean rhsValue;
    static int calls;

    static boolean rhs() {
        calls++;
        return rhsValue;
    }

    static void assign(boolean extra, boolean left) {
        result = mirror = extra || left || rhs();
    }
}
