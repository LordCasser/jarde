package defpackage;

/* JADX INFO: loaded from: input.jar:ComputedForDirectReturn.class */
final class ComputedForDirectReturn {
    static int end;
    static int calls;

    ComputedForDirectReturn() {
    }

    static int limit() {
        calls++;
        return end;
    }

    static int find(int[] iArr, int i, int i2) {
        for (int iLimit = limit() - 1; iLimit >= i2; iLimit--) {
            if (iArr[iLimit] == i) {
                return iLimit;
            }
        }
        return -1;
    }
}
