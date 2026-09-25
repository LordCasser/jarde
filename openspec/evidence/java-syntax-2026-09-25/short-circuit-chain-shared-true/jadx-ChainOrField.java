package defpackage;

/* JADX INFO: loaded from: ChainOrField.class */
public final class ChainOrField {
    static boolean result;
    static boolean rhsValue;
    static int calls;

    static boolean rhs() {
        calls++;
        return rhsValue;
    }

    static void assign(boolean z, boolean z2) {
        result = z || z2 || rhs();
    }
}
