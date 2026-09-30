public class A1 {
    static int calls = 0;
    static int n() { calls++; return 3; }
    public static int[][] dynDims() {
        int[][] a = new int[n()][];
        a[0] = new int[n()];
        return new int[][] { a[0], new int[n()] };
    }
    public static Object[] mixed() {
        return new Object[] { "s", n(), new int[n()] };
    }
    public static void main(String[] x) {
        int[][] r = dynDims();
        System.out.println(r.length + ":" + r[0].length + ":" + calls);
        System.out.println(mixed().length + ":" + calls);
    }
}
