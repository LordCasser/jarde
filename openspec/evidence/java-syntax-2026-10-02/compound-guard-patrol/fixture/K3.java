public class K3 {
    static int x = init();
    static int init() {
        int r = 1;
        for (int i = 0; i < 4; i++) { r = r * 2 + i % 2; }
        return r;
    }
    public static void main(String[] a) { System.out.println(x); }
}
