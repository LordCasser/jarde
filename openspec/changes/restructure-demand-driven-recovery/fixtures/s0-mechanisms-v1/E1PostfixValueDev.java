public class E1PostfixValueDev {
    static String trace = "";
    static int take(int a, int b) { trace += "t"; return a * 10 + b; }
    public static String run(int n) {
        trace = "";
        int x = n, old = x++, now = ++x;
        int[] a = new int[2];
        int stored = (a[0] = take(old, x++));
        int sum = (a[1] = x--) + stored;
        return old + ":" + now + ":" + x + ":" + sum + ":" + trace;
    }
}
