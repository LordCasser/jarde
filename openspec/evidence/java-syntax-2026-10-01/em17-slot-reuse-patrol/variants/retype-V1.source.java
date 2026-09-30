public class V1 {
    static int side() { return 2; }
    public static String three() {
        StringBuilder b = new StringBuilder();
        {
            int[] a = { side(), side() + 1, side() * 2 };
            for (int v : a) { b.append(v).append(','); }
        }
        {
            boolean[] f = new boolean[3];
            f[side() - 1] = true;
            b.append(f[1]);
        }
        {
            Object[] o = new Object[]{"x", "y"};
            b.append(o[1]);
        }
        return b.toString();
    }
    public static void main(String[] x) { System.out.println(three()); }
}
