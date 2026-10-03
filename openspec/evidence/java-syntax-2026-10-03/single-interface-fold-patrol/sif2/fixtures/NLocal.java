public class NLocal {
    interface Fn { String apply(String s); }
    static String use(Fn f) { Fn g = f; return g == null ? "n" : g.apply("hi"); }
    public static void main(String[] a) { System.out.println(use(null)); }
}
