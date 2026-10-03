public class V4 {
    class Inner { int v() { return 5; } }
    public static void main(String[] a) { System.out.println(new V4().new Inner().v()); }
}
