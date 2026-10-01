public class R1 {
    // Variant 1: reference *inequality* (`!=`) result directly as the boolean argument of append.
    public static String ops(String s) {
        StringBuilder b = new StringBuilder();
        String t = s + "x";
        b.append(t != s);
        return b.toString();
    }
    public static void main(String[] a) {
        System.out.println(ops("abcd"));
        System.out.println(ops("banana"));
    }
}
