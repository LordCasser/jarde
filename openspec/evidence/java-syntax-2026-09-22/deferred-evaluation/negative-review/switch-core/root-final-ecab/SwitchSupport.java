public class SwitchSupport {
    public static int trace;
    public static int mode;
    public static final RuntimeException FAILURE = new IllegalStateException("chosen");

    public static int value() {
        trace = trace * 10 + 1;
        if (mode == 1) {
            throw FAILURE;
        }
        return 101;
    }

    public static int other() {
        trace = trace * 10 + 3;
        if (mode == 1) {
            throw FAILURE;
        }
        return 303;
    }

    public static void mark() {
        trace = trace * 10 + 2;
        if (mode == 2) {
            throw FAILURE;
        }
    }
}
