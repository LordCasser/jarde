import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;

public class VerifyTest3 {
    public static void main(String[] args) throws Exception {
        Class<?> cls = Class.forName("jadx.tests.integration.trycatch.TestTryCatchFinally12$TestCls");
        Field sb = cls.getDeclaredField("sb");
        sb.setAccessible(true);
        String[] expected = {args[0], args[1], args[2]};
        for (int input = 0; input < 3; input++) {
            Object instance = cls.getConstructor().newInstance();
            StringBuilder value = new StringBuilder();
            sb.set(instance, value);
            try {
                Field second = cls.getDeclaredField("sb2");
                second.setAccessible(true);
                second.set(instance, value);
            } catch (NoSuchFieldException expectedAbsent) {
                // The fixed class and the other mutants retain one field.
            }
            Throwable thrown = null;
            try {
                cls.getMethod("test3", int.class).invoke(instance, input);
            } catch (InvocationTargetException exception) {
                thrown = exception.getCause();
            }
            if ((input == 2) != (thrown instanceof IllegalArgumentException)) {
                throw new AssertionError("wrong exceptional completion: " + input + ": " + thrown);
            }
            String actual = sb.get(instance).toString();
            if (!expected[input].equals(actual)) {
                throw new AssertionError(input + " expected=" + expected[input] + " actual=" + actual);
            }
            System.out.println(input + "=" + actual);
        }
    }
}
