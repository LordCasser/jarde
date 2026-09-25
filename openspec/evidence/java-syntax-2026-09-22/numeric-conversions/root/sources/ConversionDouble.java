public class ConversionDouble {
    public static int d2i(double x) { return (int) x; }
    public static long d2l(double x) { return (long) x; }
    public static float d2f(double x) { return (float) x; }
    public static int doubleToIntOverload(double x) { return ConversionSupport.take((int) x); }
}
