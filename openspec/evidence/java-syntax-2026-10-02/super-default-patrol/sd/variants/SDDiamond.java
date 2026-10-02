public class SDDiamond {
    interface A {
        default String name() { return "A"; }
        default void log(String message) { System.out.println("log:" + message); }
        default String greet(int count) { return "g" + count; }
    }
    interface B { default String name() { return "B"; } }
    static class Use implements A, B {
        @Override public String name() { return A.super.name() + B.super.name(); }
        String mixed() { return A.super.name() + "!"; }
        void logCall() { A.super.log("hi"); }
        String greetCall() { return A.super.greet(7); }
    }
    public static void main(String[] x) {
        Use use = new Use();
        System.out.println(use.name());
        System.out.println(use.mixed());
        use.logCall();
        System.out.println(use.greetCall());
    }
}
