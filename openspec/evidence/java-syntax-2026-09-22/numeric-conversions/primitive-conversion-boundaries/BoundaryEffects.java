public class BoundaryEffects {
    public static int trace;

    public static int next() {
        trace = trace * 10 + 7;
        return 16777217;
    }

    public static void discard(byte value) {
        trace = trace * 10 + 9;
    }
}
