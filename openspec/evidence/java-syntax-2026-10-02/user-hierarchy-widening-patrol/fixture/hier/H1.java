public class H1 {
    interface Greet {
        default String hello(String n) { return "hi:" + n; }
        String name();
    }
    interface Other { String tag(); }
    interface Sub extends Greet { }
    static class Mid implements Greet {
        public String name() { return "mid"; }
    }
    static class TwoLevel extends Mid { }
    static class Multi implements Greet, Other {
        public String name() { return "multi"; }
        public String tag() { return "mtag"; }
    }
    static class ViaSub implements Sub {
        public String name() { return "sub"; }
    }
    static final class Fin { public String name() { return "fin"; } }
    static class Caller {
        String call(Greet g) { return "call:" + g.name(); }
    }
    static String via(Greet g, String n) { return g.hello(n); }
    static String viaOther(Other o) { return "tag:" + o.tag(); }
    static String lead(String s, Greet g) { return s + ":" + g.hello("x"); }
    static String viaObject(Object o) { return "obj"; }
    public static void main(String[] a) {
        System.out.println(via(new TwoLevel(), "two"));
        System.out.println(via(new Multi(), "multi"));
        System.out.println(viaOther(new Multi()));
        System.out.println(via(new Greet() { public String name() { return "anon"; } }, "anon"));
        System.out.println(via(new ViaSub(), "sub"));
        System.out.println(lead("lead", new Multi()));
        System.out.println(new Caller().call(new TwoLevel()));
        System.out.println(viaObject(new Fin()));
    }
}
