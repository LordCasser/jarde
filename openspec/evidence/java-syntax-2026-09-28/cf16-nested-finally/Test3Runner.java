package jadx.tests.integration.trycatch;

import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;

public class Test3Runner {
    public static void main(String[] args) throws Exception {
        Class<?> cls = Class.forName("jadx.tests.integration.trycatch.FinallyMinimalProbe");
        Field sb = cls.getDeclaredField("sb");
        sb.setAccessible(true);
        String[] expected = {
            "call-finally", "call-npe-catch-finally", "call-iae-finally"
        };
        for (int input = 0; input < 3; input++) {
            Object instance = cls.getDeclaredConstructor().newInstance();
            sb.set(instance, new StringBuilder());
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
