import java.lang.reflect.Method;

/** Source-only runner: it links to RawBool by reflection, so it also runs against patched bytes. */
public final class RawBoolRunner {
    public static void main(String[] args) throws Exception {
        Method put = RawBool.class.getMethod("put", boolean[].class, int.class, int.class);
        int[] values = {0, 1, 2, 3, -1, -2, Integer.MIN_VALUE, Integer.MAX_VALUE};
        for (int value : values) {
            boolean[] array = {false};
            Object returned = put.invoke(null, array, 0, value);
            System.out.println(value + " -> " + returned + " / " + array[0]);
        }
    }
}
