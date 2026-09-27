public class IdentityOnce {
    public static final IllegalStateException ORIGINAL = new IllegalStateException("same");
    private static int count;

    public static void escaping() {
        count = 0;
        try {
            throw ORIGINAL;
        } finally {
            count++;
        }
    }

    public static int count() { return count; }
}
