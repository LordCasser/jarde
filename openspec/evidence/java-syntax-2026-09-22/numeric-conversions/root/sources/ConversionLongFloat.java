public class ConversionLongFloat {
    public static int l2i(long x) { return (int) x; }
    public static float l2f(long x) { return (float) x; }
    public static double l2d(long x) { return (double) x; }
    public static int f2i(float x) { return (int) x; }
    public static long f2l(float x) { return (long) x; }
    public static double f2d(float x) { return (double) x; }
    public static int longToIntOverload(long x) { return ConversionSupport.take((int) x); }
    public static double longToFloatOverload(long x) { return ConversionSupport.take((float) x); }
    public static double floatToDoubleOverload(float x) { return ConversionSupport.take((double) x); }
}
