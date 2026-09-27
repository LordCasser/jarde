package em24;

public final class Numbers {
    private Numbers() {}

    public static byte[] bytes() {
        return new byte[] {0, -1, -0xA, (byte) 0xff, Byte.MIN_VALUE, Byte.MAX_VALUE};
    }

    public static short[] shorts() {
        return new short[] {0, -1, -0xA, (short) 0xffff, Short.MIN_VALUE, Short.MAX_VALUE};
    }

    public static int[] ints() {
        return new int[] {0, -1, -0xA, 0xffff_ffff, Integer.MIN_VALUE, Integer.MAX_VALUE};
    }

    public static long[] longs() {
        return new long[] {0, -1, -0xA, 0xffff_ffff_ffff_ffffL, Long.MIN_VALUE, Long.MAX_VALUE};
    }

    public static float[] floats() {
        return new float[] {0.55f, -0.0f, Float.NaN, Float.NEGATIVE_INFINITY,
                Float.POSITIVE_INFINITY, Float.MIN_VALUE, Float.MIN_NORMAL, Float.MAX_VALUE};
    }

    public static double[] doubles() {
        return new double[] {0.55d, -0.0d, Double.NaN, Double.NEGATIVE_INFINITY,
                Double.POSITIVE_INFINITY, Double.MIN_VALUE, Double.MIN_NORMAL, Double.MAX_VALUE};
    }
}
