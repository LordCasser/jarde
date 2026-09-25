public final class BoundaryProbe {
    public static int trace;

    public static int mixedLeft(int a, int b, long tag) {
        return a & b;
    }

    public static int mixedRight(int a, int b, byte tag) {
        return a | b;
    }

    public static int pureIntegerLiterals(int a, int b) {
        return (a & 1) | (b ^ 0);
    }

    public static int overwrittenLocal(int old, int right) {
        int saved = old;
        old = 31;
        return saved & right;
    }

    public static int observeOnce(int a, int b) {
        int saved = a & b;
        observe(saved);
        return saved;
    }

    public static int observe(int value) {
        trace = trace * 31 + value;
        return value;
    }

    public static int seedObserve(int value) {
        return observe(value);
    }
}
