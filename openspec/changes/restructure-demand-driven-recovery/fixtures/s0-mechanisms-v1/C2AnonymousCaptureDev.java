public class C2AnonymousCaptureDev {
    static String trace = "";
    interface Job { String run(); }
    static Job make(int value, boolean longForm) {
        if (longForm) return new Job() { { trace += "L"; } public String run() { trace += "l"; return "long" + value; } };
        return new Job() { { trace += "S"; } public String run() { trace += "s"; return "short" + value; } };
    }
    public static String run(int n) {
        trace = ""; final int captured = n + 5;
        Job a = make(captured, n % 2 == 0), b = make(captured + 1, n < 0);
        return a.run() + ":" + b.run() + ":" + trace;
    }
}
