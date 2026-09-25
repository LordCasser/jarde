public final class ThrowStack {
    public static int calls;

    public static Object mark() {
        calls++;
        return null;
    }

    public static void probe(RuntimeException problem) {
        mark();
        throw problem;
    }
}
