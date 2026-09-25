package defpackage;

/* JADX INFO: loaded from: FinallyCompletion.class */
public class FinallyCompletion {
    public static int trace;
    public static final RuntimeException TRY_FAILURE = new IllegalArgumentException("try");
    public static final RuntimeException FINALLY_FAILURE = new IllegalStateException("finally");

    private static int mark(int i) {
        trace = (trace * 10) + i;
        return i;
    }

    public static int normalReturn() {
        try {
            int i = 2;
            return mark(1);
        } finally {
            mark(2);
        }
    }

    public static int finallyReturns() {
        try {
            mark(3);
            return mark(4);
        } catch (Throwable th) {
            return mark(4);
        }
    }

    public static int finallyThrows() {
        try {
            mark(5);
            mark(6);
            throw FINALLY_FAILURE;
        } catch (Throwable th) {
            mark(6);
            throw FINALLY_FAILURE;
        }
    }

    public static int tryThrowsFinallyRuns() {
        try {
            int i = 7;
            throw TRY_FAILURE;
        } finally {
            mark(8);
        }
    }
}
