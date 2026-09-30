public class M2 {
    private static int cleanupCount;
    public static void escaping() {
        cleanupCount = 0;
        try {
            throw new IllegalStateException("state");
        } catch (Throwable th) {
            cleanupCount++;
            throw th;
        }
    }
    public static int count() { return cleanupCount; }
    public static void main(String[] args) {
        try {
            escaping();
            System.out.println("missing throw");
        } catch (IllegalStateException error) {
            System.out.println(error.getMessage() + ":" + count());
        }
    }
}
