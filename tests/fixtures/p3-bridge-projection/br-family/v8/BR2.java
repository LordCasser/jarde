public class BR2 {
    interface Node2 { Node2 next(); }
    interface Mid extends Node2 {}
    static class Base2 implements Mid {
        public Base2 next() { return new Base2(); }
    }
    public static void main(String[] a) {
        Mid m = new Base2();
        System.out.println(m.next().getClass().getSimpleName());
        Node2 n = m;
        System.out.println(n.next().getClass().getSimpleName());
    }
}
