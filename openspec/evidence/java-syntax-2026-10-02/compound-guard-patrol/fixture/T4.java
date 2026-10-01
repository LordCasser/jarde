public class T4 implements AutoCloseable {
    static StringBuilder log = new StringBuilder();
    String name;
    T4(String n) { name = n; }
    @Override public void close() { log.append('[').append(name).append(']'); }
    public static String nested() throws Exception {
        try (T4 a = new T4("a")) {
            try (T4 b = new T4("b")) {
                log.append("body");
            } finally { log.append("mid"); }
        }
        return log.toString();
    }
    public static Object pick(Object o) {
        if (o instanceof String) { return ((String) o).length(); }
        if (o instanceof Integer) { return ((Integer) o) + 1; }
        return null;
    }
    public static int sync(int n) {
        synchronized (T4.class) {
            int s = 0;
            synchronized (log) {
                for (int i = 0; i < n; i++) { s += i; }
            }
            return s;
        }
    }
    public static void main(String[] x) throws Exception {
        System.out.println(nested());
        System.out.println(pick("hey") + ":" + pick(41) + ":" + pick(4.0));
        System.out.println(sync(5));
    }
}
