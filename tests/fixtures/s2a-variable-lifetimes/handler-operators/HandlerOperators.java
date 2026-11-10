public final class HandlerOperators {
    public static int intOperators(int left, int right, boolean fail) {
        int shift = right & 3;
        int saved = (left & right) + (left | right) + (left ^ right)
                + (left << shift) + (left >> shift) + (left >>> shift);
        try {
            if (fail) throw new IllegalArgumentException();
            return -saved;
        } catch (IllegalArgumentException caught) {
            return -saved + right;
        }
    }

    public static long longOperators(long left, long right, boolean fail) {
        int shift = (int) (right & 7L);
        long saved = (left & right) + (left | right) + (left ^ right)
                + (left << shift) + (left >> shift) + (left >>> shift);
        try {
            if (fail) throw new IllegalArgumentException();
            return -saved;
        } catch (IllegalArgumentException caught) {
            return -saved + right;
        }
    }

    public static int converted(long value, boolean fail) {
        int saved = (int) value;
        try {
            if (fail) throw new IllegalArgumentException();
            saved += 1;
            return saved;
        } catch (IllegalArgumentException caught) {
            saved -= 1;
            return saved;
        }
    }

    public static int quotientBeforeTry(boolean fail, int numerator, int denominator) {
        int saved = numerator / denominator;
        try {
            if (fail) throw new IllegalArgumentException();
            return saved + 1;
        } catch (IllegalArgumentException caught) {
            return saved - 1;
        }
    }
}
