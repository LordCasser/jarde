public class PrimitiveConversionsRunner {
    private static String floatBits(float value) {
        return Integer.toHexString(Float.floatToRawIntBits(value));
    }

    private static String doubleBits(double value) {
        return Long.toHexString(Double.doubleToRawLongBits(value));
    }

    public static void main(String[] args) {
        int[] ints = { Integer.MIN_VALUE, -1, 0, 16777217, Integer.MAX_VALUE };
        for (int x : ints) {
            System.out.println("i:" + x + ":" + PrimitiveConversions.i2l(x) + ":" + floatBits(PrimitiveConversions.i2f(x)) + ":" + doubleBits(PrimitiveConversions.i2d(x)) + ":" + PrimitiveConversions.i2b(x) + ":" + (int) PrimitiveConversions.i2c(x) + ":" + PrimitiveConversions.i2s(x));
            System.out.println("io:" + x + ":" + PrimitiveConversions.byteOverload(x) + ":" + PrimitiveConversions.shortOverload(x) + ":" + PrimitiveConversions.charOverload(x) + ":" + PrimitiveConversions.intFloatRound(x));
        }

        long[] longs = { Long.MIN_VALUE, -9007199254740993L, 0L, 9007199254740993L, Long.MAX_VALUE };
        for (long x : longs) {
            System.out.println("l:" + x + ":" + PrimitiveConversions.l2i(x) + ":" + floatBits(PrimitiveConversions.l2f(x)) + ":" + doubleBits(PrimitiveConversions.l2d(x)) + ":" + PrimitiveConversions.longIntOverload(x) + ":" + doubleBits(PrimitiveConversions.longFloatOverload(x)));
            System.out.println("lr:" + x + ":" + PrimitiveConversions.longFloatRound(x) + ":" + PrimitiveConversions.longDoubleRound(x));
        }

        float[] floats = { Float.NEGATIVE_INFINITY, -0.0f, 1.5f, Float.MIN_VALUE, Float.NaN, Float.POSITIVE_INFINITY };
        for (float x : floats) {
            System.out.println("f:" + floatBits(x) + ":" + PrimitiveConversions.f2i(x) + ":" + PrimitiveConversions.f2l(x) + ":" + doubleBits(PrimitiveConversions.f2d(x)) + ":" + doubleBits(PrimitiveConversions.floatDoubleOverload(x)) + ":" + PrimitiveConversions.floatByte(x));
        }

        double[] doubles = { Double.NEGATIVE_INFINITY, -0.0d, 1.5d, Double.MIN_VALUE, Double.NaN, Double.POSITIVE_INFINITY };
        for (double x : doubles) {
            System.out.println("d:" + doubleBits(x) + ":" + PrimitiveConversions.d2i(x) + ":" + PrimitiveConversions.d2l(x) + ":" + floatBits(PrimitiveConversions.d2f(x)) + ":" + PrimitiveConversions.doubleIntOverload(x) + ":" + (int) PrimitiveConversions.doubleChar(x));
        }

        System.out.println("chain:" + doubleBits(PrimitiveConversions.doubleFloatRound(16777217.0d)) + ":" + PrimitiveConversions.byteChar(257));
        for (int x : new int[] { Integer.MIN_VALUE, Integer.MAX_VALUE }) {
            for (boolean left : new boolean[] { false, true }) {
                for (boolean right : new boolean[] { false, true }) {
                    PrimitiveConversionEffects.trace = 0;
                    try {
                        System.out.println("order:" + x + ":" + left + ":" + right + ":" + PrimitiveConversions.ordered(x, left, right) + ":" + PrimitiveConversionEffects.trace);
                    } catch (Throwable error) {
                        System.out.println("order:" + x + ":" + left + ":" + right + ":" + error.getClass().getName() + ":" + PrimitiveConversionEffects.trace);
                    }
                }
            }
        }
    }
}
