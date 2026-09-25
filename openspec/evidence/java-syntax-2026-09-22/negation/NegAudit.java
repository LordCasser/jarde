public class NegAudit {
    public static int mixed(int x) { return -(-x) * -(x - 1); }
    public static long longMin() { long x = Long.MIN_VALUE; return -x; }
    public static String concat(int x) { return "v=" + -x; }
    public static Integer boxed(int x) { return Integer.valueOf(-x); }
    public static int branch(int x) { if (-x > 0) return 1; return 2; }
    public static int loop(int x) { while (-x < 0) x--; return x; }
    public static double floating(double x, double y) { return -(x + y) / -x; }
    public static int stored(int x) { int old = -x; x++; return old + x; }
}
