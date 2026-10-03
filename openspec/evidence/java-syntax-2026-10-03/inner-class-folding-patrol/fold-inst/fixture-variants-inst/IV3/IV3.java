public class IV3 {
    private int seed = 2;
    class B {
        int b() { return seed + 1; }
        class C {
            int c() { return b() + 3; }
        }
    }
    public static void main(String[] args) {
        IV3 a = new IV3();
        System.out.println(a.new B().new C().c());
    }
}
