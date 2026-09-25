public final class ThrowGuardProbe {
    public static void withFinally(RuntimeException problem) {
        try {
            throw problem;
        } finally {
            ThrowEffects.mark();
        }
    }

    public static void synchronizedBody(Object lock, RuntimeException problem) {
        synchronized (lock) {
            throw problem;
        }
    }
}
