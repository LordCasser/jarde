import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;

/** Source-only runner: it links to Order by reflection, so it also runs against patched bytes. */
public final class OrderRunner {
    public static void main(String[] args) throws Exception {
        Class<?> type = Class.forName("Order");
        Method put = type.getMethod("put", boolean[].class, int.class);
        Field trace = type.getDeclaredField("trace");
        trace.setAccessible(true);

        Object[][] cases = {
            {new boolean[1], 2},
            {null, 2},
            {new boolean[0], 2},
            {new boolean[1], 99}
        };
        String[] names = {"success", "null array", "out of bounds", "value throws"};
        for (int i = 0; i < cases.length; i++) {
            trace.setInt(null, 0);
            try {
                Object result = put.invoke(null, cases[i]);
                System.out.println(names[i] + ": return=" + result + ", trace=" + trace.getInt(null));
            } catch (InvocationTargetException exception) {
                Throwable cause = exception.getCause();
                System.out.println(names[i] + ": throws=" + cause.getClass().getSimpleName()
                        + ", trace=" + trace.getInt(null));
            }
        }
    }
}
