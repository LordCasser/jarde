import java.lang.reflect.Array;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;

/** Source-only runner shared by the patched class and the recovered class. */
public final class NarrowArrayStoreOrderRunner {
    private NarrowArrayStoreOrderRunner() {}

    private static String shown(Object array) {
        if (array == null) return "null";
        Object value = Array.get(array, 0);
        if (value instanceof Character) return Integer.toString(((Character) value).charValue());
        return String.valueOf(value);
    }

    private static void set(Object array, Class<?> component, int index, int value) {
        if (component == byte.class) Array.setByte(array, index, (byte) value);
        else if (component == char.class) Array.setChar(array, index, (char) value);
        else Array.setShort(array, index, (short) value);
    }

    private static String run(String label, String methodName, Class<?> component,
            Object array, int index, boolean failArray, boolean failIndex, boolean failValue)
            throws Exception {
        NarrowArrayStoreOrderEffects.reset();
        NarrowArrayStoreOrderEffects.mark('P');
        try {
            Method method = NarrowArrayStoreOrder.class.getMethod(methodName,
                    Array.newInstance(component, 0).getClass(), int.class, int.class,
                    boolean.class, boolean.class, boolean.class);
            try {
                method.invoke(null, array, index, -129, failArray, failIndex, failValue);
            } catch (InvocationTargetException error) {
                throw error.getCause();
            }
            NarrowArrayStoreOrderEffects.mark('Q');
            return label + ":ok:" + NarrowArrayStoreOrderEffects.state() + ":stored:" + shown(array);
        } catch (Throwable error) {
            String detail = error instanceof NullPointerException
                    || error instanceof ArrayIndexOutOfBoundsException ? "" : error.getMessage();
            return label + ":" + error.getClass().getName() + ":" + detail
                    + ":" + NarrowArrayStoreOrderEffects.state() + ":stored:" + shown(array);
        }
    }

    private static void expect(String expected, String actual) {
        if (!expected.equals(actual)) throw new AssertionError("expected [" + expected + "] got [" + actual + "]");
        System.out.println(actual);
    }

    private static void cases(String label, String method, Class<?> component, int initial)
            throws Exception {
        Object array = Array.newInstance(component, 1);
        set(array, component, 0, initial);
        int narrowed = component == byte.class ? 127 : component == char.class ? 65407 : -129;
        expect(label + ":success:ok:PpAIVqQ:1,1,1:stored:" + narrowed,
                run(label + ":success", method, component, array, 0, false, false, false));
        set(array, component, 0, initial);
        expect(label + ":array-fail:java.lang.IllegalArgumentException:array producer:PpA:1,0,0:stored:" + initial,
                run(label + ":array-fail", method, component, array, 0, true, false, false));
        expect(label + ":index-fail:java.lang.IllegalArgumentException:index producer:PpAI:1,1,0:stored:" + initial,
                run(label + ":index-fail", method, component, array, 0, false, true, false));
        expect(label + ":value-fail:java.lang.IllegalStateException:value producer:PpAIV:1,1,1:stored:" + initial,
                run(label + ":value-fail", method, component, array, 0, false, false, true));
        expect(label + ":null-value-fail:java.lang.IllegalStateException:value producer:PpAIV:1,1,1:stored:null",
                run(label + ":null-value-fail", method, component, null, 0, false, false, true));
        expect(label + ":null-store:java.lang.NullPointerException::PpAIV:1,1,1:stored:null",
                run(label + ":null-store", method, component, null, 0, false, false, false));
        expect(label + ":oob-value-fail:java.lang.IllegalStateException:value producer:PpAIV:1,1,1:stored:" + initial,
                run(label + ":oob-value-fail", method, component, array, 1, false, false, true));
        expect(label + ":oob-store:java.lang.ArrayIndexOutOfBoundsException::PpAIV:1,1,1:stored:" + initial,
                run(label + ":oob-store", method, component, array, 1, false, false, false));
    }

    public static void main(String[] args) throws Exception {
        cases("B", "storeByte", byte.class, 41);
        cases("C", "storeChar", char.class, 41);
        cases("S", "storeShort", short.class, 41);
    }
}
