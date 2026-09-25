public class ThrowAudit {
    public static void nullValue() { throw null; }
    public static void parameter(RuntimeException problem) { throw problem; }
    public static void allocation() { throw new IllegalStateException("fresh"); }
    public static void call() { throw ThrowEffects.problem(); }
    public static void cast(Object problem) { throw (RuntimeException) problem; }
    public static void checked(java.io.IOException problem) throws java.io.IOException { throw problem; }
    public static RuntimeException caught(RuntimeException problem) {
        try { throw problem; } catch (RuntimeException caught) { return caught; }
    }
    public static void withFinally(RuntimeException problem) {
        try { throw problem; } finally { ThrowEffects.mark(); }
    }
    public static void synchronizedBody(Object lock, RuntimeException problem) {
        synchronized (lock) { throw problem; }
    }
    public static void conditional(boolean first, RuntimeException a, RuntimeException b) {
        if (first) { throw a; }
        throw b;
    }
}
