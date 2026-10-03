public class Q4 {
    static class Solo { int v() { return 7; } }
    public static void main(String[] a) { Runnable r = () -> {}; r.run(); System.out.println(new Solo().v()); }
}
