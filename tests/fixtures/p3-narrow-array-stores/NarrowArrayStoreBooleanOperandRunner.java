import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;

/** Source-only verifier and execution check for the boolean-derived byte store. */
public final class NarrowArrayStoreBooleanOperandRunner {
    private NarrowArrayStoreBooleanOperandRunner() {}

    public static void main(String[] args) throws Exception {
        Method store = NarrowArrayStoreBooleanOperand.class.getMethod(
                "storeByteOperand", byte[].class, int.class, boolean.class);
        for (boolean value : new boolean[] { false, true }) {
            byte[] array = { 37 };
            try {
                store.invoke(null, array, 0, value);
            } catch (InvocationTargetException error) {
                throw new AssertionError(error.getCause());
            }
            byte expected = (byte) (value ? 1 : 0);
            if (array[0] != expected) {
                throw new AssertionError("expected " + expected + " got " + array[0]);
            }
            System.out.println(value + ":" + array[0]);
        }
    }
}
