package defpackage;

/* JADX INFO: loaded from: Wtyping.class */
public class Wtyping implements AutoCloseable {
    public static boolean boom = false;

    @Override // java.lang.AutoCloseable
    public void close() {
        if (boom) {
            throw new IllegalStateException("close");
        }
    }

    static void touch(Wtyping wtyping) {
        if (boom) {
            throw new IllegalStateException("touch");
        }
    }

    public static String stringLiteral() throws Exception {
        Wtyping wtyping = new Wtyping();
        try {
            touch(wtyping);
            wtyping.close();
            return "in";
        } catch (Throwable th) {
            try {
                wtyping.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static int intLiteral() throws Exception {
        Wtyping wtyping = new Wtyping();
        try {
            touch(wtyping);
            wtyping.close();
            return 42;
        } catch (Throwable th) {
            try {
                wtyping.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static StringBuilder constructorValue() throws Exception {
        Wtyping wtyping = new Wtyping();
        try {
            touch(wtyping);
            StringBuilder sb = new StringBuilder("built");
            wtyping.close();
            return sb;
        } catch (Throwable th) {
            try {
                wtyping.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static String callReturn() throws Exception {
        Wtyping wtyping = new Wtyping();
        try {
            touch(wtyping);
            String strValueOf = String.valueOf(7);
            wtyping.close();
            return strValueOf;
        } catch (Throwable th) {
            try {
                wtyping.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static String nullValue() throws Exception {
        Wtyping wtyping = new Wtyping();
        try {
            touch(wtyping);
            wtyping.close();
            return null;
        } catch (Throwable th) {
            try {
                wtyping.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }

    public static Class<?> classLiteral() throws Exception {
        Wtyping wtyping = new Wtyping();
        try {
            touch(wtyping);
            wtyping.close();
            return String.class;
        } catch (Throwable th) {
            try {
                wtyping.close();
            } catch (Throwable th2) {
                th.addSuppressed(th2);
            }
            throw th;
        }
    }
}
