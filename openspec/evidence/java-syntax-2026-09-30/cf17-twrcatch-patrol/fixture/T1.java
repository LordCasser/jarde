public class T1 implements AutoCloseable {
    @Override public void close() { }
    public static String twrVoid() throws Exception {
        try (T1 r = new T1()) {
            r.hashCode();
        }
        return "done";
    }
    public static String twrPop() throws Exception {
        try (T1 r = new T1()) {
            r.toString();
        }
        return "done";
    }
    public static String twrPopNamed() throws Exception {
        try (T1 r = new T1()) {
            r.toString();
        } catch (IllegalStateException e) {
            return "caught";
        }
        return "done";
    }
    public static String twrVoidNamed() throws Exception {
        try (T1 r = new T1()) {
            r.hashCode();
        } catch (IllegalStateException e) {
            return "caught";
        }
        return "done";
    }
    public static void main(String[] args) throws Exception {
        System.out.println(twrVoid());
        System.out.println(twrPop());
        System.out.println(twrPopNamed());
        System.out.println(twrVoidNamed());
    }
}
