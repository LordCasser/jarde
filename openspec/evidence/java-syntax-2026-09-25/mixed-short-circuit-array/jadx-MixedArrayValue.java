package defpackage;

/* JADX INFO: loaded from: MixedArrayValue.class */
public final class MixedArrayValue {
    static boolean[] values = new boolean[1];
    static boolean bValue;
    static boolean cValue;
    static int arrayCalls;
    static int indexCalls;
    static int bCalls;
    static int cCalls;

    static boolean[] array(boolean z) {
        arrayCalls++;
        if (z) {
            return null;
        }
        return values;
    }

    static int index(int i) {
        indexCalls++;
        return i;
    }

    static boolean b() {
        bCalls++;
        return bValue;
    }

    static boolean c() {
        cCalls++;
        return cValue;
    }

    static void one(boolean z, boolean z2, int i) {
        array(z2)[index(i)] = (z && b()) || c();
    }
}
