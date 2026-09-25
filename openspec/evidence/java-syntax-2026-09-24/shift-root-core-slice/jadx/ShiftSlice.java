
/* JADX INFO: loaded from: ShiftSlice.class */
public class ShiftSlice {
    public static int intLeft(int i, int i2) {
        return i << i2;
    }

    public static int intRight(int i, int i2) {
        return i >> i2;
    }

    public static int intUnsigned(int i, int i2) {
        return i >>> i2;
    }

    public static long longLeft(long j, int i) {
        return j << i;
    }

    public static long longRight(long j, int i) {
        return j >> i;
    }

    public static long longUnsigned(long j, int i) {
        return j >>> i;
    }

    public static int shortLeft(short s, int i) {
        return s << i;
    }

    public static int charUnsigned(char c, int i) {
        return c >>> i;
    }

    public static int nested(int i, int i2, int i3) {
        return (i << i2) >>> i3;
    }

    public static int callTarget(int i, int i2) {
        return ShiftSliceHelper.value(i) << ShiftSliceHelper.distance(i2);
    }
}
