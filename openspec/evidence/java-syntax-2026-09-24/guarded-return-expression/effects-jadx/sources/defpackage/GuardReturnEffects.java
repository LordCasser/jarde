package defpackage;

/* JADX INFO: loaded from: GuardReturnEffects.class */
public final class GuardReturnEffects {
    private static final Object LOCK = new Object();
    static int calls;
    int n;

    static RuntimeException afterRead(GuardReturnEffects guardReturnEffects, boolean z) {
        calls++;
        guardReturnEffects.n += 10;
        if (z) {
            return new IllegalStateException("after-read");
        }
        return null;
    }

    int effect(boolean z) {
        int i;
        synchronized (this) {
            i = this.n;
            RuntimeException runtimeExceptionAfterRead = afterRead(this, z);
            if (runtimeExceptionAfterRead != null) {
                throw runtimeExceptionAfterRead;
            }
        }
        return i;
    }

    int nested(boolean z) {
        int i;
        synchronized (this) {
            synchronized (LOCK) {
                i = this.n;
                RuntimeException runtimeExceptionAfterRead = afterRead(this, z);
                if (runtimeExceptionAfterRead != null) {
                    throw runtimeExceptionAfterRead;
                }
            }
        }
        return i;
    }
}
