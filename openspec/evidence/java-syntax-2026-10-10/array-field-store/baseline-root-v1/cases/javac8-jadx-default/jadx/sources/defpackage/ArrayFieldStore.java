package defpackage;

/* JADX INFO: loaded from: ArrayFieldStore.class */
public final class ArrayFieldStore {
    private byte[] values;
    private static int trace;

    public void storeLiteral() {
        this.values = new byte[]{10, 20, 30};
    }

    public static void replace(ArrayFieldStore arrayFieldStore, int i) {
        arrayFieldStore.values = new byte[]{element(1, (byte) 11, i), element(2, (byte) 22, i), element(3, (byte) 33, i)};
    }

    private static byte element(int i, byte b, int i2) {
        trace = (trace * 10) + i;
        if (i == i2) {
            throw new IllegalStateException("element-" + i);
        }
        return b;
    }

    public byte[] readValues() {
        return this.values;
    }

    public static void resetTrace() {
        trace = 0;
    }

    public static int readTrace() {
        return trace;
    }
}
