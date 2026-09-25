public class ShiftSlice {
    public static int intLeft(int value, int distance) { return value << distance; }
    public static int intRight(int value, int distance) { return value >> distance; }
    public static int intUnsigned(int value, int distance) { return value >>> distance; }
    public static long longLeft(long value, int distance) { return value << distance; }
    public static long longRight(long value, int distance) { return value >> distance; }
    public static long longUnsigned(long value, int distance) { return value >>> distance; }
    public static int shortLeft(short value, int distance) { return value << distance; }
    public static int charUnsigned(char value, int distance) { return value >>> distance; }
    public static int nested(int value, int first, int second) { return value << first >>> second; }
    public static int callTarget(int value, int distance) { return ShiftSliceHelper.value(value) << ShiftSliceHelper.distance(distance); }
}
