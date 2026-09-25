public final class BoundaryEffects {
    static String trace = "";
    static String next(String value) { trace += value; return value; }
    static void independent() { trace += "X" + BoundaryProbe.SECOND; }
    static boolean choose() { trace += "C"; return true; }
    private BoundaryEffects() {}
}
