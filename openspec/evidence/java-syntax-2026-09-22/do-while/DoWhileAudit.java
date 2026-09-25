public class DoWhileAudit {
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

    public static int withContinue(int limit) {
        int i = 0;
        do {
            i++;
            if (i == 2) {
                continue;
            }
            trace = trace * 10 + i;
        } while (i < limit);
        return trace;
    }

    public static int withBreak(int limit) {
        int i = 0;
        do {
            i++;
            if (i == 3) {
                break;
            }
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
        } while ((touch() + i) < limit);
        return trace;
    }
}
