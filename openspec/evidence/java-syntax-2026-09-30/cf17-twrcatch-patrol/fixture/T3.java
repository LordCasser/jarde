public class T3 implements AutoCloseable {
    @Override public void close() { }
    static void touch(T3 r) { }
    public static String voidNamed() throws Exception {
        try (T3 r = new T3()) { touch(r); } catch (IllegalStateException e) { return "caught"; }
        return "done";
    }
    public static String voidNamedRecover() throws Exception {
        try (T3 r = new T3()) { touch(r); } catch (IllegalStateException e) { touch(null); }
        return "done";
    }
    public static void main(String[] a) throws Exception {
        System.out.println(voidNamed()); System.out.println(voidNamedRecover());
    }
}
