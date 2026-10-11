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
            int iF = f() - 1;
            g();
            for (int i3 = iF; i3 >= i2; i3--) {
                if (iArr[i3] == i) {
                    return i3;
                }
            }
            return -1;
        } catch (RuntimeException e) {
            return e == FAILURE ? -2 : -3;
        }
    }

    static int shared(int i) {
        int iF = f();
        g();
        int i2 = iF - 1;
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
