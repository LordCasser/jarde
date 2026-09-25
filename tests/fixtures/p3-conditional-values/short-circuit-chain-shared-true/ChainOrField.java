public final class ChainOrField {
    static boolean result;
    static boolean rhsValue;
    static int calls;

    static boolean rhs() {
        calls++;
        return rhsValue;
    }

    static void assign(boolean extra, boolean left) {
        result = extra || left || rhs();
    }
}
