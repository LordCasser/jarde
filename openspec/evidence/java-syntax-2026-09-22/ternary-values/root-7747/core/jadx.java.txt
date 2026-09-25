package defpackage;

/* JADX INFO: loaded from: TernaryCore.class */
public class TernaryCore {
    private static int trace;

    public static void reset() {
        trace = 0;
    }

    public static int trace() {
        return trace;
    }

    private static int a() {
        trace = (trace * 10) + 1;
        return 7;
    }

    private static int b() {
        trace = (trace * 10) + 2;
        return 11;
    }

    public static int returned(boolean z) {
        return z ? a() : b();
    }
}
