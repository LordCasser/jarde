// The P3 parameter-slot fixture: a parameter **after** an array of a `long` or a `double` is one
// slot past the array, not two.
//
// JVMS 2.6.1 gives a `long` or a `double` two local slots and every other type one — an array of
// either is a reference, so `[J` fills one slot exactly like `[I`. A signature that used the
// element's width for the array placed the parameter after it one slot too far, and the body (whose
// names come from the real slot numbering) then read a name the declaration had not declared.
//
// Every member here is that shape or a control for it, and each body reads the parameter the
// descriptor places last so that a wrong slot in the declaration is a name the body does not use.
public class SlotTypes {
    public int offset;

    public SlotTypes() {
        offset = 3;
    }

    public static int afterArray(long[] values, int count) {
        return count;
    }

    public static int afterNested(long[][] values, int count) {
        return count;
    }

    public static long afterGrid(double[][] grid, long count) {
        return count;
    }

    public static int betweenWide(long head, long[] tail, int count) {
        return count;
    }

    public static long arraysOnly(long[] heads, double[] tails, long count) {
        return count;
    }

    public static long[] echoArray(long[] values) {
        return values;
    }

    public static int control(int[] values, int count) {
        return count;
    }

    public int instanceAfterArray(long[] values, int count) {
        return count;
    }

    public int instanceWide(long head, long[] tail, int count) {
        return count;
    }
}
