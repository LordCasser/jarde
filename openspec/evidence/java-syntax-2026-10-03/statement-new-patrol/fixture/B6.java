public class B6 {
    int n;
    B6() { n = 42; }
    B6(int v) { n = v; }
    static void argless() { new B6(); }
    static void withArg() { new B6(7); }
    static void consumed() { int r = new B6(3).n; }
    static void chained() { new B6(new B6(1).n); }
    public static void main(String[] a) { argless(); withArg(); consumed(); chained(); System.out.println(new B6(9).n); }
}
