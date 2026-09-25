public class ConversionIntegers {
    public static long i2l(int x) { return (long) x; }
    public static float i2f(int x) { return (float) x; }
    public static double i2d(int x) { return (double) x; }
    public static byte i2b(int x) { return (byte) x; }
    public static char i2c(int x) { return (char) x; }
    public static short i2s(int x) { return (short) x; }
    public static int byteOverload(int x) { return ConversionSupport.take((byte) x); }
    public static int shortOverload(int x) { return ConversionSupport.take((short) x); }
    public static int charOverload(int x) { return ConversionSupport.take((char) x); }
}
