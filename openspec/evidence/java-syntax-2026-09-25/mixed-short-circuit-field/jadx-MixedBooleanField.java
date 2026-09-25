package defpackage;

/* JADX INFO: loaded from: MixedBooleanField.class */
public final class MixedBooleanField {
    static boolean result;
    static boolean bValue;
    static boolean cValue;
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

    static void andOr(boolean z) {
        result = (z && b()) || c();
    }

    static void orAnd(boolean z) {
        result = (z || b()) && c();
    }
}
