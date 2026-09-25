public class ConversionRunner {
    private static String floatBits(float value) {
        return Integer.toHexString(Float.floatToRawIntBits(value));
    }

    private static String doubleBits(double value) {
        return Long.toHexString(Double.doubleToRawLongBits(value));
    }

    public static void main(String[] args) {
        int[] ints = { Integer.MIN_VALUE, -65537, -1, 0, 1, 65535, Integer.MAX_VALUE };
        long[] longs = { Long.MIN_VALUE, -4294967297L, -4294967296L, -1L, 0L, 1L, 4294967296L, 4294967297L, Long.MAX_VALUE };
        float[] floats = { Float.NEGATIVE_INFINITY, -Float.MAX_VALUE, -0.0f, 0.0f, Float.MIN_VALUE, 1.5f, Float.MAX_VALUE, Float.NaN, Float.POSITIVE_INFINITY };
        double[] doubles = { Double.NEGATIVE_INFINITY, -Double.MAX_VALUE, -0.0d, 0.0d, Double.MIN_VALUE, 1.5d, Double.MAX_VALUE, Double.NaN, Double.POSITIVE_INFINITY };

        for (int i = 0; i < ints.length; i++) {
            int x = ints[i];
            System.out.println("I:" + i + ":i2l=" + ConversionIntegers.i2l(x));
            System.out.println("I:" + i + ":i2f=" + floatBits(ConversionIntegers.i2f(x)));
            System.out.println("I:" + i + ":i2d=" + doubleBits(ConversionIntegers.i2d(x)));
            System.out.println("I:" + i + ":i2b=" + ConversionIntegers.i2b(x));
            System.out.println("I:" + i + ":i2c=" + (int) ConversionIntegers.i2c(x));
            System.out.println("I:" + i + ":i2s=" + ConversionIntegers.i2s(x));
            System.out.println("I:" + i + ":byteOverload=" + ConversionIntegers.byteOverload(x));
            System.out.println("I:" + i + ":shortOverload=" + ConversionIntegers.shortOverload(x));
            System.out.println("I:" + i + ":charOverload=" + ConversionIntegers.charOverload(x));
        }

        for (int i = 0; i < longs.length; i++) {
            long x = longs[i];
            System.out.println("L:" + i + ":l2i=" + ConversionLongFloat.l2i(x));
            System.out.println("L:" + i + ":l2f=" + floatBits(ConversionLongFloat.l2f(x)));
            System.out.println("L:" + i + ":l2d=" + doubleBits(ConversionLongFloat.l2d(x)));
            System.out.println("L:" + i + ":longToIntOverload=" + ConversionLongFloat.longToIntOverload(x));
            System.out.println("L:" + i + ":longToFloatOverload=" + doubleBits(ConversionLongFloat.longToFloatOverload(x)));
        }

        for (int i = 0; i < floats.length; i++) {
            float x = floats[i];
            System.out.println("F:" + i + ":f2i=" + ConversionLongFloat.f2i(x));
            System.out.println("F:" + i + ":f2l=" + ConversionLongFloat.f2l(x));
            System.out.println("F:" + i + ":f2d=" + doubleBits(ConversionLongFloat.f2d(x)));
            System.out.println("F:" + i + ":floatToDoubleOverload=" + doubleBits(ConversionLongFloat.floatToDoubleOverload(x)));
        }

        for (int i = 0; i < doubles.length; i++) {
            double x = doubles[i];
            System.out.println("D:" + i + ":d2i=" + ConversionDouble.d2i(x));
            System.out.println("D:" + i + ":d2l=" + ConversionDouble.d2l(x));
            System.out.println("D:" + i + ":d2f=" + floatBits(ConversionDouble.d2f(x)));
            System.out.println("D:" + i + ":doubleToIntOverload=" + ConversionDouble.doubleToIntOverload(x));
        }
    }
}
