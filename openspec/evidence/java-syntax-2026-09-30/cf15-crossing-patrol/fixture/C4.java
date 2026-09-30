public class C4 implements AutoCloseable {
    @Override public void close() { }
    public static String twrNamed() throws Exception {
        try (C4 r = new C4()) {
            r.toString();
        } catch (IllegalStateException e) {
            return "caught";
        }
        return "done";
    }
    public static String constructNamed() {
        StringBuilder b = new StringBuilder();
        try {
            b.append('a');
        } catch (IllegalStateException e) {
            return "caught";
        }
        return b.toString();
    }
    public static void main(String[] args) throws Exception {
        System.out.println(twrNamed());
        System.out.println(constructNamed());
    }
}
