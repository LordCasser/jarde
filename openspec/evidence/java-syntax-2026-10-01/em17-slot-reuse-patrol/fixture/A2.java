public class A2 {
    static int side() { return 2; }
    public static String fillCalc() {
        int[] a = { side(), side() + 1, side() * 2 };
        StringBuilder b = new StringBuilder();
        for (int v : a) { b.append(v).append(','); }
        boolean[] f = new boolean[3];
        f[side() - 1] = true;
        return b.toString() + f[1];
    }
    public static void main(String[] x) { System.out.println(fillCalc()); }
}
