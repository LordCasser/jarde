public final class CarriedMethodCalls {
    private static final StringBuilder TRACE = new StringBuilder();

    private static String mark(String label, String value) {
        TRACE.append(label);
        return value;
    }

    private static String combineStatic(String first, String second) {
        return first + second;
    }

    private String combineInstance(String first, String second) {
        return first + second;
    }

    public static String staticCall(boolean first, boolean second) {
        TRACE.setLength(0);
        String value = combineStatic(
            first ? mark("A", "a") : mark("B", "b"),
            second ? mark("C", "c") : mark("D", "d"));
        return TRACE.toString() + ":" + value;
    }

    public String instanceCall(boolean first, boolean second) {
        TRACE.setLength(0);
        String value = combineInstance(
            first ? mark("A", "a") : mark("B", "b"),
            second ? mark("C", "c") : mark("D", "d"));
        return TRACE.toString() + ":" + value;
    }
}
