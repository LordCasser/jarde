public class P1 implements AutoCloseable {
    static StringBuilder log = new StringBuilder();
    @Override public void close() { log.append("[c]"); }
    public static String soloFinally() throws Exception {
        try (P1 r = new P1()) {
            log.append("b");
        } finally {
            log.append("f");
        }
        return log.toString();
    }
    public static String twrCatch() throws Exception {
        try (P1 r = new P1()) {
            log.append("B");
            if (log.length() > 100) throw new IllegalStateException("x");
        } catch (IllegalStateException e) {
            log.append("E");
        }
        return log.toString();
    }
    public static void main(String[] a) throws Exception {
        System.out.println(soloFinally());
        System.out.println(twrCatch());
    }
}
