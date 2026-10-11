package defpackage;

/* JADX INFO: loaded from: input.jar:ComputedForControls.class */
final class ComputedForControls {
    static int end;
    static int trace;
    static int mode;
    static final RuntimeException FAILURE = new IllegalStateException("chosen");

    ComputedForControls() {
    }

    static int f() {
        trace = (trace * 10) + 1;
        if (mode == 1) {
            throw FAILURE;
        }
        return end;
    }

    static void g() {
        trace = (trace * 10) + 2;
        end = 2;
        if (mode == 2) {
            throw FAILURE;
        }
    }

    static int find(int[] iArr, int i, int i2) {
        try {
            g();
            for (int iF = f() - 1; iF >= i2; iF--) {
                if (iArr[iF] == i) {
                    return iF;
                }
            }
            return -1;
        } catch (RuntimeException e) {
            return e == FAILURE ? -2 : -3;
        }
    }

    static int shared(int i) {
        int iF = f();
        int i2 = iF - 1;
        g();
        while (i2 >= i) {
            i2--;
        }
        return (iF * 100) + i2;
    }

    static int observedAfter(int i) {
        int iF = f() - 1;
        while (iF >= i) {
            iF--;
        }
        return iF;
    }

    static int externalWrite(int i) {
        for (int iF = f() - 1; iF >= i; iF--) {
        }
        return i;
    }

    static int scoped(int i, boolean z) {
        if (!z) {
            return -7;
        }
        int iF = f() - 1;
        while (iF >= i) {
            iF--;
        }
        return iF;
    }
}
