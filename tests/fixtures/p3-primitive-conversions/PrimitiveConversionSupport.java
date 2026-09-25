public class PrimitiveConversionSupport {
    public static int take(byte x) { return 100 + x; }
    public static int take(short x) { return 200 + x; }
    public static int take(char x) { return 300 + x; }
    public static int take(int x) { return 400 + x; }
    public static double take(float x) { return 600.0d + x; }
    public static double take(double x) { return 700.0d + x; }
}
