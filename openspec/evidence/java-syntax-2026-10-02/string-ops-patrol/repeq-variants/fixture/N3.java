public class N3 {
    // Probe: equality against literal zero — records which branch javac lowers it to.
    static String take(boolean f) {
        return f ? "1" : "0";
    }
    public static String ops(String s) {
        return take(s.length() == 0);
    }
    public static void main(String[] a) {
        System.out.println(ops("abcd"));
        System.out.println(ops(""));
    }
}
