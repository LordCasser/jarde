public class NestedSuper {
    interface A {
        default String name() { return "A"; }
        default void log(String message) { System.out.println("log:" + message); }
        default String greet(int count) { return "g" + count; }
    }
    interface B { default String name() { return "B"; } }
    interface Sub extends A { }
    interface Abs { String name(); }
    static class Diamond implements A, B {
        @Override public String name() { return A.super.name() + B.super.name(); }
    }
    static class Single implements A {
        @Override public String name() { return "I:" + A.super.name(); }
        void runLog() { A.super.log("hi"); }
        String runGreet() { return A.super.greet(7); }
    }
    static class Indirect implements A {
        static Sub probe;
        @Override public String name() { return A.super.name(); }
    }
    static class Abstract implements Abs, A {
        @Override public String name() { return A.super.name(); }
    }
    public static void main(String[] x) {
        System.out.println(new Diamond().name());
        System.out.println(new Single().name());
        new Single().runLog();
        System.out.println(new Single().runGreet());
        System.out.println(new Indirect().name());
        System.out.println(new Abstract().name());
    }
}
