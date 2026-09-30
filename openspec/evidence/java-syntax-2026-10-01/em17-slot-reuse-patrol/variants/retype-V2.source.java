public class V2 {
    public static int sameType() {
        int s = 0;
        {
            int[] a = new int[2];
            a[0] = 1;
            s += a[0];
        }
        {
            int[] c = new int[3];
            c[1] = 2;
            s += c[1];
        }
        return s;
    }
    public static void main(String[] x) { System.out.println(sameType()); }
}
