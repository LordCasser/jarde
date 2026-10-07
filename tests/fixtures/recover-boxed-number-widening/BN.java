public class BN {
    static <T extends Number> T larger(T a, T b) { return a.doubleValue() >= b.doubleValue() ? a : b; }
    static <T extends CharSequence> T pickSeq(T a, T b) { return a.length() >= b.length() ? a : b; }
    static Number same(Number a) { return a; }
    static Number withParam(Integer a) { return larger(a, Integer.valueOf(0)); }

    public static void main(String[] a) {
        System.out.println(larger(3, 7));
        System.out.println(larger(3L, 7L));
        System.out.println(larger(3.5, 7.5));
        System.out.println(larger(3.5f, 7.5f));
        System.out.println(larger((short) 3, (short) 7));
        System.out.println(larger((byte) 3, (byte) 7));
        System.out.println(pickSeq("x", "yy"));
        System.out.println(same(Integer.valueOf(9)));
        System.out.println(withParam(Integer.valueOf(4)));
    }
}
