public final class BoundaryEffects {
    static String trace = "";

    private BoundaryEffects() {}

    static int value() {
        trace += "L";
        return 9;
    }
}
