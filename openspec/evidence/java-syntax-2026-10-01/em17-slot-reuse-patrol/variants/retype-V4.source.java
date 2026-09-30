public class V4 {
    public static Object loop(int n) {
        Object r = new int[]{1, 2, 3};
        for (int i = 0; i < n; i++) {
            r = new boolean[]{true};
        }
        return r;
    }
    public static void main(String[] x) { System.out.println(loop(0).getClass().getName()); }
}
