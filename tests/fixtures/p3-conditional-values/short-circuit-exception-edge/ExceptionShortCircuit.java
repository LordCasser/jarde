public final class ExceptionShortCircuit {
    static boolean result;
    static int calls;

    static boolean mayThrow() {
        calls++;
        throw new IllegalStateException("rhs");
    }

    static boolean assign(boolean left) {
        try {
            result = left && mayThrow();
        } catch (RuntimeException ex) {
            return result;
        }
        return result;
    }
}
