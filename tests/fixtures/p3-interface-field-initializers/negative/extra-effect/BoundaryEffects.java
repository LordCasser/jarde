public final class BoundaryEffects {
    static String trace = "";
    static int count;

    private BoundaryEffects() {}

    static String next(String value) {
        trace += value;
        return value;
    }

    static void independent() {
        trace += "X" + BoundaryProbe.SECOND;
    }
}
