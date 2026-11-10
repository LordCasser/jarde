public class E1PostfixValueDevB {
    static String trace = "";
    static int take(int a, int b) { trace += "t"; return a * 10 + b; }
    public static String run(int n) {
        trace = "";
        trace += "B";
        int x = n, old = x++, now = ++x;
        int[] a = new int[2];
        int stored = (a[0] = take(old, x++));
        int sum; { sum = (a[1] = x--) + stored + stored; }
        trace += "b";
        return old + ":" + now + ":" + x + ":" + sum + ":" + trace;
    }
}
