package defpackage;

/* JADX INFO: loaded from: MixedBooleanLocal.class */
public final class MixedBooleanLocal {
    static boolean bValue;
    static boolean cValue;
    static boolean result;
    static int bCalls;
    static int cCalls;

    static boolean b() {
        bCalls++;
        return bValue;
    }

    static boolean c() {
        cCalls++;
        return cValue;
    }

    static boolean one(boolean z) {
        boolean z2 = (z && b()) || c();
        result = z2;
        return z2;
    }
}
