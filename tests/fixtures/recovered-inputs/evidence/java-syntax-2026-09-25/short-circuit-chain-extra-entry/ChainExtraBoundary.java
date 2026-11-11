public final class ChainExtraBoundary {
    static boolean result;
    static boolean rhsValue;
    static int calls;

    static boolean rhs() {
        calls++;
        return rhsValue;
    }

    static void assign(boolean gate, boolean extra, boolean left, boolean other) {
        result = (gate ? extra : left) || other || rhs();
    }
}
