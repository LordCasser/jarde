public class N2 {
    static class Outer { static class Mid { static class Leaf { } } }
    public static String multiLevel() { return Outer.Mid.Leaf.class.getSimpleName(); }
    public static String midLevel() { return Outer.Mid.class.getName(); }
    static String recvChain() { return Outer.Mid.Leaf.class.getEnclosingClass().getSimpleName(); }
    public static void main(String[] a) {
        System.out.println(multiLevel()); System.out.println(midLevel()); System.out.println(recvChain());
    }
}
