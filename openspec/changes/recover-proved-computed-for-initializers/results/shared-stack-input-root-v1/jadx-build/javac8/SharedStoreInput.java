package defpackage;

/* JADX INFO: loaded from: jadx-input.jar:SharedStoreInput.class */
final class SharedStoreInput {
    static int savedField;
    static int limitCalls;
    static int limitValue;
    static boolean throwFromLimit;

    SharedStoreInput() {
    }

    static int limit() {
        limitCalls++;
        if (throwFromLimit) {
            throw new IllegalStateException("limit");
        }
        return limitValue;
    }

    static int find(int[] iArr, int i, int i2) {
        int iLimit = limit() - 1;
        savedField = iLimit;
        for (int i3 = iLimit; i3 >= i2; i3--) {
            if (iArr[i3] == i) {
                return i3;
            }
        }
        return -1;
    }
}
