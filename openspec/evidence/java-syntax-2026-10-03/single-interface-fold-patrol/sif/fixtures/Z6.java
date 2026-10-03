public class Z6 {
    interface StrFn { String apply(String s); }
    public static void main(String[] a) { Runnable r = () -> System.out.println("ok"); r.run(); }
}
