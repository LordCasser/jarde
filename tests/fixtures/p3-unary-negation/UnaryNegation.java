public final class UnaryNegation {
    static int calls;

    public static int negInt(int value) {
        return -value;
    }

    public static long negLong(long value) {
        return -value;
    }

    public static float negFloat(float value) {
        return -value;
    }

    public static double negDouble(double value) {
        return -value;
    }

    public static int negByte(byte value) {
        return -value;
    }

    public static int negChar(char value) {
        return -value;
    }

    public static int negShort(short value) {
        return -value;
    }

    public static int nested(int value) {
        return -(-value);
    }

    public static int negSum(int left, int right) {
        return -(left + right);
    }

    public static int multiplyRight(int left, int right) {
        return left * -right;
    }

    public static int divideRight(int left, int right) {
        return left / -right;
    }

    public static int local(int value) {
        int result = -value;
        return result;
    }

    public static int consume(int value) {
        calls++;
        return value;
    }

    public static int callArgument(int value) {
        return consume(-value);
    }

    public static int throwing(int value) {
        return fail(-value);
    }

    public static int negatedCall(int value) {
        return -consume(value);
    }

    public static int negatedFail(int value) {
        return -fail(value);
    }

    private static int fail(int value) {
        throw new IllegalStateException(Integer.toString(value));
    }

    public static int unsupportedOperand() {
        return -(byte) consume(7);
    }
}
