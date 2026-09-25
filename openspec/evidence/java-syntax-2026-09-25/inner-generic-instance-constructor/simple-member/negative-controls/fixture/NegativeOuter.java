package negative;

public final class NegativeOuter {
    public static String trace = "";
    public final class Inner {
        public final int value;
        public Inner(int value) { this.value = value; }
    }
    public static int mark(String label, int value) {
        trace += label;
        return value;
    }
}
