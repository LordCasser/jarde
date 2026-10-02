package p;
public class VN2 {
    static class Mid { static class Leaf { public static int leaf(int x) { return x + 7; } } }
    static int deep(A.B.C c, int v) { return c.three(v); }
    static int own(VN2.Mid.Leaf l, int v) { return VN2.Mid.Leaf.leaf(v); }
    public static void main(String[] a) {
        System.out.println(deep(new A.B.C(), 4) + ":" + own(new p.VN2.Mid.Leaf(), 4));
    }
}
