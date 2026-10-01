public class R2 {
    // Variant 2: numeric equality (`if_icmpXX`) result into a two-boolean-parameter static call.
    static String mark(boolean f, boolean g) {
        return String.valueOf(f) + g;
    }
    public static String ops(String s) {
        String t = s + "x";
        return mark(t.length() == s.length(), s.isEmpty());
    }
    public static void main(String[] a) {
        System.out.println(ops("abcd"));
        System.out.println(ops(""));
    }
}
