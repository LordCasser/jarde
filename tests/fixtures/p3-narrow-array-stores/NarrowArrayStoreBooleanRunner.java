import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;

/** Runs the verifier-valid general int-to-boolean bastore boundary. */
public final class NarrowArrayStoreBooleanRunner {
    private NarrowArrayStoreBooleanRunner() {}

    public static void main(String[] args) throws Exception {
        Method store = NarrowArrayStores.class.getMethod(
                "storeByte", boolean[].class, int.class, int.class);
        for (int value : new int[] {
                Integer.MIN_VALUE, -3, -2, -1, 0, 1, 2, 3, 127, 128,
                255, 256, 32767, 32768, 65535, 65536, Integer.MAX_VALUE }) {
            boolean[] array = { false };
            try {
                store.invoke(null, array, 0, value);
            } catch (InvocationTargetException error) {
                throw new AssertionError(error.getCause());
            }
            boolean expected = (value & 1) != 0;
            if (array[0] != expected) {
                throw new AssertionError("value " + value + " expected " + expected
                        + " got " + array[0]);
            }
            System.out.println(value + ":" + array[0]);
        }
    }
}
