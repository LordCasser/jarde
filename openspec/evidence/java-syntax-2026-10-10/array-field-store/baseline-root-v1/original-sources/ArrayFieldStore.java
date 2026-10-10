public final class ArrayFieldStore {
    private byte[] values;
    private static int trace;

    public ArrayFieldStore() {
    }

    public void storeLiteral() {
        this.values = new byte[] { 10, 20, 30 };
    }

    public static void replace(ArrayFieldStore receiver, int failAt) {
        receiver.values = new byte[] {
            element(1, (byte) 11, failAt),
            element(2, (byte) 22, failAt),
            element(3, (byte) 33, failAt)
        };
    }

    private static byte element(int ordinal, byte value, int failAt) {
        trace = trace * 10 + ordinal;
        if (ordinal == failAt) {
            throw new IllegalStateException("element-" + ordinal);
        }
        return value;
    }

    public byte[] readValues() {
        return values;
    }

    public static void resetTrace() {
        trace = 0;
    }

    public static int readTrace() {
        return trace;
    }
}
