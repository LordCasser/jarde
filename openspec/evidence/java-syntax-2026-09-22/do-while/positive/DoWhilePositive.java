public class DoWhilePositive {
    private static int trace;
    private static int checks;

    public static void reset() {
        trace = 0;
        checks = 0;
    }

    public static int trace() {
        return trace;
    }

    public static int checks() {
        return checks;
    }

    public static int basic(int limit) {
        int i = 0;
        do {
            i++;
            trace = trace * 10 + i;
        } while (i < limit);
        return trace;
    }

    private static int touch() {
        checks++;
        return 0;
    }

    public static int effectfulCondition(int limit) {
        int i = 0;
        do {
            i++;
            trace = trace * 10 + i;
        } while (touch() + i < limit);
        return trace;
    }
}
