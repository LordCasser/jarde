package defpackage;

import java.io.IOException;

/* JADX INFO: loaded from: ThrowAudit.class */
public class ThrowAudit {
    public static void nullValue() {
        throw null;
    }

    public static void parameter(RuntimeException runtimeException) {
        throw runtimeException;
    }

    public static void allocation() {
        throw new IllegalStateException("fresh");
    }

    public static void call() {
        throw ThrowEffects.problem();
    }

    public static void cast(Object obj) {
        throw ((RuntimeException) obj);
    }

    public static void checked(IOException iOException) throws IOException {
        throw iOException;
    }

    public static RuntimeException caught(RuntimeException runtimeException) {
        try {
            throw runtimeException;
        } catch (RuntimeException e) {
            return e;
        }
    }

    public static void withFinally(RuntimeException runtimeException) {
        try {
            throw runtimeException;
        } catch (Throwable th) {
            ThrowEffects.mark();
            throw th;
        }
    }

    public static void synchronizedBody(Object obj, RuntimeException runtimeException) {
        synchronized (obj) {
            throw runtimeException;
        }
    }

    public static void conditional(boolean z, RuntimeException runtimeException, RuntimeException runtimeException2) {
        if (!z) {
            throw runtimeException2;
        }
        throw runtimeException;
    }
}
