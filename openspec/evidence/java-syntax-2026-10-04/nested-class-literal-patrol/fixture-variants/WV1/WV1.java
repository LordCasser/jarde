public class WV1 {
    static class Nested { }
    public static String elem() { return Nested.class.getSimpleName(); }
    public static String array() { return Nested[].class.getName(); }
    public static int arrayLen() { return Nested[].class.getName().length(); }
    public static void main(String[] a) {
        System.out.println(elem());
        System.out.println(array());
        System.out.println(arrayLen());
    }
}
