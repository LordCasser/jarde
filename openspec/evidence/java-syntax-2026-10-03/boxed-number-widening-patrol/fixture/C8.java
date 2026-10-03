public class C8 {
    <T> T pick(T a, T b) { return a; }
    static <T extends Number> T larger(T a, T b) { return a.doubleValue() >= b.doubleValue() ? a : b; }
    String useWitness() { return this.<String>pick("x", "y"); }
    static Integer boxedTern(boolean c) { return c ? Integer.valueOf(1) : 2; }
    static String loopBuilder(int n) {
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < n; i++) { sb.append(i % 2 == 0 ? "e" : "o"); if (sb.length() > 6) break; }
        return sb.toString();
    }
    public static void main(String[] a) {
        System.out.println(new C8().useWitness());
        System.out.println(boxedTern(true) + ":" + boxedTern(false));
        System.out.println(larger(3, 7));
        System.out.println(loopBuilder(9));
    }
}
