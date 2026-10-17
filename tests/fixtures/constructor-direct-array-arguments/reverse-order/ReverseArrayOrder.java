public final class ReverseArrayOrder {
    public static int trace;

    public final byte[] left;
    public final byte[] right;

    public ReverseArrayOrder(byte[] left, byte[] right) {
        this.left = left;
        this.right = right;
    }

    public static int mark(int digit) {
        trace = trace * 10 + digit;
        return digit;
    }

    public static ReverseArrayOrder make() {
        return new ReverseArrayOrder(new byte[mark(1)], new byte[mark(2)]);
    }
}
