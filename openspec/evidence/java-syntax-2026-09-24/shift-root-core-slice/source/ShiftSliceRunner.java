public class ShiftSliceRunner {
    public static void main(String[] args) {
        int[] ints = {-1, 1, Integer.MIN_VALUE, Integer.MAX_VALUE};
        long[] longs = {-1L, 1L, Long.MIN_VALUE, Long.MAX_VALUE};
        int[] distances = {-1, 0, 1, 31, 32, 63};
        for (int x : ints) for (int d : distances) {
            System.out.println("i:" + x + ":" + d + "=" + ShiftSlice.intLeft(x, d) + "," + ShiftSlice.intRight(x, d) + "," + ShiftSlice.intUnsigned(x, d) + "," + ShiftSlice.nested(x, d, 1) + "," + ShiftSlice.callTarget(x, d));
        }
        for (long x : longs) for (int d : distances) {
            System.out.println("l:" + x + ":" + d + "=" + ShiftSlice.longLeft(x, d) + "," + ShiftSlice.longRight(x, d) + "," + ShiftSlice.longUnsigned(x, d));
        }
        for (short x : new short[] {Short.MIN_VALUE, -1, 0, 1, Short.MAX_VALUE})
            System.out.println("s:" + x + "=" + ShiftSlice.shortLeft(x, 3));
        for (char x : new char[] {0, 1, Character.MAX_VALUE})
            System.out.println("c:" + (int)x + "=" + ShiftSlice.charUnsigned(x, 3));
    }
}
