public class A12 {
    static class Nested { }
    public static String topLevel() { return A12.class.getSimpleName(); }
    public static String nestedLit() { return Nested.class.getSimpleName(); }
    public static String nestedRecv() { return A12.Nested.class.getName(); }
    public static void main(String[] a) { System.out.println(topLevel()); System.out.println(nestedLit()); }
}
class Top2 { }
