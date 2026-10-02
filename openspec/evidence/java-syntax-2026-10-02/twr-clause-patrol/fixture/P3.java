public class P3 implements AutoCloseable {
    static StringBuilder log = new StringBuilder();
    static void touch(P3 r) { log.append("t"); }
    static void boom() { if (log.length() > 1000) throw new IllegalStateException("x"); }
    @Override public void close() { log.append("[c]"); }
    public static String voidBodyCatch() throws Exception {
        try (P3 r = new P3()) {
            touch(r);
        } catch (IllegalStateException e) {
            log.append("E");
        }
        return log.toString();
    }
    public static String voidBodyBranch() throws Exception {
        try (P3 r = new P3()) {
            touch(r);
            boom();
        }
        return log.toString();
    }
    public static String voidBodySoloFin() throws Exception {
        try (P3 r = new P3()) {
            touch(r);
        } finally {
            log.append("f");
        }
        return log.toString();
    }
    public static void main(String[] a) throws Exception {
        System.out.println(voidBodyCatch()); System.out.println(voidBodyBranch()); System.out.println(voidBodySoloFin());
    }
}
