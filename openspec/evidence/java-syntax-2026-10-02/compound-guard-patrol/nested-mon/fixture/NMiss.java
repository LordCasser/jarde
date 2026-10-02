public class NMiss {
    static StringBuilder log = new StringBuilder();
    public static int m(int n) {
        synchronized (NMiss.class) {
            int s = 0;
            synchronized (log) {
                for (int i = 0; i < n; i++) { s += i; }
            }
            return s;
        }
    }
    public static void main(String[] x) {
        System.out.println("loaded " + m(3));
    }
}
