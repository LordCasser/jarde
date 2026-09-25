package defpackage;

import java.util.function.Supplier;

/* JADX INFO: loaded from: ObjectArrayForeach.class */
final class ObjectArrayForeach {
    ObjectArrayForeach() {
    }

    static int sumHash(Object[] objArr) {
        int iHashCode = 0;
        for (Object obj : objArr) {
            iHashCode += obj.hashCode();
        }
        return iHashCode;
    }

    static int sumHashFrom(Supplier<Object[]> supplier) {
        int iHashCode = 0;
        for (Object obj : supplier.get()) {
            iHashCode += obj.hashCode();
        }
        return iHashCode;
    }
}
