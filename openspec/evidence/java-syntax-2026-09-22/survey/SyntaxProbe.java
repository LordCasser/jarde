public class SyntaxProbe {
    public SyntaxProbe() {
        super();
    }

    public static int unaryInt(int value) {
        return -(-value);
    }

    public static long unaryLong(long value) {
        return -(-value) * (100L / -value);
    }

    public static float unaryFloat(float value) {
        return -(-value) * (100.0f / -value);
    }

    public static double unaryDouble(double value) {
        return -(-value) * (100.0d / -value);
    }

    public static float negativeZeroFloat() {
        return -0.0f;
    }

    public static double negativeZeroDouble() {
        return -0.0d;
    }

    public static String checkAndCast(Object value) {
        if (value instanceof String) {
            return (String) value;
        }
        return null;
    }

    public static boolean compareLong(long left, long right) {
        return left < right || left == right;
    }

    public static boolean compareFloating(double left, double right) {
        return left <= right || left != right;
    }

    private static int conditionValue(int[] values, int index) {
        return values[index];
    }

    public static int loopConditionCall(int[] values) {
        int index = 0;
        while (conditionValue(values, index) < 3) {
            index++;
        }
        return index;
    }

    public static int bitMix(int left, int right) {
        return (left & right) | ((left ^ right) << 1) | (left >>> 2);
    }

    public static int[] arrayInitializer() {
        return new int[] {1, 2, 3, -4};
    }

    public static void throwParameter(String message) {
        throw new IllegalArgumentException(message);
    }
}
