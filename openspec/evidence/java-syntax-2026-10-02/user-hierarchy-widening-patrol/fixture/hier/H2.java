public class H2 {
    interface Target { String tag(); }
    static class Ext extends Helper { }
    static class MyErr extends java.lang.Exception {
        MyErr(String message) { super(message); }
    }
    static String takeT(Target t) { return "t:" + t.tag(); }
    static void sink(java.lang.Throwable t) { System.out.println("sinkT:" + t.getMessage()); }
    public static void main(String[] a) {
        System.out.println(takeT(new Ext()));
        sink(new MyErr("m"));
    }
}
