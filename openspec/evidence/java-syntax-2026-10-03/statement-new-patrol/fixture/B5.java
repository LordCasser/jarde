public class B5 {
    private int a;
    private int b = 10;
    { a = b + 1; System.out.println("init1:" + a); }
    { b = a * 2; }
    B5() { System.out.println("ctor:" + a + ":" + b); }
    B5(int x) { this(); System.out.println("ctor2:" + x); }
    static class Sub extends B5 {
        { System.out.println("sub-init"); }
        Sub() { super(5); }
    }
    public static void main(String[] args) {
        new B5();
        new B5(7);
        new Sub();
    }
}
