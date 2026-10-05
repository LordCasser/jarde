public class ST {
    private static int sc;
    private static String sr;
    class S {
        void w(int v) { sc = v; }
        void wr(String v) { sr = v; }
    }
    public static void main(String[] a) {
        ST o = new ST(); S s = o.new S();
        s.w(7); s.wr("q");
        System.out.println(sc + "|" + sr);
    }
}
