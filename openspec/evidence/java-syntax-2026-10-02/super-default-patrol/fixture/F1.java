public class F1 {
    interface A { default String name() { return "A"; } }
    interface B { default String name() { return "B"; } }
    static class Diamond implements A, B {
        @Override public String name() { return A.super.name() + B.super.name(); }
    }
    static class Reabstract implements A {
        interface C extends A { @Override String name(); }
        static class Impl implements A {
            @Override public String name() { return "I:" + A.super.name(); }
        }
    }
    static String diamond() { return new Diamond().name(); }
    static String reabstract() { return new Reabstract.Impl().name(); }
    public static void main(String[] x) {
        System.out.println(diamond());
        System.out.println(reabstract());
    }
}
