public class A10 {
    static String nameOf(Class<?> c) { return c.getSimpleName(); }
    public static String asArg() { return nameOf(A10.class); }
    public static String asReceiver() { return A10.class.getName(); }
    public static String asReceiver2() { return A10.class.getSimpleName() + "!"; }
    public static int asReceiverLen() { return A10.class.getName().length(); }
    public static String viaLocal() { Class<?> c = A10.class; return c.getSimpleName(); }
    public static void main(String[] a) {
        System.out.println(asArg()); System.out.println(asReceiver());
        System.out.println(asReceiver2()); System.out.println(asReceiverLen());
        System.out.println(viaLocal());
    }
}
