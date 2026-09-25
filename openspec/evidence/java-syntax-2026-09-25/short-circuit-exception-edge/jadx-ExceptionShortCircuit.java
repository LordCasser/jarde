package defpackage;

/* JADX INFO: loaded from: ExceptionShortCircuit.class */
public final class ExceptionShortCircuit {
    static boolean result;
    static int calls;

    static boolean mayThrow() {
        calls++;
        throw new IllegalStateException("rhs");
    }

    /* JADX WARN: Code duplicated, block: B:7:0x000e  */
    static boolean assign(boolean z) {
        boolean z2;
        if (z) {
            try {
                if (mayThrow()) {
                    z2 = true;
                } else {
                    z2 = false;
                }
            } catch (RuntimeException e) {
                return result;
            }
        } else {
            z2 = false;
        }
        result = z2;
        return result;
    }
}
