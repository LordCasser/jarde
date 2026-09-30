public class Wtyping implements AutoCloseable {
    public static boolean boom = false;
    @Override public void close() { if (boom) { throw new IllegalStateException("close"); } }
    static void touch(Wtyping r) { if (boom) { throw new IllegalStateException("touch"); } }
    public static String stringLiteral() throws Exception {
        try (Wtyping r = new Wtyping()) { touch(r); return "in"; }
    }
    public static int intLiteral() throws Exception {
        try (Wtyping r = new Wtyping()) { touch(r); return 42; }
    }
    public static StringBuilder constructorValue() throws Exception {
        try (Wtyping r = new Wtyping()) { touch(r); return new StringBuilder("built"); }
    }
    public static String callReturn() throws Exception {
        try (Wtyping r = new Wtyping()) { touch(r); return String.valueOf(7); }
    }
    public static String nullValue() throws Exception {
        try (Wtyping r = new Wtyping()) { touch(r); return null; }
    }
    public static Class<?> classLiteral() throws Exception {
        try (Wtyping r = new Wtyping()) { touch(r); return String.class; }
    }
}
