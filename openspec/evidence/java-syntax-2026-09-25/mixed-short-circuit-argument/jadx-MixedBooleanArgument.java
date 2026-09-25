package defpackage;

/* JADX INFO: loaded from: MixedBooleanArgument.class */
public final class MixedBooleanArgument {
    static boolean bValue;
    static boolean cValue;
    static boolean result;
    static int bCalls;
    static int cCalls;
    static int sinkCalls;

    static boolean b() {
        bCalls++;
        return bValue;
    }

    static boolean c() {
        cCalls++;
        return cValue;
    }

    static void sink(boolean z) {
        sinkCalls++;
        result = z;
    }

    public static void call(boolean z) {
        sink((z && b()) || c());
    }
}
