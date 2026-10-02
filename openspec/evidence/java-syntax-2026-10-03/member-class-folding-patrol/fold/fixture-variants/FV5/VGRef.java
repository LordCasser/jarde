class VGRef {
    static class Base { int b() { return 8; } }
    static class Inner { static class Leaf extends Base { int l() { return b() + 1; } } }
    static Inner hold(Inner.Leaf leaf) { return null; }
    public static void main(String[] a) { System.out.println(new Base().b()); }
}
