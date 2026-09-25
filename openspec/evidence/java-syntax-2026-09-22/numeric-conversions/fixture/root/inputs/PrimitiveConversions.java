public class PrimitiveConversions {
    public static long i2l(int x) { return (long) x; }
    public static float i2f(int x) { return (float) x; }
    public static double i2d(int x) { return (double) x; }
    public static byte i2b(int x) { return (byte) x; }
    public static char i2c(int x) { return (char) x; }
    public static short i2s(int x) { return (short) x; }

    public static int l2i(long x) { return (int) x; }
    public static float l2f(long x) { return (float) x; }
    public static double l2d(long x) { return (double) x; }

    public static int f2i(float x) { return (int) x; }
    public static long f2l(float x) { return (long) x; }
    public static double f2d(float x) { return (double) x; }

    public static int d2i(double x) { return (int) x; }
    public static long d2l(double x) { return (long) x; }
    public static float d2f(double x) { return (float) x; }

    public static int byteOverload(int x) { return PrimitiveConversionSupport.take((byte) x); }
    public static int shortOverload(int x) { return PrimitiveConversionSupport.take((short) x); }
    public static int charOverload(int x) { return PrimitiveConversionSupport.take((char) x); }
    public static int longIntOverload(long x) { return PrimitiveConversionSupport.take((int) x); }
    public static double longFloatOverload(long x) { return PrimitiveConversionSupport.take((float) x); }
    public static double floatDoubleOverload(float x) { return PrimitiveConversionSupport.take((double) x); }
    public static int doubleIntOverload(double x) { return PrimitiveConversionSupport.take((int) x); }

    public static long intFloatRound(int x) { return (long) (float) x; }
    public static long longFloatRound(long x) { return (long) (float) x; }
    public static long longDoubleRound(long x) { return (long) (double) x; }
    public static double doubleFloatRound(double x) { return (double) (float) x; }
    public static byte floatByte(float x) { return (byte) x; }
    public static char doubleChar(double x) { return (char) x; }
    public static int byteChar(int x) { return (char) (byte) x; }

    public static long ordered(int x, boolean left, boolean right) {
        return (long) PrimitiveConversionEffects.left(x, left)
                + (long) PrimitiveConversionEffects.right(x, right);
    }
}
