public class V3 {
    public static Object join(boolean c) {
        Object r;
        if (c) {
            r = new int[]{1, 2, 3};
        } else {
            r = new boolean[]{true};
        }
        return r;
    }
    public static void main(String[] x) { System.out.println(join(true).getClass().getName()); }
}
