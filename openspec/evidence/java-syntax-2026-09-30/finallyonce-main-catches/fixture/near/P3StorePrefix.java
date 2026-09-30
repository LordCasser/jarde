public class P3StorePrefix {
    public static java.io.ByteArrayOutputStream open() { return new java.io.ByteArrayOutputStream(); }
    public static void t() { throw new IllegalStateException("state"); }
    public static void main(String[] args) {
        java.io.ByteArrayOutputStream r = open();
        try {
            t();
            r.write(2);
        } catch (IllegalStateException error) {
            System.out.println(r.size());
        }
    }
}
