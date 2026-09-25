public final class GuardReturnEffects {
    private static final Object LOCK = new Object();
    static int calls;

    int n;

    static RuntimeException afterRead(GuardReturnEffects target, boolean fail) {
        calls++;
        target.n += 10;
        if (fail) {
            return new IllegalStateException("after-read");
        }
        return null;
    }

    int effect(boolean fail) {
        synchronized (this) {
            int old = this.n;
            RuntimeException failure = afterRead(this, fail);
            if (failure != null) {
                throw failure;
            }
            return old;
        }
    }

    int nested(boolean fail) {
        synchronized (this) {
            synchronized (LOCK) {
                int old = this.n;
                RuntimeException failure = afterRead(this, fail);
                if (failure != null) {
                    throw failure;
                }
                return old;
            }
        }
    }
}
