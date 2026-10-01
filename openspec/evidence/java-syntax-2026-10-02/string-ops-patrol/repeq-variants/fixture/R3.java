public class R3 {
    // Variant 3: comparison result stored into a local, then passed as the boolean argument.
    public static String ops(String s) {
        StringBuilder b = new StringBuilder();
        String t = s + "x";
        boolean r = t == s;
        b.append(r);
        return b.toString();
    }
    public static void main(String[] a) {
        System.out.println(ops("abcd"));
        System.out.println(ops("banana"));
    }
}
