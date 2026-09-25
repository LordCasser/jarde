public final class AssertProbe {
    public static int conditionCalls;
    public static int messageCalls;
    public static int trace;

    private AssertProbe() {
    }

    public static void reset() {
        conditionCalls = 0;
        messageCalls = 0;
        trace = 0;
    }

    public static boolean condition(boolean result) {
        conditionCalls++;
        trace = trace * 10 + 1;
        return result;
    }

    public static boolean conditionThrows(RuntimeException sentinel) {
        conditionCalls++;
        trace = trace * 10 + 1;
        throw sentinel;
    }

    public static Object message(String text) {
        messageCalls++;
        trace = trace * 10 + 2;
        return text;
    }

    public static Object messageThrows(RuntimeException sentinel) {
        messageCalls++;
        trace = trace * 10 + 2;
        throw sentinel;
    }

    public static void succeeds() {
        assert condition(true) : message("success-message");
    }

    public static void fails() {
        assert condition(false) : message("failure-message");
    }

    public static void conditionException(RuntimeException sentinel) {
        assert conditionThrows(sentinel) : message("unreachable-message");
    }

    public static void messageException(RuntimeException sentinel) {
        assert condition(false) : messageThrows(sentinel);
    }
}
