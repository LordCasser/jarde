public class N1 {
    // Negative 1: ordering comparison (`<`, `if_icmplt` arms) at a boolean parameter — outside this
    // slice's equality family; stays refused exactly as before the change.
    static String take(boolean f) {
        return f ? "1" : "0";
    }
    public static String ops(String s) {
        return take(s.length() < 3);
    }
    public static void main(String[] a) {
        System.out.println(ops("abcd"));
        System.out.println(ops(""));
    }
}
