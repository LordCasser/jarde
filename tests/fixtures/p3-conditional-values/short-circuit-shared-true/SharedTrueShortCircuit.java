public final class SharedTrueShortCircuit {
    static boolean result;
    static int calls;
    static boolean rhs() { calls++; return true; }
    static void assign(boolean left) { result = left || rhs(); }
}
