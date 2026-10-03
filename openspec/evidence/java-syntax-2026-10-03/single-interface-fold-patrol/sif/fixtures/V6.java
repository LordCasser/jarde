public class V6 {
    interface StrFn { String apply(String s); }
    static class Impl implements StrFn { public String apply(String s) { return s + "!"; } }
    public static void main(String[] a) { System.out.println(new Impl().apply("hi")); }
}
