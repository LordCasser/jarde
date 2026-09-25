package defpackage;

/* JADX INFO: loaded from: FinallyNormal.class */
public class FinallyNormal {
    public static int trace;
    public static boolean failAtOne;
    public static final RuntimeException FAILURE = new IllegalArgumentException("mark-1");

    private static int mark(int i) {
        trace = (trace * 10) + i;
        if (i == 1 && failAtOne) {
            throw FAILURE;
        }
        return i;
    }

    public static int run() {
        try {
            int i = 2;
            return mark(1);
        } finally {
            mark(2);
        }
    }
}
