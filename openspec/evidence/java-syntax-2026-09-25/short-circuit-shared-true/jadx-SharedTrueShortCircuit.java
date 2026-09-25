package defpackage;

/* JADX INFO: loaded from: in.jar:SharedTrueShortCircuit.class */
public final class SharedTrueShortCircuit {
    static boolean result;
    static int calls;

    static boolean rhs() {
        calls++;
        return true;
    }

    static void assign(boolean z) {
        result = z || rhs();
    }
}
