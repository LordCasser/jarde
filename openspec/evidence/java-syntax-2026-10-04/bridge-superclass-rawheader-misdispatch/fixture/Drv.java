public class Drv {
    public static void main(String[] a) {
        Outer.Box b = new Spec();
        b.set("x");   // erased: 有桥则 SPEC.set，无桥+裸头则 BOX.set
        b.get();
    }
}
