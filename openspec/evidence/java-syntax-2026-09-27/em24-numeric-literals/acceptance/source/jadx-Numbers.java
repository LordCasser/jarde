package em24;

/* JADX INFO: loaded from: input.jar:em24/Numbers.class */
public final class Numbers {
    private Numbers() {
    }

    public static byte[] bytes() {
        return new byte[]{0, -1, -10, -1, -128, 127};
    }

    public static short[] shorts() {
        return new short[]{0, -1, -10, -1, Short.MIN_VALUE, Short.MAX_VALUE};
    }

    public static int[] ints() {
        return new int[]{0, -1, -10, -1, Integer.MIN_VALUE, Integer.MAX_VALUE};
    }

    public static long[] longs() {
        return new long[]{0, -1, -10, -1, Long.MIN_VALUE, Long.MAX_VALUE};
    }

    public static float[] floats() {
        return new float[]{0.55f, -0.0f, Float.NaN, Float.NEGATIVE_INFINITY, Float.POSITIVE_INFINITY, Float.MIN_VALUE, Float.MIN_NORMAL, Float.MAX_VALUE};
    }

    public static double[] doubles() {
        return new double[]{0.55d, -0.0d, Double.NaN, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY, Double.MIN_VALUE, Double.MIN_NORMAL, Double.MAX_VALUE};
    }
}
