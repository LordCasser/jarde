public class C1OverloadedRelayHoldout {
    static String trace = "";
    interface Convert<T> { T convert(int n); }
    static <T> T relay(Convert<T> c, int n) { trace += "r"; return c.convert(n); }
    static String display(CharSequence s) { trace += "c"; return "text=" + s; }
    static String display(Number n) { trace += "n"; return "num=" + n.intValue(); }
    static String text(int n) { return "x" + n; }
    static Integer number(int n) { return n * 3; }
    public static String run(int n) {
        trace = "";
        String a = display(relay(C1OverloadedRelayHoldout::text, n));
        String b = display(relay(C1OverloadedRelayHoldout::number, n));
        return a + ":" + b + ":" + trace;
    }
}
