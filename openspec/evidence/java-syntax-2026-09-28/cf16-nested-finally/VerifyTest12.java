import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;

public class VerifyTest12 {
    public static void main(String[] args) throws Exception {
        Class<?> cls = Class.forName("jadx.tests.integration.trycatch.TestTryCatchFinally12$TestCls");
        Field sb = cls.getDeclaredField("sb");
        sb.setAccessible(true);
        for (String method : new String[] {"test1", "test2"}) {
            for (int input = 0; input < 3; input++) {
                Object instance = cls.getConstructor().newInstance();
                StringBuilder value = new StringBuilder();
                sb.set(instance, value);
                Throwable thrown = null;
                try {
                    cls.getMethod(method, int.class).invoke(instance, input);
                } catch (InvocationTargetException exception) {
                    thrown = exception.getCause();
                }
                System.out.println(method + ":" + input + "=" + value + ":" +
                    (thrown == null ? "normal" : thrown.getClass().getName()));
            }
        }
    }
}
