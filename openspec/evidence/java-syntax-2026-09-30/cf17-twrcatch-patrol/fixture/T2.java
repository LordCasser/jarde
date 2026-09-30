public class T2 implements AutoCloseable {
    @Override public void close() { }
    static void touch(T2 r) { }
    public static String voidBody() throws Exception {
        try (T2 r = new T2()) { touch(r); }
        return "done";
    }
    public static String voidBodyReturnInside() throws Exception {
        try (T2 r = new T2()) { touch(r); return "in"; }
    }
    public static String popBody() throws Exception {
        try (T2 r = new T2()) { r.toString(); }
        return "done";
    }
    public static String popBodyVoidTouch() throws Exception {
        try (T2 r = new T2()) { r.hashCode(); touch(r); }
        return "done";
    }
    public static void main(String[] a) throws Exception {
        System.out.println(voidBody()); System.out.println(voidBodyReturnInside()); System.out.println(popBody()); System.out.println(popBodyVoidTouch());
    }
}
