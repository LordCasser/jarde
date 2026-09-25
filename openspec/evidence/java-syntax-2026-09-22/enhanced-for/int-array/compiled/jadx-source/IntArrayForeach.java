package defpackage;

import java.util.function.Supplier;

/* JADX INFO: loaded from: IntArrayForeach.class */
final class IntArrayForeach {
    IntArrayForeach() {
    }

    static int sum(int[] iArr) {
        int i = 0;
        for (int i2 : iArr) {
            i += i2;
        }
        return i;
    }

    static int sumFrom(Supplier<int[]> supplier) {
        int i = 0;
        for (int i2 : supplier.get()) {
            i += i2;
        }
        return i;
    }
}
