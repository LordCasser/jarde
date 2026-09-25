public class ConditionalBoundarySwitch {
    private static int trace;

    private static int left() {
        trace = trace * 10 + 1;
        return 101;
    }

    private static int middle() {
        trace = trace * 10 + 2;
        return 202;
    }

    private static int fallback() {
        trace = trace * 10 + 3;
        return 303;
    }

    public static void reset() {
        trace = 0;
    }

    public static int trace() {
        return trace;
    }

    public static int choose(int selector) {
        int value;
        switch (selector) {
            case 0:
                value = left();
                break;
            case 1:
                value = middle();
                break;
            default:
                value = fallback();
                break;
        }
        return value;
    }
}
