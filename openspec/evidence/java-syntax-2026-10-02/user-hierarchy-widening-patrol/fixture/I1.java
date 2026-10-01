public class I1 {
    interface Greet {
        default String hello(String n) { return "hi:" + n; }
        static Greet of() { return new I1.Greet() { public String name() { return "static"; } }; }
        String name();
    }
    static class En implements Greet {
        public String name() { return "en"; }
        public String hello(String n) { return "hello:" + n; }
    }
    public static String useDefault() { return new I1.Greet() { public String name() { return "anon"; } }.hello("d"); }
    public static String useOverride() { return new En().hello("d"); }
    public static String useStatic() { return I1.Greet.of().name(); }
    public static String viaInterface(Greet g, String n) { return g.hello(n); }
    public static void main(String[] a) {
        System.out.println(useDefault());
        System.out.println(useOverride());
        System.out.println(useStatic());
        System.out.println(viaInterface(new En(), "v"));
    }
}
