public class S1 {
    public static String ops(String s) {
        StringBuilder b = new StringBuilder();
        b.append(s.charAt(0));
        b.append(s.substring(1, 3));
        b.append(s.indexOf('a'));
        b.append(s.length());
        b.append(s.toUpperCase());
        b.append(String.valueOf(42));
        b.append(String.format("%s-%d", s, 7));
        String t = s + "x" + 9;
        b.append(t.replace('a', 'b'));
        b.append(t.equals(s));
        b.append(t == t.intern());
        return b.toString();
    }
    public static void main(String[] a) {
        System.out.println(ops("abcd"));
        System.out.println(ops("banana"));
    }
}
