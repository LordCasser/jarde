public class S1 implements AutoCloseable {
    @Override public void close() { }
    static String tag(IllegalStateException e) { return "tag:" + e.getMessage(); }
    static void touch(S1 r) { throw new IllegalStateException("boom"); }
    public static String twrHelper() throws Exception {
        try (S1 r = new S1()) { touch(r); } catch (IllegalStateException e) { return tag(e); }
        return "done";
    }
    public static String plainHelper() throws Exception {
        try { touch(null); } catch (IllegalStateException e) { return tag(e); }
        return "done";
    }
    public static void main(String[] a) throws Exception {
        System.out.println(twrHelper()); System.out.println(plainHelper());
    }
}
