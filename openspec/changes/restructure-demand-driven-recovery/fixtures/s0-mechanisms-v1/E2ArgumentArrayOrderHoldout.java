public class E2ArgumentArrayOrderHoldout {
    static String trace = "";
    static int[] data;
    static int[] target() { trace += "A"; return data; }
    static int slot(int n) { trace += "K"; return n == 9 ? 9 : (n + 10) % 4; }
    static int value(int n) { trace += "V"; if (n == 4) throw new IllegalStateException(); return n * 2; }
    static int put(int[] a, int k, int v) { trace += "P"; a[k] = v; return a[k]; }
    public static String run(int n) {
        trace = ""; data = n == 0 ? null : new int[4];
        int result = put(target(), slot(n), value(n));
        data[slot(n)] += value(n);
        return result + ":" + data[(n + 10) % 4] + ":" + trace;
    }
}
