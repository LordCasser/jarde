class VGrand {
    static class Base { int b() { return 6; } }
    static class Inner { static class Leaf extends Base { } }
    public static void main(String[] a) { System.out.println(new Base().b()); }
}
