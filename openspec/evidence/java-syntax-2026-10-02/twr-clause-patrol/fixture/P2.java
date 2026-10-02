public class P2 implements AutoCloseable {
    static StringBuilder log = new StringBuilder();
    static boolean mode;
    @Override public void close() { log.append("[c]"); }
    public static String branchNoCatch() throws Exception {
        try (P2 r = new P2()) {
            if (mode) throw new IllegalStateException("x");
            log.append("B");
        }
        return log.toString();
    }
    public static String plainCatch() throws Exception {
        try (P2 r = new P2()) {
            log.append("b");
        } catch (IllegalStateException e) {
            log.append("E");
        }
        return log.toString();
    }
    public static void main(String[] a) throws Exception {
        System.out.println(branchNoCatch()); System.out.println(plainCatch());
    }
}
