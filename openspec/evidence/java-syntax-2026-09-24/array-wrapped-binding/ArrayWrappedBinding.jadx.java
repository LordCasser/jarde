package defpackage;

/* JADX INFO: loaded from: ArrayWrappedBinding.class */
final class ArrayWrappedBinding {
    static int calls;
    static int[] watched;
    static RuntimeException failure;
    static int last;

    ArrayWrappedBinding() {
    }

    static int tick() {
        calls++;
        if (failure != null) {
            throw failure;
        }
        return calls;
    }

    static int mutateAndTick() {
        calls++;
        watched[0] = 90 + calls;
        if (failure != null) {
            throw failure;
        }
        return calls;
    }

    static int plusArrayFirst(int[] iArr) {
        int iTick = 0;
        for (int i : iArr) {
            iTick += i + tick();
        }
        return iTick;
    }

    static int plusTickFirst(int[] iArr) {
        int iMutateAndTick = 0;
        watched = iArr;
        for (int i : iArr) {
            iMutateAndTick += mutateAndTick() + i;
        }
        return iMutateAndTick;
    }

    static int callTickThenArray(int[] iArr) {
        int i = 0;
        for (int i2 : iArr) {
            consume(tick(), i2);
            i += last;
        }
        return i;
    }

    static int callMutateThenArray(int[] iArr) {
        int i = 0;
        watched = iArr;
        for (int i2 : iArr) {
            consume(mutateAndTick(), i2);
            i += last;
        }
        return i;
    }

    static int readAfterEffect(int[] iArr) {
        int i = 0;
        watched = iArr;
        for (int i2 : iArr) {
            mutateAndTick();
            i += i2;
        }
        return i;
    }

    static int twoReadsWithMutation(int[] iArr) {
        int i = 0;
        watched = iArr;
        int length = iArr.length;
        for (int i2 = 0; i2 < length; i2++) {
            int i3 = iArr[i2];
            mutateAndTick();
            i += i3 + iArr[i2];
        }
        return i;
    }

    static int wrappedOnlyRead(int[] iArr) {
        int iTick = 0;
        for (int i : iArr) {
            iTick += i + tick();
        }
        return iTick;
    }

    static void consume(int i, int i2) {
        last = (i * 1000) + i2;
    }
}
