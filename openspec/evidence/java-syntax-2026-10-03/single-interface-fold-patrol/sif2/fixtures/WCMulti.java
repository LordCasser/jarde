// Variant 1 (multi-interface method call family): two interfaces, each called through its own
// static method, plus one abstract interface method implemented by a static class child — every
// external reference to a folded member is an interface method reference.
public class WCMulti {
    interface A { static int sv() { return 8; } default int dv() { return 3; } }
    interface B { static int sv() { return 9; } int iv(); }
    static class Impl implements B { public int iv() { return 4; } }
    static B held = new Impl();
    public static void main(String[] a) {
        System.out.println(A.sv() + B.sv());
        System.out.println(held.iv());
    }
}
