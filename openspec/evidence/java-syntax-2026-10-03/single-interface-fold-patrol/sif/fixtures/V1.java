public class V1 {
    interface StrFn { String apply(String s); }
    static String go(StrFn f) { return f == null ? "n" : f.apply("hi"); }
    public static void main(String[] a) { System.out.println(go(null)); System.out.println("hi!"); }
}
