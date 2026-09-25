public class DoWhileCore {
    private static int trace;

    public static void reset() {
        trace = 0;
    }

    public static int trace() {
        return trace;
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
}
