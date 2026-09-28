package jadx.tests.integration.trycatch;

public class CatchRunner {
    public static void main(String[] args) throws Exception {
        Class<?> type = Class.forName("jadx.tests.integration.trycatch.TestTryCatchFinally7$TestCls");
        java.lang.reflect.Method test = type.getDeclaredMethod("test", Object.class);
        java.lang.reflect.Field field = type.getDeclaredField("f");
        test.setAccessible(true);
        field.setAccessible(true);
        AssertionError sentinel = new AssertionError("sentinel");
        for (Object input : new Object[] { null, "e", sentinel }) {
            Object instance = type.getDeclaredConstructor().newInstance();
            String label = input == sentinel ? "r" : String.valueOf(input);
            try {
                Object value = test.invoke(instance, input);
                System.out.println(label + ":return=" + value + ",f=" + field.getInt(instance));
            } catch (java.lang.reflect.InvocationTargetException error) {
                Throwable cause = error.getCause();
                System.out.println(label + ":throw=" + cause.getClass().getSimpleName() + ",f=" + field.getInt(instance));
                if (input == sentinel && cause != sentinel) {
                    throw new AssertionError("Throwable identity changed", cause);
                }
            }
        }
    }
}
