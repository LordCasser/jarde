public class K1 {
    static int a = 1;
    static {
        a = a + 10;
        if (a > 5) { b = "big"; } else { b = "small"; }
    }
    static String b;
    static final int C;
    static {
        C = 42;
    }
    static int d = compute();
    static int compute() { return a + 100; }
    static int e;
    static {
        try { e = Integer.parseInt("7"); } catch (NumberFormatException ex) { e = -1; }
    }
    public static void main(String[] x) {
        System.out.println(a + ":" + b + ":" + C + ":" + d + ":" + e);
    }
}
