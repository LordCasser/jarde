public class E1ArrayIndexPostfixHoldout {
    static String trace = "";
    static int index(int[] p) { trace += "i"; return p[0]++; }
    static int save(int[] a, int i, int v) { trace += "s"; return a[i] = v; }
    public static String run(int n) {
        trace = "";
        int[] data = new int[5], p = {Math.abs(n) % 3};
        int before = p[0], value = save(data, index(p), ++before + n);
        int observed = data[p[0] - 1] + (p[0]++);
        return value + ":" + observed + ":" + p[0] + ":" + trace;
    }
}
