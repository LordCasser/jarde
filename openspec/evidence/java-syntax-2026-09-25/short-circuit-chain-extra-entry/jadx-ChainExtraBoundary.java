package defpackage;

/* JADX INFO: loaded from: ChainExtraBoundary.class */
public final class ChainExtraBoundary {
    static boolean result;
    static boolean rhsValue;
    static int calls;

    static boolean rhs() {
        calls++;
        return rhsValue;
    }

    /* JADX WARN: Code duplicated, block: B:13:0x0019  */
    static void assign(boolean z, boolean z2, boolean z3, boolean z4) {
        boolean z5;
        if (!z ? !z3 : !z2) {
            z5 = true;
        } else if (z4 || rhs()) {
            z5 = true;
        } else {
            z5 = false;
        }
        result = z5;
    }
}
